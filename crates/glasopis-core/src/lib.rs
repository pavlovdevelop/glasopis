//! Glasopis core — platform independent logic.
//!
//! This crate deliberately contains no Windows, Tauri or audio-device code so it
//! can be unit tested on any platform (and in CI on Linux). It holds:
//!
//! * [`settings`] — user configuration and its on-disk representation
//! * [`commands`] — the Bulgarian voice command processor
//! * [`text`] — transcript normalisation / formatting
//! * [`dictionary`] — the personal dictionary (spoken form -> written form)
//! * [`models`] — the speech model catalogue and its metadata
//! * [`history`] — the optional local dictation history
//! * [`audio`] — pure audio helpers (mixdown, resampling, level metering)

pub mod audio;
pub mod commands;
pub mod dictionary;
pub mod history;
pub mod models;
pub mod pipeline;
pub mod settings;
pub mod text;

pub use pipeline::{process_transcript, PipelineOptions};
