import { useState } from "react";
import { Banner, Button, Card, Row, Toggle } from "../components/ui";
import { HotkeyInput } from "../components/HotkeyInput";
import { useAppState } from "../hooks/useAppState";

const DEFAULT_TOGGLE = "Ctrl+Alt+Space";
const DEFAULT_PUSH_TO_TALK = "Ctrl+Alt+D";

export function HotkeysPage() {
  const { settings, save, t, error } = useAppState();
  const [pending, setPending] = useState<string | null>(null);
  if (!settings) return null;

  const hotkeys = settings.hotkeys;
  const update = (patch: Partial<typeof hotkeys>) => {
    setPending(null);
    void save({ ...settings, hotkeys: { ...hotkeys, ...patch } });
  };

  return (
    <Card title={t("hotkeys.title")} description={t("hotkeys.modifierHint")}>
      {error && <Banner kind="error">{error}</Banner>}
      <Row label={t("hotkeys.toggle")} hint={t("hotkeys.recordHint")}>
        <HotkeyInput
          value={pending ?? hotkeys.toggle}
          placeholder={t("hotkeys.press")}
          onChange={(toggle) => update({ toggle })}
        />
      </Row>
      <Toggle
        id="enable-ptt"
        label={t("hotkeys.enablePushToTalk")}
        checked={hotkeys.push_to_talk_enabled}
        onChange={(push_to_talk_enabled) => update({ push_to_talk_enabled })}
      />
      <Row label={t("hotkeys.pushToTalk")}>
        <HotkeyInput
          value={hotkeys.push_to_talk}
          placeholder={t("hotkeys.press")}
          disabled={!hotkeys.push_to_talk_enabled}
          onChange={(push_to_talk) => update({ push_to_talk })}
        />
      </Row>
      <div className="actions">
        <Button
          onClick={() =>
            update({ toggle: DEFAULT_TOGGLE, push_to_talk: DEFAULT_PUSH_TO_TALK })
          }
        >
          {t("common.reset")}
        </Button>
      </div>
    </Card>
  );
}
