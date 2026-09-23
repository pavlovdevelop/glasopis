//! The voice assistant: command -> AI interpretation -> spoken confirmation
//! -> whitelisted action. A separate opt-in mode from ordinary dictation (see
//! `glasopis_core::assistant` for the whitelist and the safety rules).
//!
//! ```text
//! hotkey -> record command -> transcribe -> AI picks a whitelisted action
//!        -> ask "Да отворя ли Chrome?" (spoken + shown)
//! hotkey -> record yes/no -> transcribe -> classify
//!        -> "да": execute            -> "не"/unclear: cancel, nothing runs
//! ```
//!
//! Shares the dictation flow's recorder and speech engine (`state.recorder`,
//! `speech::transcribe_with_settings`) - the two modes never run at once.

use tauri::{AppHandle, Manager};

use glasopis_core::assistant::{self as core_assistant, Confirmation, ProposedAction};
use glasopis_core::settings::Settings;

use crate::errors::{GlasopisError, Result};
use crate::state::{AppState, Status};
use crate::{sounds, ui};

mod actions;

/// The current step of the flow, kept in [`AppState`] between hotkey presses.
#[derive(Debug, Clone)]
pub enum AssistantPhase {
    /// Recording the spoken command.
    ListeningCommand,
    /// The AI proposed `action`; waiting for the next hotkey press to record
    /// the yes/no reply.
    AwaitingConfirmation(ProposedAction),
    /// Recording the yes/no reply to `action`.
    ListeningConfirmation(ProposedAction),
}

const OVERLAY_LINGER_MS: u64 = 1_800;
const OVERLAY_ERROR_LINGER_MS: u64 = 3_000;
const MIN_TRANSCRIPTION_SECONDS: f32 = 1.0;

/// The single entry point, bound to `settings.assistant.hotkey`. Each press
/// either starts a recording or, if one is already running, stops it and
/// moves the flow to its next step.
pub fn toggle(app: &AppHandle) {
    let state = app.state::<AppState>();
    if !state.settings().assistant.enabled {
        return;
    }

    if state.recorder.is_recording() {
        // The recorder is already running for *some* phase - stop it and
        // process whichever phase started it.
        match state.assistant_phase() {
            Some(AssistantPhase::ListeningCommand) => stop_listening_command(app),
            Some(AssistantPhase::ListeningConfirmation(action)) => {
                stop_listening_confirmation(app, action)
            }
            _ => {}
        }
        return;
    }

    match state.assistant_phase() {
        None => {
            if state.is_busy() {
                return;
            }
            start_listening_command(app);
        }
        Some(AssistantPhase::AwaitingConfirmation(action)) => {
            start_listening_confirmation(app, action);
        }
        // A recording should have been running; nothing sane to do but reset.
        Some(_) => state.set_assistant_phase(None),
    }
}

/// Cancels whatever the assistant is doing and returns to idle. Used when the
/// user starts an ordinary dictation instead of finishing the assistant flow.
pub fn cancel(app: &AppHandle) {
    let state = app.state::<AppState>();
    if state.recorder.is_recording() {
        state.recorder.cancel();
    }
    state.set_assistant_phase(None);
    ui::shrink_overlay(app);
}

fn start_listening_command(app: &AppHandle) {
    let state = app.state::<AppState>();
    let settings = state.settings();
    if let Err(err) = ready_check(&settings) {
        report(app, err);
        return;
    }
    if let Err(err) = state.recorder.start(selected_device(&settings)) {
        report(app, err);
        return;
    }
    if settings.general.play_sounds {
        sounds::play_start();
    }
    state.set_assistant_phase(Some(AssistantPhase::ListeningCommand));
    state.set_status(app, Status::Listening);
    ui::show_overlay(app);
}

fn stop_listening_command(app: &AppHandle) {
    let state = app.state::<AppState>();
    let settings = state.settings();
    let captured = match state.recorder.stop() {
        Ok(captured) => captured,
        Err(err) => {
            state.set_assistant_phase(None);
            report(app, err);
            return;
        }
    };
    if settings.general.play_sounds {
        sounds::play_stop();
    }
    state.busy.store(true, std::sync::atomic::Ordering::SeqCst);
    state.set_status(app, Status::Processing);

    let app_handle = app.clone();
    let spawned = std::thread::Builder::new()
        .name("glasopis-assistant-command".into())
        .spawn(move || {
            let outcome = interpret(&app_handle, &settings, captured.samples);
            let state = app_handle.state::<AppState>();
            state.busy.store(false, std::sync::atomic::Ordering::SeqCst);
            match outcome {
                Ok(action @ ProposedAction::OpenApp { .. }) => {
                    let question = match &action {
                        ProposedAction::OpenApp { question, .. } => question.clone(),
                        ProposedAction::Unknown => unreachable!(),
                    };
                    speak_if_enabled(&settings, &question);
                    state.set_assistant_phase(Some(AssistantPhase::AwaitingConfirmation(action)));
                    state.set_status(&app_handle, Status::Confirming { question });
                    ui::expand_overlay_for_question(&app_handle);
                }
                Ok(ProposedAction::Unknown) => {
                    let message = "Не разбрах командата.".to_string();
                    speak_if_enabled(&settings, &message);
                    state.set_assistant_phase(None);
                    report_message(&app_handle, message);
                }
                Err(err) => {
                    state.set_assistant_phase(None);
                    report(&app_handle, err);
                }
            }
        });
    if spawned.is_err() {
        state.busy.store(false, std::sync::atomic::Ordering::SeqCst);
        state.set_assistant_phase(None);
        report(
            app,
            GlasopisError::other("Обработката на командата не може да започне.".to_string()),
        );
    }
}

fn start_listening_confirmation(app: &AppHandle, action: ProposedAction) {
    let state = app.state::<AppState>();
    let settings = state.settings();
    ui::shrink_overlay(app);
    if let Err(err) = state.recorder.start(selected_device(&settings)) {
        state.set_assistant_phase(None);
        report(app, err);
        return;
    }
    if settings.general.play_sounds {
        sounds::play_start();
    }
    state.set_assistant_phase(Some(AssistantPhase::ListeningConfirmation(action)));
    state.set_status(app, Status::Listening);
}

fn stop_listening_confirmation(app: &AppHandle, action: ProposedAction) {
    let state = app.state::<AppState>();
    let settings = state.settings();
    let captured = match state.recorder.stop() {
        Ok(captured) => captured,
        Err(err) => {
            state.set_assistant_phase(None);
            report(app, err);
            return;
        }
    };
    if settings.general.play_sounds {
        sounds::play_stop();
    }
    state.busy.store(true, std::sync::atomic::Ordering::SeqCst);
    state.set_status(app, Status::Processing);

    let app_handle = app.clone();
    let spawned = std::thread::Builder::new()
        .name("glasopis-assistant-confirm".into())
        .spawn(move || {
            let confirmation = transcribe_confirmation(&app_handle, &settings, captured.samples);
            let state = app_handle.state::<AppState>();
            state.busy.store(false, std::sync::atomic::Ordering::SeqCst);
            state.set_assistant_phase(None);

            match confirmation {
                Ok(Confirmation::Yes) => execute(&app_handle, &settings, action),
                Ok(Confirmation::No) | Ok(Confirmation::Unclear) => {
                    let message = "Добре, отказано.".to_string();
                    speak_if_enabled(&settings, &message);
                    report_message(&app_handle, message);
                }
                Err(err) => report(&app_handle, err),
            }
        });
    if spawned.is_err() {
        state.busy.store(false, std::sync::atomic::Ordering::SeqCst);
        state.set_assistant_phase(None);
        report(
            app,
            GlasopisError::other("Потвърждението не може да се обработи.".to_string()),
        );
    }
}

fn execute(app: &AppHandle, settings: &Settings, action: ProposedAction) {
    let result = match &action {
        ProposedAction::OpenApp { app: known, .. } => actions::open_app(known),
        ProposedAction::Unknown => Err(GlasopisError::other("Няма какво да изпълня.".to_string())),
    };
    match result {
        Ok(message) => {
            speak_if_enabled(settings, &message);
            report_message(app, message);
        }
        Err(err) => report(app, err),
    }
}

fn interpret(app: &AppHandle, settings: &Settings, samples: Vec<f32>) -> Result<ProposedAction> {
    if glasopis_core::audio::is_silence(&samples) {
        return Err(GlasopisError::other(
            "Не беше чута реч. Опитайте отново.".to_string(),
        ));
    }
    let samples = glasopis_core::audio::pad_to_min_duration(
        samples,
        glasopis_core::audio::WHISPER_SAMPLE_RATE,
        MIN_TRANSCRIPTION_SECONDS,
    );
    let text = crate::speech::transcribe_with_settings(app, settings, &samples)?;
    drop(samples);
    if text.trim().is_empty() {
        return Err(GlasopisError::other(
            "Не беше разпозната реч. Опитайте отново.".to_string(),
        ));
    }
    log::info!("асистент: команда \"{text}\"");

    let raw = crate::speech::groq::interpret_command(&settings.cloud.api_key, &text)?;
    Ok(core_assistant::parse_model_response(&raw))
}

fn transcribe_confirmation(
    app: &AppHandle,
    settings: &Settings,
    samples: Vec<f32>,
) -> Result<Confirmation> {
    if glasopis_core::audio::is_silence(&samples) {
        return Ok(Confirmation::Unclear);
    }
    let samples = glasopis_core::audio::pad_to_min_duration(
        samples,
        glasopis_core::audio::WHISPER_SAMPLE_RATE,
        MIN_TRANSCRIPTION_SECONDS,
    );
    let text = crate::speech::transcribe_with_settings(app, settings, &samples)?;
    log::info!("асистент: потвърждение \"{text}\"");
    Ok(core_assistant::classify_confirmation(&text))
}

/// The AI interpretation step always needs a Groq API key - it is a chat
/// call, independent of which engine transcribes the audio itself.
fn ready_check(settings: &Settings) -> Result<()> {
    if !settings.cloud.has_key() {
        return Err(GlasopisError::MissingApiKey);
    }
    Ok(())
}

fn selected_device(settings: &Settings) -> Option<String> {
    match &settings.voice.microphone {
        glasopis_core::settings::MicrophoneChoice::Default => None,
        glasopis_core::settings::MicrophoneChoice::Device(name) => Some(name.clone()),
    }
}

fn speak_if_enabled(settings: &Settings, text: &str) {
    if !settings.assistant.speak_replies {
        return;
    }
    if !crate::tts::is_available() {
        log::info!("говоримият отговор пропуснат - няма инсталиран български глас в Windows");
        return;
    }
    if let Err(err) = crate::tts::speak(text) {
        log::info!("говоримият отговор не бе възможен: {err}");
    }
}

fn report_message(app: &AppHandle, message: String) {
    ui::shrink_overlay(app);
    let state = app.state::<AppState>();
    state.set_status(
        app,
        Status::Done {
            text: message,
            clipboard_only: false,
        },
    );
    hide_overlay_later(app, OVERLAY_LINGER_MS);
}

fn report(app: &AppHandle, err: GlasopisError) {
    ui::shrink_overlay(app);
    log::error!("асистентът приключи с грешка: {err}");
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
        if !state.recorder.is_recording() && !state.is_busy() && state.assistant_phase().is_none() {
            ui::hide_overlay(&app);
            state.set_status(&app, Status::Idle);
        }
    });
}
