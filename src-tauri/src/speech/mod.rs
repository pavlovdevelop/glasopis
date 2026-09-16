//! Local speech recognition.
//!
//! The engine is abstracted behind [`SpeechEngine`] so the rest of the
//! application never talks to whisper.cpp directly. Today there is exactly one
//! implementation — local whisper.cpp through `whisper-rs`. There is no cloud
//! backend and no API key anywhere in Glasopis: audio never leaves the machine.

use std::path::{Path, PathBuf};

use parking_lot::Mutex;

use crate::errors::{GlasopisError, Result};

#[cfg(feature = "whisper")]
mod whisper_engine;

/// Everything the dictation flow needs from a speech backend.
pub trait SpeechEngine: Send {
    /// Transcribes mono 16 kHz audio into text.
    fn transcribe(&mut self, samples: &[f32], language: &str) -> Result<String>;
}

/// Loads the engine lazily and keeps it in memory between dictations, so the
/// model file is only read from disk once.
pub struct EngineHolder {
    inner: Mutex<Option<LoadedEngine>>,
}

struct LoadedEngine {
    path: PathBuf,
    threads: u32,
    engine: Box<dyn SpeechEngine>,
}

impl Default for EngineHolder {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineHolder {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    /// Drops the loaded model (used when the user selects another one).
    pub fn unload(&self) {
        *self.inner.lock() = None;
    }

    pub fn is_loaded(&self) -> bool {
        self.inner.lock().is_some()
    }

    /// Transcribes with the model at `model_path`, loading it if needed.
    pub fn transcribe(
        &self,
        model_path: &Path,
        samples: &[f32],
        language: &str,
        threads: u32,
    ) -> Result<String> {
        if !model_path.is_file() {
            return Err(GlasopisError::ModelMissingOrCorrupted);
        }
        let mut guard = self.inner.lock();
        let needs_reload = match guard.as_ref() {
            Some(loaded) => loaded.path != model_path || loaded.threads != threads,
            None => true,
        };
        if needs_reload {
            log::info!("зареждам модел: {}", model_path.display());
            let engine = create_engine(model_path, threads)?;
            *guard = Some(LoadedEngine {
                path: model_path.to_path_buf(),
                threads,
                engine,
            });
        }
        let loaded = guard.as_mut().expect("моделът току-що беше зареден");
        loaded.engine.transcribe(samples, language)
    }
}

#[cfg(feature = "whisper")]
fn create_engine(model_path: &Path, threads: u32) -> Result<Box<dyn SpeechEngine>> {
    Ok(Box::new(whisper_engine::WhisperEngine::load(
        model_path, threads,
    )?))
}

#[cfg(not(feature = "whisper"))]
fn create_engine(_model_path: &Path, _threads: u32) -> Result<Box<dyn SpeechEngine>> {
    // This build was produced with `--no-default-features` (used for
    // type-checking on machines without a C/C++ toolchain). Real releases
    // always include the whisper feature.
    Err(GlasopisError::other(
        "Тази компилация е без вградено разпознаване на реч (feature `whisper`).".to_string(),
    ))
}

/// Number of threads to use when the user has not chosen a value.
///
/// Whisper stops scaling well past a handful of threads, and on a hybrid CPU
/// the efficiency cores only slow the others down. Half the logical cores is a
/// good approximation of "the fast cores", capped at eight because the gain
/// past that is noise. The user can override it in the settings.
pub fn default_threads() -> u32 {
    let cores = std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(4);
    (cores / 2).clamp(1, 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_threads_is_sane() {
        let threads = default_threads();
        assert!((1..=8).contains(&threads), "нишки: {threads}");
    }

    #[test]
    fn missing_model_file_is_reported() {
        let holder = EngineHolder::new();
        let err = holder
            .transcribe(Path::new("/не/съществува.bin"), &[0.0; 16], "bg", 2)
            .unwrap_err();
        assert!(matches!(err, GlasopisError::ModelMissingOrCorrupted));
    }
}
