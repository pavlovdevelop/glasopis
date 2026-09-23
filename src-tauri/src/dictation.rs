//! The dictation flow - the heart of Glasopis.
//!
//! ```text
//! hotkey -> remember the focused window -> record -> transcribe locally
//!        -> dictionary + voice commands + normalisation
//!        -> restore focus -> insert the text
//! ```
//!
//! Everything after "record" happens on a background thread so the user
//! interface (and the rest of Windows) stays responsive.

use std::sync::atomic::Ordering;

use glasopis_core::pipeline::{process_transcript, PipelineOptions};
use glasopis_core::settings::{Settings, SpeechEngineKind};
use tauri::{AppHandle, Emitter, Manager};

use crate::errors::{GlasopisError, Result};
use crate::injection::{self, InjectionOptions, InjectionOutcome};
use crate::state::{AppState, Status, LEVEL_EVENT};
use crate::{history_store, models_manager, sounds, ui};

/// How long the "Готово" / error overlay stays on screen.
const OVERLAY_LINGER_MS: u64 = 1_200;
const OVERLAY_ERROR_LINGER_MS: u64 = 3_000;
/// Shortest buffer whisper.cpp accepts, with a little margin.
const MIN_TRANSCRIPTION_SECONDS: f32 = 1.2;

/// Starts or stops the dictation, depending on the current state.
pub fn toggle(app: &AppHandle) {
    let state = app.state::<AppState>();
    if state.recorder.is_recording() {
        stop(app);
    } else {
        start(app);
    }
}

/// Starts recording.
pub fn start(app: &AppHandle) {
    let state = app.state::<AppState>();
    if state.is_busy() {
        log::info!("предишната диктовка още се обработва");
        return;
    }
    if state.recorder.is_recording() {
        return;
    }
    // A stale "waiting for yes/no" from the assistant must not resurface
    // once an unrelated dictation starts and finishes.
    crate::assistant::cancel(app);
    let settings = state.settings();

    // Проверката е преди записа: да запишем и чак после да кажем „няма ключ“
    // означава да загубим казаното.
    if let Err((err, route)) = check_ready(app, &settings) {
        report_error(app, err);
        let _ = ui::show_main_window_at(app, route);
        return;
    }

    remember_focused_window(app);

    if let Err(err) = state.recorder.start(selected_device(&settings)) {
        report_error(app, err);
        return;
    }

    if settings.general.play_sounds {
        sounds::play_start();
    }
    state.set_status(app, Status::Listening);
    if settings.general.show_floating_window {
        ui::show_overlay(app);
    }
    spawn_level_reporter(app);
}

/// Проверява, че избраният двигател може да работи. При грешка връща и
/// страницата от настройките, която я решава.
fn check_ready(
    app: &AppHandle,
    settings: &Settings,
) -> std::result::Result<(), (GlasopisError, &'static str)> {
    match settings.voice.engine {
        SpeechEngineKind::Groq => {
            if settings.cloud.has_key() {
                Ok(())
            } else {
                Err((GlasopisError::MissingApiKey, "#/recognition"))
            }
        }
        SpeechEngineKind::Local => {
            models_manager::active_model_path(app, settings.voice.model_id.as_deref())
                .map(|_| ())
                .map_err(|err| (err, "#/models"))
        }
    }
}

/// The microphone the user selected, or `None` for the Windows default.
fn selected_device(settings: &Settings) -> Option<String> {
    match &settings.voice.microphone {
        glasopis_core::settings::MicrophoneChoice::Default => None,
        glasopis_core::settings::MicrophoneChoice::Device(name) => Some(name.clone()),
    }
}

/// Records **only** to drive the level meter in the settings window.
///
/// No model is needed, nothing is transcribed and nothing is inserted anywhere:
/// this exists so the user can check that the microphone works before (or
/// without) downloading a speech model.
pub fn start_microphone_test(app: &AppHandle) -> Result<()> {
    let state = app.state::<AppState>();
    if state.is_busy() {
        return Err(GlasopisError::other(
            "Изчакайте текущата диктовка да приключи.".to_string(),
        ));
    }
    if state.recorder.is_recording() {
        return Ok(());
    }
    let settings = state.settings();
    state.recorder.start(selected_device(&settings))?;
    spawn_level_reporter(app);
    log::info!("стартиран е тест на микрофона");
    Ok(())
}

/// Stops the microphone test and throws the audio away.
pub fn stop_microphone_test(app: &AppHandle) {
    let state = app.state::<AppState>();
    state.recorder.cancel();
}

/// Stops recording and runs the transcription.
pub fn stop(app: &AppHandle) {
    let state = app.state::<AppState>();
    if !state.recorder.is_recording() {
        return;
    }
    let settings = state.settings();
    let captured = match state.recorder.stop() {
        Ok(captured) => captured,
        Err(err) => {
            report_error(app, err);
            return;
        }
    };
    if settings.general.play_sounds {
        sounds::play_stop();
    }
    log::info!("записани са {:.1} секунди аудио", captured.seconds);
    state.busy.store(true, Ordering::SeqCst);
    state.set_status(app, Status::Processing);

    let app_handle = app.clone();
    let result = std::thread::Builder::new()
        .name("glasopis-transcribe".into())
        .spawn(move || {
            let outcome = transcribe_and_insert(&app_handle, captured.samples, &settings);
            let state = app_handle.state::<AppState>();
            state.busy.store(false, Ordering::SeqCst);
            match outcome {
                Ok((text, InjectionOutcome::Inserted)) => {
                    state.set_status(
                        &app_handle,
                        Status::Done {
                            text,
                            clipboard_only: false,
                        },
                    );
                    hide_overlay_later(&app_handle, OVERLAY_LINGER_MS);
                }
                Ok((text, InjectionOutcome::ClipboardOnly)) => {
                    state.set_status(
                        &app_handle,
                        Status::Done {
                            text,
                            clipboard_only: true,
                        },
                    );
                    hide_overlay_later(&app_handle, OVERLAY_ERROR_LINGER_MS);
                }
                Err(err) => report_error(&app_handle, err),
            }
        });

    if let Err(err) = result {
        state.busy.store(false, Ordering::SeqCst);
        report_error(
            app,
            GlasopisError::other(format!("Обработката не може да започне: {err}")),
        );
    }
}

/// Stops recording and throws the audio away.
pub fn cancel(app: &AppHandle) {
    let state = app.state::<AppState>();
    state.recorder.cancel();
    state.set_status(app, Status::Idle);
    ui::hide_overlay(app);
}

fn transcribe_and_insert(
    app: &AppHandle,
    samples: Vec<f32>,
    settings: &Settings,
) -> Result<(String, InjectionOutcome)> {
    if glasopis_core::audio::is_silence(&samples) {
        return Err(GlasopisError::other(
            "Не беше чута реч. Проверете микрофона и опитайте отново.".to_string(),
        ));
    }

    // Кратък запис се допълва с тишина: и whisper.cpp, и API-то се справят
    // по-добре с поне секунда аудио.
    let samples = glasopis_core::audio::pad_to_min_duration(
        samples,
        glasopis_core::audio::WHISPER_SAMPLE_RATE,
        MIN_TRANSCRIPTION_SECONDS,
    );

    let raw = crate::speech::transcribe_with_settings(app, settings, &samples)?;
    // Записът вече не е нужен - освобождава се възможно най-рано.
    drop(samples);

    let text = process_transcript(&raw, &PipelineOptions::from(settings));
    if text.trim().is_empty() {
        return Err(GlasopisError::other(
            "Не беше разпозната реч. Опитайте отново.".to_string(),
        ));
    }

    history_store::record(app, settings, &text);
    restore_focused_window(app);

    let outcome = injection::insert(
        &text,
        InjectionOptions {
            mode: settings.injection.mode,
            restore_clipboard: settings.injection.restore_clipboard,
            restore_delay_ms: settings.injection.restore_clipboard_delay_ms,
        },
    )?;
    Ok((text, outcome))
}

fn report_error(app: &AppHandle, err: GlasopisError) {
    log::error!("диктовката приключи с грешка: {err}");
    let state = app.state::<AppState>();
    state.set_status(
        app,
        Status::Error {
            message: err.to_string(),
        },
    );
    hide_overlay_later(app, OVERLAY_ERROR_LINGER_MS);
}

fn hide_overlay_later(app: &AppHandle, millis: u64) {
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(millis));
        let state = app.state::<AppState>();
        // Do not hide an overlay that belongs to a newer dictation.
        if !state.recorder.is_recording() && !state.is_busy() {
            ui::hide_overlay(&app);
            state.set_status(&app, Status::Idle);
        }
    });
}

/// Sends the microphone level to the overlay while recording.
fn spawn_level_reporter(app: &AppHandle) {
    let app = app.clone();
    std::thread::Builder::new()
        .name("glasopis-level".into())
        .spawn(move || {
            loop {
                let state = app.state::<AppState>();
                if !state.recorder.is_recording() {
                    break;
                }
                let _ = app.emit(LEVEL_EVENT, state.recorder.level());
                std::thread::sleep(std::time::Duration::from_millis(60));
            }
            let _ = app.emit(LEVEL_EVENT, 0.0f32);
        })
        .ok();
}

#[cfg(windows)]
fn remember_focused_window(app: &AppHandle) {
    let state = app.state::<AppState>();
    let target = crate::injection::focus::current_foreground_window();
    if let Some(target) = target {
        log::debug!("целеви прозорец: {:?}", target.title());
        *state.target_window.lock() = Some(target.0);
    } else {
        *state.target_window.lock() = None;
    }
}

#[cfg(not(windows))]
fn remember_focused_window(_app: &AppHandle) {}

#[cfg(windows)]
fn restore_focused_window(app: &AppHandle) {
    let state = app.state::<AppState>();
    let handle = *state.target_window.lock();
    if let Some(handle) = handle {
        let target = crate::injection::focus::TargetWindow(handle);
        if !target.restore_focus() {
            log::warn!("фокусът не беше върнат към целевия прозорец");
        }
        // Give the target a moment to actually become foreground.
        std::thread::sleep(std::time::Duration::from_millis(60));
    }
}

#[cfg(not(windows))]
fn restore_focused_window(_app: &AppHandle) {}
