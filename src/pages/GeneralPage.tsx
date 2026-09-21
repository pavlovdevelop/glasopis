import { Card, Select, Toggle } from "../components/ui";
import { useAppState } from "../hooks/useAppState";
import type { Language } from "../i18n";

export function GeneralPage() {
  const { settings, save, t } = useAppState();
  if (!settings) return null;

  const general = settings.general;
  const update = (patch: Partial<typeof general>) =>
    void save({ ...settings, general: { ...general, ...patch } });

  return (
    <Card title={t("general.title")}>
      <Toggle
        id="launch-at-startup"
        label={t("general.launchAtStartup")}
        checked={general.launch_at_startup}
        onChange={(launch_at_startup) => update({ launch_at_startup })}
      />
      <Toggle
        id="start-minimized"
        label={t("general.startMinimized")}
        checked={general.start_minimized}
        onChange={(start_minimized) => update({ start_minimized })}
      />
      <Toggle
        id="floating-window"
        label={t("general.showFloatingWindow")}
        checked={general.show_floating_window}
        onChange={(show_floating_window) => update({ show_floating_window })}
      />
      <Toggle
        id="play-sounds"
        label={t("general.playSounds")}
        checked={general.play_sounds}
        onChange={(play_sounds) => update({ play_sounds })}
      />
      <Toggle
        id="check-for-updates"
        label={t("general.checkForUpdates")}
        checked={general.check_for_updates}
        onChange={(check_for_updates) => update({ check_for_updates })}
      />
      <Select<Language>
        id="ui-language"
        label={t("general.uiLanguage")}
        value={general.ui_language === "en" ? "en" : "bg"}
        options={[
          { value: "bg", label: t("general.languageBg") },
          { value: "en", label: t("general.languageEn") },
        ]}
        onChange={(ui_language) => update({ ui_language })}
      />
    </Card>
  );
}
