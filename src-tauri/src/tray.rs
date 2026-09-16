//! The Windows system tray icon and its menu.

use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use crate::errors::{GlasopisError, Result};
use crate::state::AppState;
use crate::{autostart, dictation, paths, ui};

pub const TRAY_ID: &str = "glasopis-tray";

pub fn create(app: &AppHandle) -> Result<()> {
    let dictate = MenuItemBuilder::with_id("dictate", "🎙  Започни диктовка").build(app)?;
    let settings = MenuItemBuilder::with_id("settings", "⚙  Настройки").build(app)?;
    let models = MenuItemBuilder::with_id("models", "📥  Езикови модели").build(app)?;
    let microphone = MenuItemBuilder::with_id("microphone", "🎤  Микрофон").build(app)?;
    let startup_label = if autostart::is_enabled(app) {
        "🚀  Стартирай с Windows: включено"
    } else {
        "🚀  Стартирай с Windows: изключено"
    };
    let startup = MenuItemBuilder::with_id("startup", startup_label).build(app)?;
    let logs = MenuItemBuilder::with_id("logs", "🗒  Отвори папката с логове").build(app)?;
    let about = MenuItemBuilder::with_id("about", "ℹ  За Glasopis").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "❌  Изход").build(app)?;

    let menu = MenuBuilder::new(app)
        .items(&[&dictate])
        .separator()
        .items(&[&settings, &models, &microphone, &startup])
        .separator()
        .items(&[&logs, &about])
        .separator()
        .items(&[&quit])
        .build()?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| GlasopisError::other("Липсва икона на приложението.".to_string()))?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("Glasopis — Говориш. То пише.")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(|tray, event| {
            // Double click opens the settings window.
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                let _ = ui::show_main_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "dictate" => dictation::toggle(app),
        "settings" => {
            let _ = ui::show_main_window_at(app, "#/settings");
        }
        "models" => {
            let _ = ui::show_main_window_at(app, "#/models");
        }
        "microphone" => {
            let _ = ui::show_main_window_at(app, "#/microphone");
        }
        "startup" => toggle_autostart(app),
        "logs" => open_logs_folder(app),
        "about" => {
            let _ = ui::show_main_window_at(app, "#/about");
        }
        "quit" => {
            let state = app.state::<AppState>();
            state.recorder.cancel();
            app.exit(0);
        }
        other => log::warn!("непознат елемент от менюто: {other}"),
    }
}

fn toggle_autostart(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut settings = state.settings();
    let next = !autostart::is_enabled(app);
    match autostart::set_enabled(app, next) {
        Ok(()) => {
            settings.general.launch_at_startup = next;
            state.replace_settings(settings.clone());
            let _ = crate::settings_store::save(app, &settings);
            // Rebuild the menu so the label matches the new state.
            if let Err(err) = refresh(app) {
                log::error!("менюто в трея не беше обновено: {err}");
            }
        }
        Err(err) => log::error!("{err}"),
    }
}

/// Rebuilds the tray menu (after the autostart setting changed).
pub fn refresh(app: &AppHandle) -> Result<()> {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_visible(false).ok();
        app.remove_tray_by_id(TRAY_ID);
    }
    create(app)
}

fn open_logs_folder(app: &AppHandle) {
    match paths::logs_dir(app) {
        Ok(dir) => {
            use tauri_plugin_opener::OpenerExt;
            if let Err(err) = app.opener().open_path(dir.to_string_lossy(), None::<&str>) {
                log::error!("папката с логове не беше отворена: {err}");
            }
        }
        Err(err) => log::error!("{err}"),
    }
}
