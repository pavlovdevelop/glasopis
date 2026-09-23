//! Shared application state.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use glasopis_core::settings::Settings;
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::audio::Recorder;
use crate::speech::EngineHolder;

/// Event name used for every status update sent to the windows.
pub const STATUS_EVENT: &str = "glasopis://status";
/// Event name for the microphone level while recording.
pub const LEVEL_EVENT: &str = "glasopis://level";
/// Event name for model download progress.
pub const MODEL_EVENT: &str = "glasopis://model";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Status {
    /// Nothing is happening.
    Idle,
    /// Recording ("Слушам...").
    Listening,
    /// Transcribing ("Обработвам...").
    Processing,
    /// The assistant proposes `question` and waits for a yes/no reply.
    Confirming { question: String },
    /// Finished; `text` is what was inserted.
    Done { text: String, clipboard_only: bool },
    /// Something went wrong; `message` is shown to the user in Bulgarian.
    Error { message: String },
}

pub struct AppState {
    settings: Mutex<Settings>,
    pub recorder: Recorder,
    pub engine: Arc<EngineHolder>,
    status: Mutex<Status>,
    /// True while a transcription is running, so a second hotkey press cannot
    /// start a new dictation in the middle of one.
    pub busy: AtomicBool,
    /// The window that had focus when the dictation started.
    pub target_window: Mutex<Option<isize>>,
    /// Download cancellation flags, keyed by model id.
    pub cancel_flags: Mutex<std::collections::HashMap<String, Arc<AtomicBool>>>,
    /// Set once in `setup()` when this launch follows an updater-driven
    /// relaunch; the frontend reads it (via `take_update_notice`) to show a
    /// one-time "updated to vX" banner, and reading it clears it.
    just_updated: Mutex<Option<String>>,
    /// Where the voice assistant is in its command -> confirm -> execute
    /// flow. `None` means it is idle (any active recording belongs to an
    /// ordinary dictation instead - the two never run at once).
    assistant_phase: Mutex<Option<crate::assistant::AssistantPhase>>,
    /// The overlay's position just before `ui::expand_overlay_for_question`
    /// grew it (and possibly shifted it, to keep the wider box on screen), so
    /// `ui::shrink_overlay` can put it back exactly rather than leaving it
    /// wherever the edge-clamp moved it to.
    overlay_pre_expand_position: Mutex<Option<(f64, f64)>>,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings: Mutex::new(settings),
            recorder: Recorder::new(),
            engine: Arc::new(EngineHolder::new()),
            status: Mutex::new(Status::Idle),
            busy: AtomicBool::new(false),
            target_window: Mutex::new(None),
            cancel_flags: Mutex::new(std::collections::HashMap::new()),
            just_updated: Mutex::new(None),
            assistant_phase: Mutex::new(None),
            overlay_pre_expand_position: Mutex::new(None),
        }
    }

    pub fn assistant_phase(&self) -> Option<crate::assistant::AssistantPhase> {
        self.assistant_phase.lock().clone()
    }

    pub fn set_overlay_pre_expand_position(&self, x: f64, y: f64) {
        *self.overlay_pre_expand_position.lock() = Some((x, y));
    }

    pub fn take_overlay_pre_expand_position(&self) -> Option<(f64, f64)> {
        self.overlay_pre_expand_position.lock().take()
    }

    pub fn set_assistant_phase(&self, phase: Option<crate::assistant::AssistantPhase>) {
        *self.assistant_phase.lock() = phase;
    }

    pub fn set_just_updated(&self, version: String) {
        *self.just_updated.lock() = Some(version);
    }

    pub fn take_update_notice(&self) -> Option<String> {
        self.just_updated.lock().take()
    }

    pub fn settings(&self) -> Settings {
        self.settings.lock().clone()
    }

    pub fn replace_settings(&self, settings: Settings) {
        let previous = self.settings.lock().clone();
        // A different model means the loaded one must be released.
        if previous.voice.model_id != settings.voice.model_id
            || previous.voice.threads != settings.voice.threads
        {
            self.engine.unload();
        }
        *self.settings.lock() = settings;
    }

    pub fn status(&self) -> Status {
        self.status.lock().clone()
    }

    pub fn set_status(&self, app: &AppHandle, status: Status) {
        *self.status.lock() = status.clone();
        if let Err(err) = app.emit(STATUS_EVENT, &status) {
            log::warn!("статусът не беше изпратен към интерфейса: {err}");
        }
    }

    pub fn is_busy(&self) -> bool {
        self.busy.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_serializes_with_a_state_tag() {
        let json = serde_json::to_string(&Status::Listening).unwrap();
        assert_eq!(json, r#"{"state":"listening"}"#);

        let json = serde_json::to_string(&Status::Error {
            message: "Не е открит микрофон.".into(),
        })
        .unwrap();
        assert!(json.contains("\"state\":\"error\""));
        assert!(json.contains("микрофон"));
    }

    #[test]
    fn changing_the_model_unloads_the_engine() {
        let state = AppState::new(Settings::default());
        let mut next = Settings::default();
        next.voice.model_id = Some("medium".into());
        state.replace_settings(next.clone());
        assert!(!state.engine.is_loaded());
        assert_eq!(state.settings().voice.model_id, next.voice.model_id);
    }
}
