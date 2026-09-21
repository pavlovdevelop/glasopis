//! Where Glasopis keeps its files.
//!
//! * settings + history: `%APPDATA%\Glasopis\`
//! * speech models:      `%LOCALAPPDATA%\Glasopis\models\`
//! * logs:               `%LOCALAPPDATA%\Glasopis\logs\`
//!
//! Models go to the *local* application data folder because they are large and
//! must not travel with a roaming Windows profile.

use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::errors::{GlasopisError, Result};

pub fn config_dir(app: &AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| GlasopisError::other(format!("Няма достъп до папката с настройки: {e}")))?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn settings_file(app: &AppHandle) -> Result<PathBuf> {
    Ok(config_dir(app)?.join("settings.json"))
}

pub fn history_file(app: &AppHandle) -> Result<PathBuf> {
    Ok(config_dir(app)?.join("history.json"))
}

pub fn data_dir(app: &AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| GlasopisError::other(format!("Няма достъп до папката с данни: {e}")))?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn models_dir(app: &AppHandle) -> Result<PathBuf> {
    let dir = data_dir(app)?.join("models");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Written right before handing off to the updater's silent installer, so
/// the freshly-relaunched process knows to show its window even if the user
/// normally starts minimized — see `commands::mark_pending_relaunch`.
pub fn update_marker_file(app: &AppHandle) -> Result<PathBuf> {
    Ok(config_dir(app)?.join(".update-relaunch"))
}

pub fn logs_dir(app: &AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_log_dir()
        .map_err(|e| GlasopisError::other(format!("Няма достъп до папката с логове: {e}")))?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
