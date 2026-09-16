//! whisper.cpp backend (through the `whisper-rs` bindings).
//!
//! Everything here runs locally on the CPU. The model file is a ggml/GGUF
//! Whisper model downloaded by the model manager.

use std::path::Path;

use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::errors::{GlasopisError, Result};
use crate::speech::SpeechEngine;

pub struct WhisperEngine {
    context: WhisperContext,
    threads: i32,
    /// Състоянието се преизползва между диктовките: създаването му заделя
    /// килобайти памет за KV кеша всеки път, а с `no_context` нищо не се
    /// пренася от предишния запис.
    state: Option<whisper_rs::WhisperState>,
}

impl WhisperEngine {
    pub fn load(model_path: &Path, threads: u32) -> Result<Self> {
        let context =
            WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
                .map_err(|err| {
                    log::error!("моделът не може да бъде зареден: {err}");
                    GlasopisError::ModelMissingOrCorrupted
                })?;
        Ok(Self {
            context,
            threads: threads.clamp(1, 16) as i32,
            state: None,
        })
    }
}

impl SpeechEngine for WhisperEngine {
    fn transcribe(&mut self, samples: &[f32], language: &str) -> Result<String> {
        let started = std::time::Instant::now();
        if self.state.is_none() {
            self.state = Some(self.context.create_state().map_err(|err| {
                log::error!("неуспешно създаване на whisper състояние: {err}");
                GlasopisError::TranscriptionFailed
            })?);
        }
        let state = self
            .state
            .as_mut()
            .expect("състоянието току-що беше създадено");

        let seconds = samples.len() as f32 / 16_000.0;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(self.threads);
        // Без това всяка диктовка се смята като тридесетсекундна.
        params.set_audio_ctx(glasopis_core::audio::whisper_audio_context(seconds));
        // Bulgarian (or whatever the user selected) — never auto-translate.
        params.set_language(Some(language));
        params.set_translate(false);
        params.set_no_timestamps(true);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        // Do not carry context between dictations: each one is independent and
        // this avoids the model repeating an earlier sentence.
        params.set_no_context(true);
        params.set_suppress_blank(true);
        // Whisper по подразбиране преразпознава записа до пет пъти с по-висока
        // температура, когато не е уверен. За диктовка това означава секунди
        // чакане за съмнителна печалба, затова остава един опит.
        params.set_temperature(0.0);
        params.set_temperature_inc(0.0);

        log::info!(
            "започвам разпознаване: {seconds:.1} s аудио, {} нишки, контекст {}",
            self.threads,
            glasopis_core::audio::whisper_audio_context(seconds)
        );
        state.full(params, samples).map_err(|err| {
            log::error!("неуспешно разпознаване: {err}");
            GlasopisError::TranscriptionFailed
        })?;

        log::info!(
            "разпознаването отне {:.1} s",
            started.elapsed().as_secs_f32()
        );

        let mut text = String::new();
        for segment in state.as_iter() {
            match segment.to_str_lossy() {
                Ok(part) => {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(part.trim());
                }
                Err(err) => log::warn!("сегмент без текст: {err}"),
            }
        }
        Ok(text.trim().to_string())
    }
}
