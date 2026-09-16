//! User settings: definition, defaults, (de)serialisation and migration.
//!
//! Settings are stored as JSON in the Windows roaming application data folder
//! (`%APPDATA%\Glasopis\settings.json`). Every field has a serde default, so a
//! settings file written by an older version keeps working after an update and
//! an unknown/removed field never breaks startup.

use serde::{Deserialize, Serialize};

use crate::dictionary::Dictionary;

/// Bumped whenever a migration is needed.
pub const CURRENT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InjectionMode {
    /// Clipboard first, keyboard simulation as a fallback. The default.
    #[default]
    Automatic,
    /// Always paste through the clipboard.
    Clipboard,
    /// Always type the text with simulated Unicode key events.
    Keyboard,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingMode {
    /// Press once to start, press again to stop.
    #[default]
    Toggle,
    /// Record while the push-to-talk key is held down.
    PushToTalk,
}

/// Microphone selection. `Default` follows the Windows default device.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "name", rename_all = "snake_case")]
pub enum MicrophoneChoice {
    #[default]
    Default,
    Device(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralSettings {
    pub launch_at_startup: bool,
    pub start_minimized: bool,
    pub show_floating_window: bool,
    pub play_sounds: bool,
    /// Language of the Glasopis user interface (`bg` or `en`).
    pub ui_language: String,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            launch_at_startup: false,
            start_minimized: true,
            show_floating_window: true,
            play_sounds: true,
            ui_language: "bg".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct VoiceSettings {
    pub microphone: MicrophoneChoice,
    /// Spoken language, BCP-47 style. Bulgarian is the default.
    pub language: String,
    /// Id from [`crate::models::CATALOG`]; `None` until a model is downloaded.
    pub model_id: Option<String>,
    pub auto_punctuation: bool,
    pub voice_commands: bool,
    pub capitalize_sentences: bool,
    /// Number of CPU threads for transcription; `None` = pick automatically.
    pub threads: Option<u32>,
}

impl Default for VoiceSettings {
    fn default() -> Self {
        Self {
            microphone: MicrophoneChoice::Default,
            language: "bg".to_string(),
            model_id: None,
            auto_punctuation: true,
            voice_commands: true,
            capitalize_sentences: true,
            threads: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HotkeySettings {
    /// Accelerator for start/stop recording, e.g. `Ctrl+Alt+Space`.
    pub toggle: String,
    /// Push-to-talk accelerator, e.g. `Ctrl+Alt+D`. Held down while speaking.
    /// A bare modifier such as Right Ctrl is not supported by the Windows
    /// hotkey API, so a combination is used (see docs/architecture.md).
    pub push_to_talk: String,
    pub push_to_talk_enabled: bool,
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self {
            toggle: DEFAULT_TOGGLE_HOTKEY.to_string(),
            push_to_talk: DEFAULT_PUSH_TO_TALK.to_string(),
            push_to_talk_enabled: false,
        }
    }
}

pub const DEFAULT_TOGGLE_HOTKEY: &str = "Ctrl+Alt+Space";
pub const DEFAULT_PUSH_TO_TALK: &str = "Ctrl+Alt+D";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct InjectionSettings {
    pub mode: InjectionMode,
    /// Put the previous clipboard content back after pasting.
    pub restore_clipboard: bool,
    /// How long to wait before restoring the clipboard, in milliseconds.
    pub restore_clipboard_delay_ms: u64,
}

impl Default for InjectionSettings {
    fn default() -> Self {
        Self {
            mode: InjectionMode::Automatic,
            restore_clipboard: true,
            restore_clipboard_delay_ms: 800,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PrivacySettings {
    /// Off by default: Glasopis keeps no record of what you dictate.
    pub keep_history: bool,
    /// How many entries to keep when history is enabled.
    pub history_limit: usize,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            keep_history: false,
            history_limit: 200,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub version: u32,
    pub general: GeneralSettings,
    pub voice: VoiceSettings,
    pub hotkeys: HotkeySettings,
    pub injection: InjectionSettings,
    pub privacy: PrivacySettings,
    pub dictionary: Dictionary,
    pub recording_mode: RecordingMode,
    /// Set to true once the onboarding wizard has been completed.
    pub onboarding_completed: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            general: GeneralSettings::default(),
            voice: VoiceSettings::default(),
            hotkeys: HotkeySettings::default(),
            injection: InjectionSettings::default(),
            privacy: PrivacySettings::default(),
            dictionary: Dictionary::default(),
            recording_mode: RecordingMode::default(),
            onboarding_completed: false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("настройките не могат да бъдат прочетени: {0}")]
    Io(#[from] std::io::Error),
    #[error("настройките са в невалиден формат: {0}")]
    Format(#[from] serde_json::Error),
}

impl Settings {
    /// Parses settings from JSON, applying defaults for missing fields.
    ///
    /// A file that cannot be parsed at all is *not* an error for the caller of
    /// [`Settings::load_or_default`]; the application then starts with the
    /// defaults instead of refusing to run.
    pub fn from_json(json: &str) -> Result<Self, SettingsError> {
        let mut settings: Settings = serde_json::from_str(json)?;
        settings.migrate();
        Ok(settings)
    }

    pub fn to_json(&self) -> Result<String, SettingsError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Applies any migration needed to bring an older file up to date.
    pub fn migrate(&mut self) {
        if self.version == 0 {
            // Version 0 files were written before the field existed.
            self.version = CURRENT_VERSION;
        }
        self.version = CURRENT_VERSION;
        if self.general.ui_language.trim().is_empty() {
            self.general.ui_language = "bg".into();
        }
        if self.voice.language.trim().is_empty() {
            self.voice.language = "bg".into();
        }
        if self.hotkeys.toggle.trim().is_empty() {
            self.hotkeys.toggle = DEFAULT_TOGGLE_HOTKEY.into();
        }
    }

    pub fn load_from(path: &std::path::Path) -> Result<Self, SettingsError> {
        let raw = std::fs::read_to_string(path)?;
        Self::from_json(&raw)
    }

    /// Reads the settings file, falling back to the defaults if it is missing
    /// or damaged.
    pub fn load_or_default(path: &std::path::Path) -> Self {
        match Self::load_from(path) {
            Ok(settings) => settings,
            Err(err) => {
                if path.exists() {
                    log_broken_settings(path, &err);
                }
                Settings::default()
            }
        }
    }

    /// Writes the settings atomically (write to a temporary file, then rename)
    /// so a crash during saving cannot leave a truncated file behind.
    pub fn save_to(&self, path: &std::path::Path) -> Result<(), SettingsError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, self.to_json()?)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }
}

fn log_broken_settings(path: &std::path::Path, err: &SettingsError) {
    eprintln!(
        "glasopis: неуспешно четене на {} ({err}); използвам настройки по подразбиране",
        path.display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_bulgarian_and_private() {
        let s = Settings::default();
        assert_eq!(s.general.ui_language, "bg");
        assert_eq!(s.voice.language, "bg");
        assert!(
            !s.privacy.keep_history,
            "историята трябва да е изключена по подразбиране"
        );
        assert_eq!(s.injection.mode, InjectionMode::Automatic);
        assert_eq!(s.hotkeys.toggle, "Ctrl+Alt+Space");
        assert!(s.voice.auto_punctuation);
    }

    #[test]
    fn round_trip_is_lossless() {
        let mut s = Settings::default();
        s.voice.model_id = Some("large-v3-turbo-q5_0".into());
        s.voice.microphone = MicrophoneChoice::Device("Микрофон (USB)".into());
        s.general.launch_at_startup = true;
        let json = s.to_json().unwrap();
        assert_eq!(Settings::from_json(&json).unwrap(), s);
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let s = Settings::from_json(r#"{"voice":{"model_id":"medium"}}"#).unwrap();
        assert_eq!(s.voice.model_id.as_deref(), Some("medium"));
        assert_eq!(s.voice.language, "bg");
        assert_eq!(s.hotkeys.toggle, "Ctrl+Alt+Space");
        assert_eq!(s.version, CURRENT_VERSION);
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let s = Settings::from_json(r#"{"version":1,"someFutureField":42}"#).unwrap();
        assert_eq!(s, Settings::default());
    }

    #[test]
    fn empty_hotkey_is_repaired() {
        let s = Settings::from_json(r#"{"hotkeys":{"toggle":"  "}}"#).unwrap();
        assert_eq!(s.hotkeys.toggle, DEFAULT_TOGGLE_HOTKEY);
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = std::env::temp_dir().join(format!("glasopis-test-{}", std::process::id()));
        let path = dir.join("settings.json");
        let mut s = Settings::default();
        s.general.start_minimized = false;
        s.save_to(&path).unwrap();
        assert_eq!(Settings::load_or_default(&path), s);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn damaged_file_falls_back_to_defaults() {
        let dir = std::env::temp_dir().join(format!("glasopis-broken-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        std::fs::write(&path, "{ това не е JSON").unwrap();
        assert_eq!(Settings::load_or_default(&path), Settings::default());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
