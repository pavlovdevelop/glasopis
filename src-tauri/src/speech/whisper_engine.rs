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
        })
    }
}

impl SpeechEngine for WhisperEngine {
    fn transcribe(&mut self, samples: &[f32], language: &str) -> Result<String> {
        let mut state = self.context.create_state().map_err(|err| {
            log::error!("неуспешно създаване на whisper състояние: {err}");
            GlasopisError::TranscriptionFailed
        })?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(self.threads);
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

        state.full(params, samples).map_err(|err| {
            log::error!("неуспешно разпознаване: {err}");
            GlasopisError::TranscriptionFailed
        })?;

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
