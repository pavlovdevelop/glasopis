import { useEffect, useState } from "react";
import { Button, Card, Row } from "../components/ui";
import { useAppState } from "../hooks/useAppState";
import { api, errorMessage, type AppInfo } from "../services/api";

const REPOSITORY = "https://github.com/pavlovdevelop/glasopis";

export function AboutPage() {
  const { t, setError } = useAppState();
  const [info, setInfo] = useState<AppInfo | null>(null);

  useEffect(() => {
    api.getAppInfo().then(setInfo).catch((err) => setError(errorMessage(err)));
  }, [setError]);

  return (
    <Card title={t("about.title")} description={t("app.tagline")}>
      <p className="hint">{t("about.openSource")}</p>
      <p className="hint">{t("about.free")}</p>
      <Row label={t("about.version")}>
        <span>{info?.version ?? "—"}</span>
      </Row>
      <Row label={t("about.speechEngine")}>
        <span>
          {info?.speech_available ? t("about.speechEngineLocal") : t("about.speechEngineMissing")}
        </span>
      </Row>
      <Row label={t("about.license")}>
        <span>MIT</span>
      </Row>
      <Row label={t("about.repository")}>
        <code>{REPOSITORY}</code>
      </Row>
      <Row label={t("about.config")} hint={info?.config_dir}>
        <Button onClick={() => api.openFolder("config")}>{t("common.openFolder")}</Button>
      </Row>
      <Row label={t("about.models")} hint={info?.models_dir}>
        <Button onClick={() => api.openFolder("models")}>{t("common.openFolder")}</Button>
      </Row>
      <Row label={t("about.logs")} hint={info?.logs_dir}>
        <Button onClick={() => api.openFolder("logs")}>{t("common.openFolder")}</Button>
      </Row>
    </Card>
  );
}
