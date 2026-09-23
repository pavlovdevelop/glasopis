import { Banner, Card, Row, Toggle } from "../components/ui";
import { HotkeyInput } from "../components/HotkeyInput";
import { useAppState } from "../hooks/useAppState";

const DEFAULT_ASSISTANT_HOTKEY = "Ctrl+Alt+K";

export function AssistantPage() {
  const { settings, save, t, error } = useAppState();
  if (!settings) return null;

  const assistant = settings.assistant;
  const update = (patch: Partial<typeof assistant>) => {
    void save({ ...settings, assistant: { ...assistant, ...patch } });
  };

  return (
    <Card title={t("assistant.title")} description={t("assistant.description")}>
      {error && <Banner kind="error">{error}</Banner>}
      <Banner kind="info">{t("assistant.confirmHint")}</Banner>
      <Toggle
        id="assistant-enabled"
        label={t("assistant.enable")}
        hint={t("assistant.enableHint")}
        checked={assistant.enabled}
        onChange={(enabled) => update({ enabled })}
      />
      <Row label={t("assistant.hotkey")} hint={t("assistant.hotkeyHint")}>
        <HotkeyInput
          value={assistant.hotkey || DEFAULT_ASSISTANT_HOTKEY}
          placeholder={t("hotkeys.press")}
          disabled={!assistant.enabled}
          onChange={(hotkey) => update({ hotkey })}
        />
      </Row>
      <Toggle
        id="assistant-speak"
        label={t("assistant.speakReplies")}
        hint={t("assistant.speakRepliesHint")}
        checked={assistant.speak_replies}
        disabled={!assistant.enabled}
        onChange={(speak_replies) => update({ speak_replies })}
      />
      {!settings.cloud.api_key && (
        <Banner kind="error">{t("assistant.needsApiKey")}</Banner>
      )}
    </Card>
  );
}
