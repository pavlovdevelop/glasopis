//! Glasopis core - platform independent logic.
//!
//! This crate deliberately contains no Windows, Tauri or audio-device code so it
//! can be unit tested on any platform (and in CI on Linux). It holds:
//!
//! * [`settings`] - user configuration and its on-disk representation
//! * [`commands`] - the Bulgarian voice command processor
//! * [`text`] - transcript normalisation / formatting
//! * [`dictionary`] - the personal dictionary (spoken form -> written form)
//! * [`models`] - the local speech model catalogue and its metadata
//! * [`cloud_models`] - the transcription models the cloud provider offers
//! * [`history`] - the optional local dictation history
//! * [`audio`] - pure audio helpers (mixdown, resampling, level metering)
//! * [`wav`] - WAV encoding for the cloud transcription request

pub mod audio;
pub mod cloud_models;
pub mod commands;
pub mod dictionary;
pub mod history;
pub mod models;
pub mod pipeline;
pub mod settings;
pub mod text;
pub mod wav;

pub use pipeline::{process_transcript, PipelineOptions};
