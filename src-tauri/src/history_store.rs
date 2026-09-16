//! The optional dictation history file.

use glasopis_core::history::{History, HistoryEntry};
use glasopis_core::settings::Settings;
use tauri::AppHandle;

use crate::errors::Result;
use crate::paths;

pub fn load(app: &AppHandle) -> History {
    match paths::history_file(app) {
        Ok(path) => History::load_or_default(&path),
        Err(err) => {
            log::warn!("историята не може да бъде прочетена: {err}");
            History::default()
        }
    }
}

/// Appends a transcript — but only when the user switched history on.
pub fn record(app: &AppHandle, settings: &Settings, text: &str) {
    if !settings.privacy.keep_history {
        return;
    }
    let Ok(path) = paths::history_file(app) else {
        return;
    };
    let mut history = History::load_or_default(&path);
    history.push(
        HistoryEntry {
            timestamp: now_seconds(),
            text: text.to_string(),
        },
        settings.privacy.history_limit,
    );
    if let Err(err) = history.save_to(&path) {
        log::warn!("историята не беше записана: {err}");
    }
}

pub fn clear(app: &AppHandle) -> Result<()> {
    let path = paths::history_file(app)?;
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

fn now_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
