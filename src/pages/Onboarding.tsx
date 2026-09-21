import { useEffect, useState } from "react";
import { Banner, Button, Card, LevelMeter, Select } from "../components/ui";
import { HotkeyInput } from "../components/HotkeyInput";
import { useAppState } from "../hooks/useAppState";
import { useStatus } from "../hooks/useStatus";
import {
  api,
  errorMessage,
  events,
  formatBytes,
  type InputDevice,
  type ModelStatus,
} from "../services/api";

const TOTAL_STEPS = 5;

export function Onboarding({ onFinished }: { onFinished: () => void }) {
  const { settings, save, reload, t, setError, error } = useAppState();
  const { status, level } = useStatus();
  const [step, setStep] = useState(0);
  const [devices, setDevices] = useState<InputDevice[]>([]);
  const [models, setModels] = useState<ModelStatus[]>([]);
  const [progress, setProgress] = useState<number | null>(null);
  const [dictated, setDictated] = useState("");
  const [testing, setTesting] = useState(false);

  useEffect(() => {
    api.listMicrophones().then(setDevices).catch((err) => setError(errorMessage(err)));
    api.listModels().then(setModels).catch((err) => setError(errorMessage(err)));
  }, [setError]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void events
      .onModel((event) => {
        if (event.state === "downloading") setProgress(event.percent);
        else if (event.state === "verifying") setProgress(100);
        else {
          setProgress(null);
          if (event.state === "failed") setError(event.message);
          api.listModels().then(setModels).catch(() => undefined);
        }
      })
      .then((un) => {
        unlisten = un;
      });
    return () => unlisten?.();
  }, [setError]);

  useEffect(() => {
    if (status.state === "done") setDictated(status.text);
  }, [status]);

  if (!settings) return null;

  const hasModel = models.some((model) => model.downloaded);
  const recommended = models.find((model) => model.recommended) ?? models[0];
  const cloud = settings.voice.engine === "groq";

  const steps = [
    {
      title: t("onboarding.welcomeTitle"),
      text: t("onboarding.welcomeText"),
      body: (
        <div className="onboarding__hero">
          <p className="onboarding__tagline">{t("app.tagline")}</p>
          <p className="hint">{t("privacy.localText")}</p>
        </div>
      ),
      canContinue: true,
    },
    {
      title: t("onboarding.micTitle"),
      text: t("onboarding.micText"),
      body: (
        <>
          <Select
            id="onboarding-mic"
            label={t("microphone.select")}
            value={
              settings.voice.microphone.kind === "device" ? settings.voice.microphone.name : ""
            }
            options={[
              { value: "", label: t("voice.defaultMicrophone") },
              ...devices.map((device) => ({ value: device.name, label: device.name })),
            ]}
            onChange={(name) =>
              void save({
                ...settings,
                voice: {
                  ...settings.voice,
                  microphone: name === "" ? { kind: "default" } : { kind: "device", name },
                },
              })
            }
          />
          <div className="actions">
            <LevelMeter level={level} />
            <Button
              onClick={async () => {
                setError(null);
                try {
                  if (testing) {
                    await api.stopMicrophoneTest();
                    setTesting(false);
                  } else {
                    await api.startMicrophoneTest();
                    setTesting(true);
                  }
                } catch (err) {
                  setTesting(false);
                  setError(errorMessage(err));
                }
              }}
            >
              {testing ? t("microphone.stopTest") : t("microphone.test")}
            </Button>
          </div>
        </>
      ),
      canContinue: devices.length > 0,
    },
    cloud
      ? {
          title: t("onboarding.keyTitle"),
          text: t("onboarding.keyText"),
          body: (
            <>
              <Banner kind="info">{t("recognition.cloudWarning")}</Banner>
              <input
                className="input"
                type="password"
                value={settings.cloud.api_key}
                placeholder={t("recognition.apiKeyPlaceholder")}
                autoComplete="off"
                spellCheck={false}
                onChange={(event) =>
                  void save({
                    ...settings,
                    cloud: { ...settings.cloud, api_key: event.target.value },
                  })
                }
              />
              <p className="hint">{t("recognition.apiKeyHint")}</p>
              <div className="actions">
                <Button
                  onClick={() => {
                    api
                      .openUrl("https://console.groq.com/keys")
                      .catch((err) => setError(errorMessage(err)));
                  }}
                >
                  {t("recognition.getKey")}
                </Button>
              </div>
            </>
          ),
          canContinue: settings.cloud.api_key.trim().length > 0,
        }
      : {
      title: t("onboarding.modelTitle"),
      text: t("onboarding.modelText"),
      body: (
        <>
          {recommended && (
            <div className="onboarding__model">
              <p>
                <strong>{recommended.label}</strong> - {recommended.technical_name} ·{" "}
                {formatBytes(recommended.size_bytes)}
              </p>
              {progress !== null ? (
                <div className="progress">
                  <div className="progress__bar" style={{ width: `${progress}%` }} />
                  <span className="progress__label">
                    {t("models.downloading")} {Math.round(progress)}%
                  </span>
                </div>
              ) : hasModel ? (
                <Banner kind="success">{t("models.downloaded")}</Banner>
              ) : (
                <Button variant="primary" onClick={() => api.downloadModel(recommended.id)}>
                  {t("common.download")}
                </Button>
              )}
            </div>
          )}
        </>
      ),
      canContinue: hasModel,
    },
    {
      title: t("onboarding.hotkeyTitle"),
      text: t("onboarding.hotkeyText"),
      body: (
        <div className="actions">
          <HotkeyInput
            value={settings.hotkeys.toggle}
            placeholder={t("hotkeys.press")}
            onChange={(toggle) =>
              void save({ ...settings, hotkeys: { ...settings.hotkeys, toggle } })
            }
          />
        </div>
      ),
      canContinue: true,
    },
    {
      title: t("onboarding.testTitle"),
      text: t("onboarding.testText"),
      body: (
        <>
          <textarea
            className="input input--textarea"
            value={dictated}
            placeholder={t("onboarding.testPlaceholder")}
            onChange={(event) => setDictated(event.target.value)}
            rows={4}
          />
          <p className="hint">
            {t("dictation.hint", { hotkey: settings.hotkeys.toggle })}
          </p>
        </>
      ),
      canContinue: true,
    },
  ];

  const current = steps[step];

  return (
    <div className="onboarding">
      <Card title={current.title} description={current.text}>
        {error && <Banner kind="error">{error}</Banner>}
        {current.body}
        <div className="onboarding__footer">
          <span className="hint">
            {t("onboarding.step", { current: step + 1, total: TOTAL_STEPS })}
          </span>
          <div className="actions">
            {step > 0 && <Button onClick={() => setStep(step - 1)}>{t("common.back")}</Button>}
            {step < TOTAL_STEPS - 1 ? (
              <Button
                variant="primary"
                disabled={!current.canContinue}
                onClick={() => setStep(step + 1)}
              >
                {t("common.next")}
              </Button>
            ) : (
              <Button
                variant="primary"
                onClick={async () => {
                  try {
                    await api.completeOnboarding();
                    await reload();
                    onFinished();
                  } catch (err) {
                    setError(errorMessage(err));
                  }
                }}
              >
                {t("common.finish")}
              </Button>
            )}
          </div>
        </div>
      </Card>
    </div>
  );
}
