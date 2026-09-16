import { Card, NumberField, Select, Toggle } from "../components/ui";
import { useAppState } from "../hooks/useAppState";
import type { InjectionMode } from "../services/api";

export function InsertionPage() {
  const { settings, save, t } = useAppState();
  if (!settings) return null;

  const injection = settings.injection;
  const update = (patch: Partial<typeof injection>) =>
    void save({ ...settings, injection: { ...injection, ...patch } });

  return (
    <Card title={t("insertion.title")} description={t("insertion.explanation")}>
      <Select<InjectionMode>
        id="injection-mode"
        label={t("insertion.mode")}
        value={injection.mode}
        options={[
          { value: "automatic", label: t("insertion.automatic") },
          { value: "clipboard", label: t("insertion.clipboard") },
          { value: "keyboard", label: t("insertion.keyboard") },
        ]}
        onChange={(mode) => update({ mode })}
      />
      <Toggle
        id="restore-clipboard"
        label={t("insertion.restoreClipboard")}
        checked={injection.restore_clipboard}
        onChange={(restore_clipboard) => update({ restore_clipboard })}
      />
      <NumberField
        id="restore-delay"
        label={t("insertion.delay")}
        value={injection.restore_clipboard_delay_ms}
        min={100}
        max={5000}
        onChange={(restore_clipboard_delay_ms) => update({ restore_clipboard_delay_ms })}
      />
    </Card>
  );
}
