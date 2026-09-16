import { useEffect, useState } from "react";
import { Card, Select, Toggle } from "../components/ui";
import { useAppState } from "../hooks/useAppState";
import { api, errorMessage, type ModelStatus } from "../services/api";

const LANGUAGES = [
  { value: "bg", label: "Български (bg-BG)" },
  { value: "en", label: "English (en-US)" },
];

export function VoicePage() {
  const { settings, save, t, setError } = useAppState();
  const [models, setModels] = useState<ModelStatus[]>([]);

  useEffect(() => {
    api.listModels().then(setModels).catch((err) => setError(errorMessage(err)));
  }, [setError]);

  if (!settings) return null;
  const voice = settings.voice;
  const update = (patch: Partial<typeof voice>) =>
    void save({ ...settings, voice: { ...voice, ...patch } });

  const downloaded = models.filter((model) => model.downloaded);

  return (
    <Card title={t("voice.title")}>
      <Select
        id="speech-language"
        label={t("voice.language")}
        value={voice.language}
        options={LANGUAGES}
        onChange={(language) => update({ language })}
      />
      <Select
        id="active-model"
        label={t("voice.model")}
        value={voice.model_id ?? ""}
        options={[
          { value: "", label: t("common.notSelected") },
          ...downloaded.map((model) => ({
            value: model.id,
            label: `${model.label} — ${model.technical_name}`,
          })),
        ]}
        onChange={(model_id) => update({ model_id: model_id === "" ? null : model_id })}
      />
      <Toggle
        id="auto-punctuation"
        label={t("voice.autoPunctuation")}
        checked={voice.auto_punctuation}
        onChange={(auto_punctuation) => update({ auto_punctuation })}
      />
      <Toggle
        id="voice-commands"
        label={t("voice.voiceCommands")}
        hint={t("voice.commandsHint")}
        checked={voice.voice_commands}
        onChange={(voice_commands) => update({ voice_commands })}
      />
      <Toggle
        id="capitalize"
        label={t("voice.capitalize")}
        checked={voice.capitalize_sentences}
        onChange={(capitalize_sentences) => update({ capitalize_sentences })}
      />
      <Select
        id="threads"
        label={t("voice.threads")}
        value={voice.threads === null ? "auto" : String(voice.threads)}
        options={[
          { value: "auto", label: t("voice.threadsAuto") },
          ...[1, 2, 4, 6, 8].map((n) => ({ value: String(n), label: String(n) })),
        ]}
        onChange={(value) => update({ threads: value === "auto" ? null : Number(value) })}
      />
    </Card>
  );
}
