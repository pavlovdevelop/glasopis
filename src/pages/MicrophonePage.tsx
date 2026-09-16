import { useCallback, useEffect, useState } from "react";
import { Banner, Button, Card, LevelMeter, Row, Select } from "../components/ui";
import { useAppState } from "../hooks/useAppState";
import { useStatus } from "../hooks/useStatus";
import { api, errorMessage, type InputDevice } from "../services/api";

export function MicrophonePage() {
  const { settings, save, t, setError, error } = useAppState();
  const { status, level } = useStatus();
  const [devices, setDevices] = useState<InputDevice[]>([]);

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

  const recording = status.state === "listening";

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
        <Button
          variant={recording ? "danger" : "primary"}
          onClick={() => (recording ? api.cancelDictation() : api.startDictation())}
        >
          {recording ? t("microphone.stopTest") : t("microphone.test")}
        </Button>
        <Button onClick={() => void refresh()}>{t("microphone.reload")}</Button>
      </div>
      {recording && <p className="hint">{t("microphone.testing")}</p>}
    </Card>
  );
}
