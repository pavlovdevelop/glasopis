import { useEffect, useState } from "react";
import { Banner, Button, Card, Row, Select } from "../components/ui";
import { useAppState } from "../hooks/useAppState";
import {
  api,
  errorMessage,
  type CloudModelInfo,
  type SpeechEngineKind,
} from "../services/api";

/// Имената на моделите живеят в преводите, а не в бекенда.
const MODEL_LABELS: Record<string, string> = {
  "whisper-large-v3-turbo": "recognition.modelTurbo",
  "whisper-large-v3": "recognition.modelLarge",
};

const KEYS_URL = "https://console.groq.com/keys";

export function RecognitionPage() {
  const { settings, save, t, setError, error } = useAppState();
  const [localAvailable, setLocalAvailable] = useState(false);
  const [visible, setVisible] = useState(false);
  const [checking, setChecking] = useState(false);
  const [valid, setValid] = useState(false);
  const [key, setKey] = useState("");
  const [models, setModels] = useState<CloudModelInfo[]>([]);

  useEffect(() => {
    api
      .getAppInfo()
      .then((info) => setLocalAvailable(info.local_engine_available))
      .catch(() => setLocalAvailable(false));
  }, []);

  useEffect(() => {
    if (settings) setKey(settings.cloud.api_key);
  }, [settings]);

  useEffect(() => {
    api.listCloudModels().then(setModels).catch(() => setModels([]));
  }, []);

  if (!settings) return null;

  const engine = settings.voice.engine;

  const saveKey = async (next: string) => {
    setValid(false);
    await save({ ...settings, cloud: { ...settings.cloud, api_key: next } });
  };

  const check = async () => {
    setChecking(true);
    setError(null);
    setValid(false);
    try {
      await save({ ...settings, cloud: { ...settings.cloud, api_key: key } });
      await api.checkApiKey();
      setValid(true);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setChecking(false);
    }
  };

  return (
    <Card title={t("recognition.title")}>
      {error && <Banner kind="error">{error}</Banner>}
      {valid && <Banner kind="success">{t("recognition.keyValid")}</Banner>}

      <Select<SpeechEngineKind>
        id="engine"
        label={t("recognition.engine")}
        value={engine}
        options={[
          { value: "groq", label: t("recognition.engineGroq") },
          {
            value: "local",
            label: localAvailable
              ? t("recognition.engineLocal")
              : t("recognition.engineLocalMissing"),
          },
        ]}
        onChange={(next) =>
          void save({ ...settings, voice: { ...settings.voice, engine: next } })
        }
      />

      {engine === "groq" && (
        <>
          <Banner kind="info">{t("recognition.cloudWarning")}</Banner>
          <Select
            id="cloud-model"
            label={t("recognition.model")}
            hint={t("recognition.modelHint")}
            value={settings.cloud.model}
            options={models.map((model) => ({
              value: model.id,
              label: `${t(MODEL_LABELS[model.id] ?? model.id)} — ${model.word_error_rate}% ${t(
                "recognition.modelAccuracy",
              )}`,
            }))}
            onChange={(model) =>
              void save({ ...settings, cloud: { ...settings.cloud, model } })
            }
          />
          <Row label={t("recognition.apiKey")} hint={t("recognition.apiKeyHint")} htmlFor="api-key">
            <div className="key">
              <input
                id="api-key"
                className="input"
                type={visible ? "text" : "password"}
                value={key}
                placeholder={t("recognition.apiKeyPlaceholder")}
                autoComplete="off"
                spellCheck={false}
                onChange={(event) => setKey(event.target.value)}
                onBlur={() => void saveKey(key)}
              />
              <Button variant="ghost" onClick={() => setVisible(!visible)}>
                {visible ? t("recognition.hide") : t("recognition.show")}
              </Button>
            </div>
          </Row>
          <div className="actions">
            <Button variant="primary" disabled={checking || !key.trim()} onClick={() => void check()}>
              {checking ? t("recognition.checking") : t("recognition.check")}
            </Button>
            <Button
              onClick={() => {
                api.openUrl(KEYS_URL).catch((err) => setError(errorMessage(err)));
              }}
            >
              {t("recognition.getKey")}
            </Button>
          </div>
        </>
      )}

      {engine === "local" && !localAvailable && (
        <Banner kind="error">{t("recognition.engineLocalMissing")}</Banner>
      )}
    </Card>
  );
}
