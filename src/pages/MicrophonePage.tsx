import { useCallback, useEffect, useState } from "react";
import { Banner, Button, Card, LevelMeter, Row, Select } from "../components/ui";
import { useAppState } from "../hooks/useAppState";
import { useStatus } from "../hooks/useStatus";
import { api, errorMessage, type InputDevice } from "../services/api";

export function MicrophonePage() {
  const { settings, save, t, setError, error } = useAppState();
  const { status, level } = useStatus();
  const [devices, setDevices] = useState<InputDevice[]>([]);
  const [testing, setTesting] = useState(false);

  const refresh = useCallback(async () => {
    try {
      setDevices(await api.listMicrophones());
      setError(null);
    } catch (err) {
      setDevices([]);
      setError(errorMessage(err));
    }
  }, [setError]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  if (!settings) return null;

  const current =
    settings.voice.microphone.kind === "device" ? settings.voice.microphone.name : "";

  const select = (name: string) =>
    void save({
      ...settings,
      voice: {
        ...settings.voice,
        microphone: name === "" ? { kind: "default" } : { kind: "device", name },
      },
    });

  // Тестът на микрофона само записва - не изисква модел и не въвежда текст.
  const toggleTest = async () => {
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
  };

  // Ако диктовка спре записа, тестът също е приключил.
  useEffect(() => {
    if (testing && status.state !== "idle" && status.state !== "listening") {
      setTesting(false);
    }
  }, [status, testing]);

  return (
    <Card title={t("microphone.title")}>
      {error && <Banner kind="error">{error}</Banner>}
      <Select
        id="microphone"
        label={t("microphone.select")}
        value={current}
        options={[
          { value: "", label: t("voice.defaultMicrophone") },
          ...devices.map((device) => ({
            value: device.name,
            label: device.is_default ? `${device.name} ★` : device.name,
          })),
        ]}
        onChange={select}
      />
      <Row label={t("microphone.level")}>
        <LevelMeter level={level} />
      </Row>
      <div className="actions">
        <Button variant={testing ? "danger" : "primary"} onClick={() => void toggleTest()}>
          {testing ? t("microphone.stopTest") : t("microphone.test")}
        </Button>
        <Button onClick={() => void refresh()}>{t("microphone.reload")}</Button>
      </div>
      {testing && <p className="hint">{t("microphone.testing")}</p>}
    </Card>
  );
}
