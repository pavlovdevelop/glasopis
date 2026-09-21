import { useEffect, useState } from "react";
import { Banner, Button, Card, Row } from "../components/ui";
import { useAppState } from "../hooks/useAppState";
import { useUpdater } from "../hooks/useUpdater";
import { api, errorMessage, type AppInfo } from "../services/api";

const REPOSITORY = "https://github.com/pavlovdevelop/glasopis";

export function AboutPage() {
  const { t, setError, settings } = useAppState();
  const [info, setInfo] = useState<AppInfo | null>(null);
  const updater = useUpdater();

  useEffect(() => {
    api.getAppInfo().then(setInfo).catch((err) => setError(errorMessage(err)));
  }, [setError]);

  const openFolder = (which: "models" | "logs" | "config") => {
    api.openFolder(which).catch((err) => setError(errorMessage(err)));
  };

  return (
    <Card title={t("about.title")} description={t("app.tagline")}>
      <p className="hint">{t("about.openSource")}</p>
      <p className="hint">{t("about.free")}</p>
      <Row label={t("about.version")}>
        <span>{info?.version ?? "—"}</span>
      </Row>
      <Row label={t("update.title")}>
        {updater.state.phase === "idle" || updater.state.phase === "up-to-date" ? (
          <Button onClick={() => void updater.check()}>{t("update.checkNow")}</Button>
        ) : updater.state.phase === "checking" ? (
          <span className="hint">{t("update.checking")}</span>
        ) : updater.state.phase === "error" ? (
          <span className="hint">{t("update.error")}</span>
        ) : null}
      </Row>
      {updater.state.phase === "up-to-date" && (
        <Banner kind="success">{t("update.upToDate")}</Banner>
      )}
      {updater.state.phase === "error" && (
        <Banner kind="error">{updater.state.message}</Banner>
      )}
      {updater.state.phase === "available" && (
        <Banner kind="info">
          <p>{t("update.available", { version: updater.state.version })}</p>
          {updater.state.notes && (
            <details>
              <summary>{t("update.notes")}</summary>
              <p className="hint">{updater.state.notes}</p>
            </details>
          )}
          <Button variant="primary" onClick={() => void updater.install()}>
            {t("update.install")}
          </Button>
        </Banner>
      )}
      {updater.state.phase === "downloading" && (
        <Banner kind="info">
          {updater.state.progress !== null ? (
            <div className="progress">
              <div
                className="progress__bar"
                style={{ width: `${Math.round(updater.state.progress * 100)}%` }}
              />
              <span className="progress__label">
                {t("update.downloading", { percent: Math.round(updater.state.progress * 100) })}
              </span>
            </div>
          ) : (
            <p>{t("update.downloadingUnknown")}</p>
          )}
        </Banner>
      )}
      {updater.state.phase === "ready" && <Banner kind="success">{t("update.ready")}</Banner>}
      <Row label={t("about.speechEngine")}>
        <span>
          {settings?.voice.engine === "local"
            ? info?.local_engine_available
              ? t("about.speechEngineLocal")
              : t("about.speechEngineMissing")
            : t("about.speechEngineCloud")}
        </span>
      </Row>
      <Row label={t("about.license")}>
        <span>MIT</span>
      </Row>
      <Row label={t("about.repository")}>
        <code>{REPOSITORY}</code>
      </Row>
      <Row label={t("about.config")} hint={info?.config_dir}>
        <Button onClick={() => openFolder("config")}>{t("common.openFolder")}</Button>
      </Row>
      <Row label={t("about.models")} hint={info?.models_dir}>
        <Button onClick={() => openFolder("models")}>{t("common.openFolder")}</Button>
      </Row>
      <Row label={t("about.logs")} hint={info?.logs_dir}>
        <Button onClick={() => openFolder("logs")}>{t("common.openFolder")}</Button>
      </Row>
    </Card>
  );
}
