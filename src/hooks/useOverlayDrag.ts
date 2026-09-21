import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "../services/api";

const SAVE_DELAY_MS = 500;

/**
 * Persists the floating overlay's position after the user drags it
 * (`data-tauri-drag-region` moves the OS window; this only remembers where
 * it ended up), so it reopens in the same spot next time.
 */
export function useOverlayDrag(): void {
  useEffect(() => {
    const win = getCurrentWindow();
    let timer: ReturnType<typeof setTimeout> | undefined;
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    void win
      .onMoved(({ payload }) => {
        clearTimeout(timer);
        timer = setTimeout(() => {
          void win.scaleFactor().then((factor) => {
            const logical = payload.toLogical(factor);
            void api.saveOverlayPosition(Math.round(logical.x), Math.round(logical.y));
          });
        }, SAVE_DELAY_MS);
      })
      .then((un) => {
        if (cancelled) un();
        else unlisten = un;
      });

    return () => {
      cancelled = true;
      clearTimeout(timer);
      unlisten?.();
    };
  }, []);
}
