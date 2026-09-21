import { useEffect, useState } from "react";
import { AppStateProvider, useAppState } from "./hooks/useAppState";
import { UpdaterProvider, useUpdater } from "./hooks/useUpdater";
import { useStatus } from "./hooks/useStatus";
import { api, errorMessage } from "./services/api";
import { GeneralPage } from "./pages/GeneralPage";
import { VoicePage } from "./pages/VoicePage";
import { HotkeysPage } from "./pages/HotkeysPage";
import { InsertionPage } from "./pages/InsertionPage";
import { ModelsPage } from "./pages/ModelsPage";
import { MicrophonePage } from "./pages/MicrophonePage";
import { PrivacyPage } from "./pages/PrivacyPage";
import { HistoryPage } from "./pages/HistoryPage";
import { AboutPage } from "./pages/AboutPage";
import { RecognitionPage } from "./pages/RecognitionPage";
import { Onboarding } from "./pages/Onboarding";
import { Overlay } from "./pages/Overlay";
import { Banner, Button } from "./components/ui";

type Route =
  | "settings"
  | "recognition"
  | "voice"
  | "hotkeys"
  | "insertion"
  | "models"
  | "microphone"
  | "privacy"
  | "history"
  | "about"
  | "onboarding"
  | "overlay";

const ROUTES: { id: Route; labelKey: string }[] = [
  { id: "settings", labelKey: "nav.general" },
  { id: "recognition", labelKey: "nav.recognition" },
  { id: "voice", labelKey: "nav.voice" },
  { id: "microphone", labelKey: "nav.microphone" },
  { id: "models", labelKey: "nav.models" },
  { id: "hotkeys", labelKey: "nav.hotkeys" },
  { id: "insertion", labelKey: "nav.insertion" },
  { id: "privacy", labelKey: "nav.privacy" },
  { id: "history", labelKey: "nav.history" },
  { id: "about", labelKey: "nav.about" },
];

function routeFromHash(): Route {
  const raw = window.location.hash.replace(/^#\/?/, "");
  const known = [...ROUTES.map((route) => route.id), "onboarding", "overlay"];
  return (known.includes(raw) ? raw : "settings") as Route;
}

function useRoute(): [Route, (route: Route) => void] {
  const [route, setRoute] = useState<Route>(routeFromHash);
  useEffect(() => {
    const onHashChange = () => setRoute(routeFromHash());
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, []);
  return [
    route,
    (next: Route) => {
      window.location.hash = `#/${next}`;
      setRoute(next);
    },
  ];
}

function Page({ route }: { route: Route }) {
  switch (route) {
    case "recognition":
      return <RecognitionPage />;
    case "voice":
      return <VoicePage />;
    case "hotkeys":
      return <HotkeysPage />;
    case "insertion":
      return <InsertionPage />;
    case "models":
      return <ModelsPage />;
    case "microphone":
      return <MicrophonePage />;
    case "privacy":
      return <PrivacyPage />;
    case "history":
      return <HistoryPage />;
    case "about":
      return <AboutPage />;
    default:
      return <GeneralPage />;
  }
}

/** Compact, app-wide notice - the full details/progress live on the About page. */
function UpdateBanner({ onOpenAbout }: { onOpenAbout: () => void }) {
  const { t } = useAppState();
  const { state } = useUpdater();

  if (state.phase === "available") {
    return (
      <Banner kind="info">
        {t("update.available", { version: state.version })}{" "}
        <Button variant="primary" onClick={onOpenAbout}>
          {t("update.install")}
        </Button>
      </Banner>
    );
  }
  if (state.phase === "downloading") {
    return (
      <Banner kind="info">
        {state.progress !== null
          ? t("update.downloading", { percent: Math.round(state.progress * 100) })
          : t("update.downloadingUnknown")}
      </Banner>
    );
  }
  if (state.phase === "ready") {
    return <Banner kind="success">{t("update.ready")}</Banner>;
  }
  return null;
}

function StatusBar() {
  const { t, settings, setError } = useAppState();
  const { status } = useStatus();
  if (!settings) return null;

  const label =
    status.state === "listening"
      ? t("status.listening")
      : status.state === "processing"
        ? t("status.processing")
        : status.state === "error"
          ? status.message
          : t("status.idle");

  const recording = status.state === "listening";

  return (
    <div className="statusbar">
      <span className={`statusbar__dot statusbar__dot--${status.state}`} aria-hidden="true" />
      <span className="statusbar__label">{label}</span>
      <span className="statusbar__hint">
        {t("dictation.hint", { hotkey: settings.hotkeys.toggle })}
      </span>
      <Button
        variant={recording ? "danger" : "primary"}
        onClick={() => {
          api.toggleDictation().catch((err) => setError(errorMessage(err)));
        }}
      >
        {recording ? t("dictation.stop") : t("dictation.start")}
      </Button>
    </div>
  );
}

function Shell() {
  const { settings, t, error, setError } = useAppState();
  const [route, setRoute] = useRoute();
  const [localAvailable, setLocalAvailable] = useState(false);
  const [justUpdatedTo, setJustUpdatedTo] = useState<string | null>(null);

  useEffect(() => {
    api
      .getAppInfo()
      .then((info) => setLocalAvailable(info.local_engine_available))
      .catch(() => setLocalAvailable(false));
  }, []);

  useEffect(() => {
    api.takeUpdateNotice().then(setJustUpdatedTo).catch(() => undefined);
  }, []);

  if (route === "overlay") return <Overlay />;
  if (!settings) return <div className="loading">{t("common.loading")}</div>;

  if (route === "onboarding" || !settings.onboarding_completed) {
    return <Onboarding onFinished={() => setRoute("settings")} />;
  }

  return (
    <UpdaterProvider>
      <div className="shell">
        <aside className="sidebar">
          <div className="brand">
            <span className="brand__mark" aria-hidden="true">
              <img src="/logo-mark-white.png" alt="" width="20" height="20" />
            </span>
            <div>
              <p className="brand__name">{t("app.name")}</p>
              <p className="brand__tagline">{t("app.tagline")}</p>
            </div>
          </div>
          <nav className="nav" aria-label={t("nav.general")}>
            {ROUTES.filter((item) => item.id !== "models" || localAvailable).map((item) => (
              <button
                key={item.id}
                type="button"
                className={`nav__item ${route === item.id ? "nav__item--active" : ""}`}
                aria-current={route === item.id ? "page" : undefined}
                onClick={() => setRoute(item.id)}
              >
                {t(item.labelKey)}
              </button>
            ))}
          </nav>
        </aside>
        <main className="content">
          <StatusBar />
          {justUpdatedTo && (
            <Banner kind="success">
              {t("update.justUpdated", { version: justUpdatedTo })}{" "}
              <Button variant="ghost" onClick={() => setJustUpdatedTo(null)}>
                {t("common.close")}
              </Button>
            </Banner>
          )}
          <UpdateBanner onOpenAbout={() => setRoute("about")} />
          {error && (
            <Banner kind="error">
              {error}{" "}
              <Button variant="ghost" onClick={() => setError(null)}>
                {t("common.close")}
              </Button>
            </Banner>
          )}
          <Page route={route} />
        </main>
      </div>
    </UpdaterProvider>
  );
}

export default function App() {
  return (
    <AppStateProvider>
      <Shell />
    </AppStateProvider>
  );
}
