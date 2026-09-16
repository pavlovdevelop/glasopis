/**
 * Тънък слой над Tauri командите на Glasopis.
 * Типовете тук отговарят на структурите в `src-tauri`.
 */
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type InjectionMode = "automatic" | "clipboard" | "keyboard";
export type RecordingMode = "toggle" | "push_to_talk";

export type MicrophoneChoice = { kind: "default" } | { kind: "device"; name: string };

export type SpeechEngineKind = "groq" | "local";

export interface DictionaryEntry {
  spoken: string;
  written: string;
}

export interface Settings {
  version: number;
  general: {
    launch_at_startup: boolean;
    start_minimized: boolean;
    show_floating_window: boolean;
    play_sounds: boolean;
    ui_language: string;
  };
  voice: {
    engine: SpeechEngineKind;
    microphone: MicrophoneChoice;
    language: string;
    model_id: string | null;
    auto_punctuation: boolean;
    voice_commands: boolean;
    capitalize_sentences: boolean;
    threads: number | null;
  };
  hotkeys: {
    toggle: string;
    push_to_talk: string;
    push_to_talk_enabled: boolean;
  };
  injection: {
    mode: InjectionMode;
    restore_clipboard: boolean;
    restore_clipboard_delay_ms: number;
  };
  privacy: {
    keep_history: boolean;
    history_limit: number;
  };
  cloud: {
    api_key: string;
    model: string;
    terms: string;
  };
  dictionary: { entries: DictionaryEntry[] };
  recording_mode: RecordingMode;
  onboarding_completed: boolean;
}

export type Status =
  | { state: "idle" }
  | { state: "listening" }
  | { state: "processing" }
  | { state: "done"; text: string; clipboard_only: boolean }
  | { state: "error"; message: string };

export interface InputDevice {
  name: string;
  is_default: boolean;
}

export interface ModelStatus {
  id: string;
  label: string;
  technical_name: string;
  size_bytes: number;
  size_mb: number;
  ram_mb: number;
  recommended: boolean;
  downloaded: boolean;
  active: boolean;
  path: string;
}

export type ModelEvent =
  | { state: "downloading"; id: string; downloaded: number; total: number; percent: number }
  | { state: "verifying"; id: string }
  | { state: "ready"; id: string }
  | { state: "cancelled"; id: string }
  | { state: "failed"; id: string; message: string }
  | { state: "deleted"; id: string };

export interface CloudModelInfo {
  id: string;
  word_error_rate: number;
  fast: boolean;
  default: boolean;
}

export interface HistoryEntry {
  timestamp: number;
  text: string;
}

export interface AppInfo {
  version: string;
  speech_available: boolean;
  models_dir: string;
  config_dir: string;
  logs_dir: string;
  autostart_enabled: boolean;
  model_loaded: boolean;
  local_engine_available: boolean;
}

export const api = {
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  listMicrophones: () => invoke<InputDevice[]>("list_microphones"),
  listModels: () => invoke<ModelStatus[]>("list_models"),
  downloadModel: (id: string) => invoke<void>("download_model", { id }),
  cancelModelDownload: (id: string) => invoke<void>("cancel_model_download", { id }),
  deleteModel: (id: string) => invoke<void>("delete_model", { id }),
  selectModel: (id: string) => invoke<Settings>("select_model", { id }),
  startDictation: () => invoke<void>("start_dictation"),
  stopDictation: () => invoke<void>("stop_dictation"),
  toggleDictation: () => invoke<void>("toggle_dictation"),
  cancelDictation: () => invoke<void>("cancel_dictation"),
  startMicrophoneTest: () => invoke<void>("start_microphone_test"),
  stopMicrophoneTest: () => invoke<void>("stop_microphone_test"),
  getStatus: () => invoke<Status>("get_status"),
  getHistory: () => invoke<{ entries: HistoryEntry[] }>("get_history"),
  clearHistory: () => invoke<void>("clear_history"),
  checkApiKey: () => invoke<void>("check_api_key"),
  listCloudModels: () => invoke<CloudModelInfo[]>("list_cloud_models"),
  validateHotkey: (accelerator: string) => invoke<boolean>("validate_hotkey", { accelerator }),
  completeOnboarding: () => invoke<Settings>("complete_onboarding"),
  getAppInfo: () => invoke<AppInfo>("get_app_info"),
  openFolder: (which: "models" | "logs" | "config") => invoke<void>("open_folder", { which }),
  openUrl: (url: string) => invoke<void>("open_url", { url }),
  hideOverlay: () => invoke<void>("hide_overlay"),
  hideMainWindow: () => invoke<void>("hide_main_window"),
  quitApp: () => invoke<void>("quit_app"),
};

export const events = {
  onStatus: (handler: (status: Status) => void): Promise<UnlistenFn> =>
    listen<Status>("glasopis://status", (event) => handler(event.payload)),
  onLevel: (handler: (level: number) => void): Promise<UnlistenFn> =>
    listen<number>("glasopis://level", (event) => handler(event.payload)),
  onModel: (handler: (event: ModelEvent) => void): Promise<UnlistenFn> =>
    listen<ModelEvent>("glasopis://model", (event) => handler(event.payload)),
};

/** Грешките от бекенда идват като текст на български. */
export function errorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return String(error);
}

/** Форматира байтове като „547 MB“. */
export function formatBytes(bytes: number): string {
  const mb = bytes / (1024 * 1024);
  if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
  return `${Math.round(mb)} MB`;
}
