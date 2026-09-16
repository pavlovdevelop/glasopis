//! The commands the user interface can call.
//!
//! Every command returns a Bulgarian error message when it fails, so the
//! frontend can show it without translating anything.

use glasopis_core::history::History;
use glasopis_core::models;
use glasopis_core::settings::Settings;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::audio::{list_input_devices, InputDevice};
use crate::errors::{GlasopisError, Result};
use crate::models_manager::{self, ModelStatus};
use crate::state::{AppState, Status};
use crate::{autostart, dictation, history_store, hotkeys, paths, settings_store, tray, ui};

#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub version: String,
    /// False only in builds made without the `whisper` feature.
    pub speech_available: bool,
    pub models_dir: String,
    pub config_dir: String,
    pub logs_dir: String,
    pub autostart_enabled: bool,
    /// True while a speech model is loaded in memory.
    pub model_loaded: bool,
    /// Дали тази компилация носи локално разпознаване (whisper.cpp).
    pub local_engine_available: bool,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings()
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Settings> {
    let mut settings = settings;
    settings.migrate();

    if !hotkeys::is_valid_accelerator(&settings.hotkeys.toggle) {
        return Err(GlasopisError::other(format!(
            "Комбинацията „{}“ не е валидна.",
            settings.hotkeys.toggle
        )));
    }
    if settings.hotkeys.push_to_talk_enabled
        && !hotkeys::is_valid_accelerator(&settings.hotkeys.push_to_talk)
    {
        return Err(GlasopisError::other(format!(
            "Комбинацията „{}“ не е валидна.",
            settings.hotkeys.push_to_talk
        )));
    }

    let previous = state.settings();
    if previous.general.launch_at_startup != settings.general.launch_at_startup {
        autostart::set_enabled(&app, settings.general.launch_at_startup)?;
        let _ = tray::refresh(&app);
    }

    state.replace_settings(settings.clone());
    settings_store::save(&app, &settings)?;

    if previous.hotkeys != settings.hotkeys {
        hotkeys::register_all(&app, &settings)?;
    }
    Ok(settings)
}

#[tauri::command]
pub fn list_microphones() -> Result<Vec<InputDevice>> {
    list_input_devices()
}

#[tauri::command]
pub fn list_models(app: AppHandle) -> Result<Vec<ModelStatus>> {
    models_manager::list(&app)
}

#[tauri::command]
pub fn download_model(app: AppHandle, id: String) -> Result<()> {
    models_manager::download(&app, &id)
}

#[tauri::command]
pub fn cancel_model_download(app: AppHandle, id: String) {
    models_manager::cancel(&app, &id);
}

#[tauri::command]
pub fn delete_model(app: AppHandle, id: String) -> Result<()> {
    models_manager::delete(&app, &id)
}

/// Makes `id` the active model (it must already be downloaded).
#[tauri::command]
pub fn select_model(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<Settings> {
    let info = models::find(&id).ok_or(GlasopisError::NoModelSelected)?;
    let path = models_manager::model_path(&app, info)?;
    if !path.is_file() {
        return Err(GlasopisError::ModelMissingOrCorrupted);
    }
    let mut settings = state.settings();
    settings.voice.model_id = Some(id);
    state.replace_settings(settings.clone());
    settings_store::save(&app, &settings)?;
    Ok(settings)
}

#[tauri::command]
pub fn start_dictation(app: AppHandle) {
    dictation::start(&app);
}

#[tauri::command]
pub fn stop_dictation(app: AppHandle) {
    dictation::stop(&app);
}

#[tauri::command]
pub fn toggle_dictation(app: AppHandle) {
    dictation::toggle(&app);
}

#[tauri::command]
pub fn cancel_dictation(app: AppHandle) {
    dictation::cancel(&app);
}

/// Проверява дали въведеният ключ за Groq работи.
#[tauri::command]
pub fn check_api_key(state: State<'_, AppState>) -> Result<()> {
    let settings = state.settings();
    crate::speech::groq::check_api_key(&settings.cloud.api_key, &settings.cloud.model)
}

/// Records only for the level meter — no model and no text insertion.
#[tauri::command]
pub fn start_microphone_test(app: AppHandle) -> Result<()> {
    dictation::start_microphone_test(&app)
}

#[tauri::command]
pub fn stop_microphone_test(app: AppHandle) {
    dictation::stop_microphone_test(&app);
}

#[tauri::command]
pub fn get_status(state: State<'_, AppState>) -> Status {
    state.status()
}

#[tauri::command]
pub fn get_history(app: AppHandle) -> History {
    history_store::load(&app)
}

#[tauri::command]
pub fn clear_history(app: AppHandle) -> Result<()> {
    history_store::clear(&app)
}

#[tauri::command]
pub fn validate_hotkey(accelerator: String) -> bool {
    hotkeys::is_valid_accelerator(&accelerator)
}

#[tauri::command]
pub fn complete_onboarding(app: AppHandle, state: State<'_, AppState>) -> Result<Settings> {
    let mut settings = state.settings();
    settings.onboarding_completed = true;
    state.replace_settings(settings.clone());
    settings_store::save(&app, &settings)?;
    Ok(settings)
}

#[tauri::command]
pub fn get_app_info(app: AppHandle) -> Result<AppInfo> {
    let state = app.state::<AppState>();
    Ok(AppInfo {
        version: app.package_info().version.to_string(),
        speech_available: cfg!(feature = "whisper"),
        models_dir: paths::models_dir(&app)?.to_string_lossy().to_string(),
        config_dir: paths::config_dir(&app)?.to_string_lossy().to_string(),
        logs_dir: paths::logs_dir(&app)?.to_string_lossy().to_string(),
        autostart_enabled: autostart::is_enabled(&app),
        model_loaded: state.engine.is_loaded(),
        local_engine_available: cfg!(feature = "whisper"),
    })
}

#[tauri::command]
pub fn open_folder(app: AppHandle, which: String) -> Result<()> {
    use tauri_plugin_opener::OpenerExt;
    let dir = match which.as_str() {
        "models" => paths::models_dir(&app)?,
        "logs" => paths::logs_dir(&app)?,
        "config" => paths::config_dir(&app)?,
        other => return Err(GlasopisError::other(format!("Непозната папка: {other}"))),
    };
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|e| GlasopisError::other(format!("Папката не беше отворена: {e}")))
}

/// Отваря адрес в браузъра. Позволени са само адресите, които самият
/// интерфейс показва — командата не е общ „отвори каквото ти кажат“.
#[tauri::command]
pub fn open_url(app: AppHandle, url: String) -> Result<()> {
    const ALLOWED: &[&str] = &[
        "https://console.groq.com/keys",
        "https://github.com/pavlovdevelop/glasopis",
    ];
    if !ALLOWED.contains(&url.as_str()) {
        return Err(GlasopisError::other(format!("Непозволен адрес: {url}")));
    }
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| GlasopisError::other(format!("Адресът не беше отворен: {e}")))
}

#[tauri::command]
pub fn hide_overlay(app: AppHandle) {
    ui::hide_overlay(&app);
}

/// Hides the settings window to the tray instead of closing the application.
#[tauri::command]
pub fn hide_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window(ui::MAIN_WINDOW) {
        let _ = window.hide();
    }
}

#[tauri::command]
pub fn quit_app(app: AppHandle, state: State<'_, AppState>) {
    state.recorder.cancel();
    app.exit(0);
}
