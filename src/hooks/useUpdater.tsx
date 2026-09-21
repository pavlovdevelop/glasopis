import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { api, errorMessage } from "../services/api";
import { useAppState } from "./useAppState";

export type UpdateState =
  | { phase: "idle" }
  | { phase: "checking" }
  | { phase: "up-to-date" }
  | { phase: "available"; version: string; notes: string | null }
  /** `progress` is 0–1, or `null` when the server did not report a size. */
  | { phase: "downloading"; version: string; progress: number | null }
  | { phase: "ready" }
  | { phase: "error"; message: string };

interface UpdaterValue {
  state: UpdateState;
  check: () => Promise<void>;
  install: () => Promise<void>;
}

const UpdaterContext = createContext<UpdaterValue | null>(null);

/**
 * Checks GitHub Releases for a newer build and installs it. On Windows,
 * `downloadAndInstall` hands off to the NSIS installer and exits Glasopis -
 * the installer restarts it, so there is nothing left for this hook to do
 * once installation starts.
 *
 * Lives as a single instance in `UpdaterProvider` (checks once, automatically,
 * for the whole app) so every page - the sidebar banner and the About page's
 * own button - shows the same state instead of triggering separate checks.
 */
function useUpdaterInternal() {
  const [state, setState] = useState<UpdateState>({ phase: "idle" });
  const pending = useRef<Update | null>(null);

  const runCheck = useCallback(async () => {
    setState({ phase: "checking" });
    try {
      const update = await check();
      if (update) {
        pending.current = update;
        setState({ phase: "available", version: update.version, notes: update.body ?? null });
      } else {
        setState({ phase: "up-to-date" });
      }
    } catch (err) {
      setState({ phase: "error", message: errorMessage(err) });
    }
  }, []);

  const install = useCallback(async () => {
    const update = pending.current;
    if (!update) return;
    let total = 0;
    let downloaded = 0;
    try {
      // Windows exits the app as soon as the installer launches successfully
      // (see downloadAndInstall's docs) - write the marker first, or a crash
      // between that exit and a later write would lose it.
      await api.markPendingRelaunch().catch(() => undefined);
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          total = event.data.contentLength ?? 0;
          setState({ phase: "downloading", version: update.version, progress: total ? 0 : null });
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          setState({
            phase: "downloading",
            version: update.version,
            progress: total ? Math.min(1, downloaded / total) : null,
          });
        } else if (event.event === "Finished") {
          setState({ phase: "ready" });
        }
      });
    } catch (err) {
      setState({ phase: "error", message: errorMessage(err) });
    }
  }, []);

  return { state, check: runCheck, install };
}

export function UpdaterProvider({ children }: { children: ReactNode }) {
  const value = useUpdaterInternal();
  const { settings } = useAppState();
  const autoCheck = settings?.general.check_for_updates ?? false;
  const { check: runCheck } = value;

  useEffect(() => {
    if (autoCheck) void runCheck();
  }, [autoCheck, runCheck]);

  return <UpdaterContext.Provider value={value}>{children}</UpdaterContext.Provider>;
}

export function useUpdater(): UpdaterValue {
  const value = useContext(UpdaterContext);
  if (!value) throw new Error("useUpdater трябва да е вътре в UpdaterProvider");
  return value;
}
