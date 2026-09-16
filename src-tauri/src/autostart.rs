//! "Стартирай Glasopis с Windows".
//!
//! Implemented with the official Tauri autostart plugin, which writes a normal
//! per-user registry entry (`HKCU\...\Run`). No administrator rights needed.

use tauri::AppHandle;

use crate::errors::Result;

#[cfg(windows)]
pub fn set_enabled(app: &AppHandle, enabled: bool) -> Result<()> {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    let outcome = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    outcome.map_err(|err| {
        crate::errors::GlasopisError::other(format!(
            "Автоматичното стартиране не може да бъде променено: {err}"
        ))
    })
}

#[cfg(windows)]
pub fn is_enabled(app: &AppHandle) -> bool {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[cfg(not(windows))]
pub fn set_enabled(_app: &AppHandle, _enabled: bool) -> Result<()> {
    Err(crate::errors::GlasopisError::WindowsOnly)
}

#[cfg(not(windows))]
pub fn is_enabled(_app: &AppHandle) -> bool {
    false
}
