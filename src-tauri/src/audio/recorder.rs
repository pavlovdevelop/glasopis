//! Microphone recording.
//!
//! The cpal stream lives on its own thread because a WASAPI stream is not
//! `Send`. The thread builds the stream, collects samples into a shared buffer
//! and shuts down when it receives the stop signal. Nothing is ever written to
//! disk: the samples stay in memory and are dropped as soon as the transcript
//! has been produced.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use glasopis_core::audio as core_audio;
use parking_lot::Mutex;

use crate::audio::devices::resolve_device;
use crate::errors::{GlasopisError, Result};

/// Maximum length of a single dictation. Protects against a hotkey that was
/// never pressed a second time (and against unbounded memory growth).
pub const MAX_RECORDING_SECONDS: f32 = 300.0;

#[derive(Debug, Clone)]
pub struct CapturedAudio {
    /// Mono, 16 kHz, f32 samples - exactly what whisper.cpp expects.
    pub samples: Vec<f32>,
    pub seconds: f32,
}

struct Active {
    stop: Sender<()>,
    thread: JoinHandle<Result<CapturedAudio>>,
}

/// Owns the current recording, if any.
pub struct Recorder {
    active: Mutex<Option<Active>>,
    /// Current input level (0..1) encoded as the bits of an `f32`.
    level: Arc<AtomicU32>,
}

impl Default for Recorder {
    fn default() -> Self {
        Self::new()
    }
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            active: Mutex::new(None),
            level: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn is_recording(&self) -> bool {
        self.active.lock().is_some()
    }

    /// Current microphone level, 0.0 .. 1.0.
    pub fn level(&self) -> f32 {
        f32::from_bits(self.level.load(Ordering::Relaxed))
    }

    /// Starts recording from `device_name` (or the system default).
    pub fn start(&self, device_name: Option<String>) -> Result<()> {
        let mut active = self.active.lock();
        if active.is_some() {
            return Ok(());
        }
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<()>>();
        let level = Arc::clone(&self.level);

        let thread = std::thread::Builder::new()
            .name("glasopis-recorder".into())
            .spawn(move || {
                let outcome = run_capture(device_name, level, stop_rx, &ready_tx);
                if let Err(err) = &outcome {
                    log::error!("записът е прекратен: {err}");
                }
                outcome
            })
            .map_err(|e| GlasopisError::other(format!("Неуспешно стартиране на записа: {e}")))?;

        // Wait for the capture thread to report whether the stream started.
        match ready_rx.recv() {
            Ok(Ok(())) => {
                *active = Some(Active {
                    stop: stop_tx,
                    thread,
                });
                Ok(())
            }
            Ok(Err(err)) => {
                let _ = thread.join();
                Err(err)
            }
            Err(_) => Err(GlasopisError::other(
                "Записът не можа да бъде стартиран.".to_string(),
            )),
        }
    }

    /// Stops recording and returns the captured audio.
    pub fn stop(&self) -> Result<CapturedAudio> {
        let active = self.active.lock().take();
        let Some(active) = active else {
            return Err(GlasopisError::other("В момента не се записва.".to_string()));
        };
        let _ = active.stop.send(());
        self.level.store(0, Ordering::Relaxed);
        match active.thread.join() {
            Ok(result) => result,
            Err(_) => Err(GlasopisError::other(
                "Записът приключи неуспешно.".to_string(),
            )),
        }
    }

    /// Stops and throws away the audio (used when the user cancels).
    pub fn cancel(&self) {
        let active = self.active.lock().take();
        if let Some(active) = active {
            let _ = active.stop.send(());
            let _ = active.thread.join();
        }
        self.level.store(0, Ordering::Relaxed);
    }
}

fn run_capture(
    device_name: Option<String>,
    level: Arc<AtomicU32>,
    stop_rx: mpsc::Receiver<()>,
    ready_tx: &Sender<Result<()>>,
) -> Result<CapturedAudio> {
    let device = match resolve_device(device_name.as_deref()) {
        Ok(device) => device,
        Err(err) => {
            let _ = ready_tx.send(Err(err));
            return Err(GlasopisError::NoMicrophone);
        }
    };

    let (config, sample_format) = match pick_config(&device) {
        Ok(value) => value,
        Err(err) => {
            let _ = ready_tx.send(Err(err));
            return Err(GlasopisError::MicrophoneUnavailable);
        }
    };
    let channels = config.channels;
    let sample_rate = config.sample_rate;
    let max_samples = (sample_rate as f32 * MAX_RECORDING_SECONDS) as usize * channels as usize;

    let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::with_capacity(
        sample_rate as usize * channels as usize * 5,
    )));

    let stream = match build_stream(
        &device,
        &config,
        sample_format,
        Arc::clone(&buffer),
        Arc::clone(&level),
        channels,
        max_samples,
    ) {
        Ok(stream) => stream,
        Err(err) => {
            let _ = ready_tx.send(Err(err));
            return Err(GlasopisError::MicrophoneUnavailable);
        }
    };

    if let Err(err) = stream.play() {
        let message = format!("Микрофонът не може да бъде стартиран: {err}");
        let _ = ready_tx.send(Err(GlasopisError::other(message)));
        return Err(GlasopisError::MicrophoneUnavailable);
    }
    let _ = ready_tx.send(Ok(()));

    // Block until the dictation is stopped (or the safety limit is reached).
    loop {
        match stop_rx.recv_timeout(std::time::Duration::from_millis(250)) {
            Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if buffer.lock().len() >= max_samples {
                    log::warn!("достигнат е лимитът от {MAX_RECORDING_SECONDS} секунди");
                    break;
                }
            }
        }
    }

    drop(stream);
    level.store(0, Ordering::Relaxed);

    let raw = std::mem::take(&mut *buffer.lock());
    let samples = core_audio::prepare_for_whisper(&raw, channels, sample_rate);
    let seconds = core_audio::duration_secs(&samples, core_audio::WHISPER_SAMPLE_RATE);
    Ok(CapturedAudio { samples, seconds })
}

/// Chooses a capture configuration, preferring 16 kHz so no resampling is
/// needed at all.
fn pick_config(device: &cpal::Device) -> Result<(StreamConfig, SampleFormat)> {
    let default = device
        .default_input_config()
        .map_err(|e| GlasopisError::other(format!("Микрофонът не е достъпен: {e}")))?;

    if let Ok(supported) = device.supported_input_configs() {
        let target = core_audio::WHISPER_SAMPLE_RATE;
        let mut candidates: Vec<_> = supported
            .filter(|range| {
                range.min_sample_rate() <= target
                    && range.max_sample_rate() >= target
                    && matches!(
                        range.sample_format(),
                        SampleFormat::F32 | SampleFormat::I16 | SampleFormat::U16
                    )
            })
            .collect();
        // Fewest channels first: mono capture needs no mixdown.
        candidates.sort_by_key(|range| range.channels());
        if let Some(range) = candidates.first() {
            let chosen = (*range).with_sample_rate(target);
            return Ok((chosen.config(), chosen.sample_format()));
        }
    }

    Ok((default.config(), default.sample_format()))
}

fn build_stream(
    device: &cpal::Device,
    config: &StreamConfig,
    format: SampleFormat,
    buffer: Arc<Mutex<Vec<f32>>>,
    level: Arc<AtomicU32>,
    channels: u16,
    max_samples: usize,
) -> Result<cpal::Stream> {
    let on_error = |err| log::error!("грешка в аудио потока: {err}");

    macro_rules! stream_for {
        ($sample:ty, $convert:expr) => {{
            let buffer = Arc::clone(&buffer);
            let level = Arc::clone(&level);
            device.build_input_stream(
                config,
                move |data: &[$sample], _: &cpal::InputCallbackInfo| {
                    let converted: Vec<f32> = $convert(data);
                    update_level(&level, &converted, channels);
                    let mut guard = buffer.lock();
                    if guard.len() < max_samples {
                        guard.extend_from_slice(&converted);
                    }
                },
                on_error,
                None,
            )
        }};
    }

    let stream = match format {
        SampleFormat::F32 => stream_for!(f32, |data: &[f32]| data.to_vec()),
        SampleFormat::I16 => stream_for!(i16, |data: &[i16]| core_audio::i16_to_f32(data)),
        SampleFormat::U16 => stream_for!(u16, |data: &[u16]| core_audio::u16_to_f32(data)),
        other => {
            return Err(GlasopisError::other(format!(
                "Неподдържан аудио формат на микрофона: {other:?}"
            )))
        }
    };

    stream.map_err(|err| match err {
        cpal::BuildStreamError::DeviceNotAvailable => GlasopisError::MicrophoneUnavailable,
        other => {
            let message = other.to_string();
            // WASAPI reports a denied microphone permission as a generic
            // backend error, so the message is what identifies it.
            if message.contains("0x80070005")
                || message.to_lowercase().contains("access is denied")
                || message.to_lowercase().contains("access denied")
            {
                GlasopisError::MicrophoneAccessDenied
            } else {
                GlasopisError::other(format!("Микрофонът не може да бъде отворен: {message}"))
            }
        }
    })
}

fn update_level(level: &AtomicU32, samples: &[f32], channels: u16) {
    let mono = core_audio::mixdown_to_mono(samples, channels);
    let value = core_audio::meter_level(&mono);
    // Smooth the meter a little so the waveform does not flicker.
    let previous = f32::from_bits(level.load(Ordering::Relaxed));
    let smoothed = if value > previous {
        value
    } else {
        previous * 0.7 + value * 0.3
    };
    level.store(smoothed.to_bits(), Ordering::Relaxed);
}
