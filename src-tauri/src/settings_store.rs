//! Loading and saving the settings file.

use glasopis_core::settings::Settings;
use tauri::AppHandle;

use crate::errors::Result;
use crate::paths;

pub fn load(app: &AppHandle) -> Settings {
    match paths::settings_file(app) {
        Ok(path) => Settings::load_or_default(&path),
        Err(err) => {
            log::error!("настройките не могат да бъдат прочетени: {err}");
            Settings::default()
        }
    }
}

pub fn save(app: &AppHandle, settings: &Settings) -> Result<()> {
    let path = paths::settings_file(app)?;
    settings
        .save_to(&path)
        .map_err(|e| crate::errors::GlasopisError::other(e.to_string()))?;
    log::info!("настройките са записани в {}", path.display());
    Ok(())
}
