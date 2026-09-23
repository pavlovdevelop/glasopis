import { Banner, Card, NumberField, Toggle } from "../components/ui";
import { useAppState } from "../hooks/useAppState";

export function PrivacyPage() {
  const { settings, save, t } = useAppState();
  if (!settings) return null;

  const privacy = settings.privacy;
  const update = (patch: Partial<typeof privacy>) =>
    void save({ ...settings, privacy: { ...privacy, ...patch } });

  const cloud = settings.voice.engine === "groq";

  return (
    <Card title={t("privacy.title")}>
      {cloud ? (
        <Banner kind="info">
          <strong>{t("privacy.cloud")}</strong>
          <br />
          {t("privacy.cloudText")}
        </Banner>
      ) : (
        <Banner kind="success">
          <strong>{t("privacy.local")}</strong>
          <br />
          {t("privacy.localText")}
        </Banner>
      )}
      <p className="hint">{t("privacy.noTelemetry")}</p>
      <p className="hint">{t("privacy.audioDeleted")}</p>
      {cloud && <p className="hint">{t("privacy.switchToLocal")}</p>}
      {settings.assistant.enabled && (
        <Banner kind="info">{t("privacy.assistantText")}</Banner>
      )}
      <Toggle
        id="keep-history"
        label={t("privacy.keepHistory")}
        hint={t("privacy.historyOff")}
        checked={privacy.keep_history}
        onChange={(keep_history) => update({ keep_history })}
      />
      <NumberField
        id="history-limit"
        label={t("privacy.historyLimit")}
        value={privacy.history_limit}
        min={10}
        max={5000}
        onChange={(history_limit) => update({ history_limit })}
      />
    </Card>
  );
}
