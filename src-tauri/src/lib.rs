//! Glasopis — системно гласово въвеждане на български за Windows.
//!
//! Ти говориш — Аз пиша.

mod audio;
mod autostart;
mod commands;
mod dictation;
mod errors;
mod history_store;
mod hotkeys;
mod injection;
mod models_manager;
mod paths;
mod settings_store;
mod sounds;
mod speech;
mod state;
mod tray;
mod ui;

use tauri::{Manager, WindowEvent};

use crate::state::AppState;

/// Starts the application. Called from `main.rs`.
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(windows)]
    {
        // A second launch focuses the running instance instead of starting a
        // second tray icon and a second speech engine.
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            log::info!("вече има стартиран Glasopis — показвам съществуващия прозорец");
            let _ = ui::show_main_window(app);
        }));
        builder = builder.plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ));
    }

    builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("glasopis".into()),
                    }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                ])
                .max_file_size(2 * 1024 * 1024)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::list_microphones,
            commands::list_models,
            commands::download_model,
            commands::cancel_model_download,
            commands::delete_model,
            commands::select_model,
            commands::start_dictation,
            commands::stop_dictation,
            commands::toggle_dictation,
            commands::cancel_dictation,
            commands::check_api_key,
            commands::list_cloud_models,
            commands::start_microphone_test,
            commands::stop_microphone_test,
            commands::get_status,
            commands::get_history,
            commands::clear_history,
            commands::validate_hotkey,
            commands::complete_onboarding,
            commands::get_app_info,
            commands::open_folder,
            commands::open_url,
            commands::hide_overlay,
            commands::save_overlay_position,
            commands::mark_pending_relaunch,
            commands::take_update_notice,
            commands::hide_main_window,
            commands::quit_app,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let settings = settings_store::load(&handle);
            log::info!(
                "Glasopis стартира (език на интерфейса: {})",
                settings.general.ui_language
            );

            app.manage(AppState::new(settings.clone()));

            if let Err(err) = tray::create(&handle) {
                log::error!("иконата в системния трей не беше създадена: {err}");
            }
            if let Err(err) = hotkeys::register_all(&handle, &settings) {
                log::error!("{err}");
            }
            if let Err(err) = ui::ensure_overlay(&handle) {
                log::error!("{err}");
            }

            // A relaunch straight after the updater's silent install shows the
            // window once, even if the user normally starts minimized —
            // otherwise the update looks like it silently did nothing.
            let update_marker = paths::update_marker_file(&handle).ok();
            let just_updated = update_marker.as_ref().is_some_and(|path| path.exists());
            if let Some(path) = &update_marker {
                let _ = std::fs::remove_file(path);
            }
            if just_updated {
                let version = app.package_info().version.to_string();
                app.state::<AppState>().set_just_updated(version);
            }

            // First run (or a run started by hand) opens the window; a start
            // with Windows goes straight to the tray.
            let started_minimized = std::env::args().any(|arg| arg == "--minimized");
            let show_window = !settings.onboarding_completed
                || just_updated
                || (!settings.general.start_minimized && !started_minimized);
            if show_window {
                let route = if settings.onboarding_completed {
                    "#/settings"
                } else {
                    "#/onboarding"
                };
                if let Err(err) = ui::show_main_window_at(&handle, route) {
                    log::error!("{err}");
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the settings window keeps Glasopis running in the tray.
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == ui::MAIN_WINDOW {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("Glasopis не може да бъде стартиран")
        .run(|_app, event| {
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                // Keep running in the tray unless the exit was explicit.
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
