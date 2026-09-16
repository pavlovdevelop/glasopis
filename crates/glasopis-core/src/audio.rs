//! Pure audio helpers.
//!
//! Whisper expects 16 kHz, mono, 32-bit float samples in the range [-1, 1].
//! Capture devices rarely deliver exactly that, so the recorder converts the
//! captured frames with the functions below. They are deliberately free of any
//! device/OS dependency so they can be tested.

/// Sample rate required by whisper.cpp.
pub const WHISPER_SAMPLE_RATE: u32 = 16_000;

/// Converts interleaved 16-bit PCM to float samples.
pub fn i16_to_f32(samples: &[i16]) -> Vec<f32> {
    samples
        .iter()
        .map(|s| *s as f32 / i16::MAX as f32)
        .collect()
}

/// Converts interleaved unsigned 16-bit PCM to float samples.
pub fn u16_to_f32(samples: &[u16]) -> Vec<f32> {
    samples
        .iter()
        .map(|s| (*s as f32 - 32_768.0) / 32_768.0)
        .collect()
}

/// Averages interleaved channels into a single mono channel.
pub fn mixdown_to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    let channels = channels.max(1) as usize;
    if channels == 1 {
        return samples.to_vec();
    }
    samples
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
        .collect()
}

/// Resamples mono audio.
///
/// When down-sampling, a moving-average low-pass filter is applied first to
/// avoid the aliasing that plain decimation would introduce (48 kHz -> 16 kHz
/// is the common case on Windows). Linear interpolation is then used for the
/// rate conversion itself, which is accurate enough for speech recognition.
pub fn resample(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate || input.is_empty() || from_rate == 0 || to_rate == 0 {
        return input.to_vec();
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let filtered: Vec<f32> = if ratio > 1.0 {
        low_pass(input, ratio.round() as usize)
    } else {
        input.to_vec()
    };

    let out_len = ((input.len() as f64) / ratio).floor() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let pos = i as f64 * ratio;
        let idx = pos.floor() as usize;
        let frac = (pos - idx as f64) as f32;
        let a = filtered[idx.min(filtered.len() - 1)];
        let b = filtered[(idx + 1).min(filtered.len() - 1)];
        out.push(a + (b - a) * frac);
    }
    out
}

fn low_pass(input: &[f32], window: usize) -> Vec<f32> {
    if window <= 1 {
        return input.to_vec();
    }
    let mut out = Vec::with_capacity(input.len());
    let mut acc = 0.0f32;
    for (i, sample) in input.iter().enumerate() {
        acc += *sample;
        if i >= window {
            acc -= input[i - window];
        }
        let count = (i + 1).min(window) as f32;
        out.push(acc / count);
    }
    out
}

/// Root mean square amplitude of a block of samples.
pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = samples.iter().map(|s| s * s).sum();
    (sum / samples.len() as f32).sqrt()
}

/// Maps the RMS amplitude to a 0..1 value suitable for a level meter.
///
/// The mapping is logarithmic (-60 dBFS .. 0 dBFS) so that normal speech fills
/// a useful part of the meter instead of hugging the bottom.
pub fn meter_level(samples: &[f32]) -> f32 {
    let rms = rms(samples);
    if rms <= 0.000_001 {
        return 0.0;
    }
    let db = 20.0 * rms.log10();
    ((db + 60.0) / 60.0).clamp(0.0, 1.0)
}

/// Length of a mono buffer in seconds.
pub fn duration_secs(samples: &[f32], sample_rate: u32) -> f32 {
    if sample_rate == 0 {
        return 0.0;
    }
    samples.len() as f32 / sample_rate as f32
}

/// True when the recording is (practically) silence, so Glasopis can skip
/// transcription instead of making the model hallucinate words.
pub fn is_silence(samples: &[f32]) -> bool {
    rms(samples) < 0.002
}

/// Converts whatever the capture device produced into the mono 16 kHz float
/// buffer whisper.cpp expects.
pub fn prepare_for_whisper(samples: &[f32], channels: u16, sample_rate: u32) -> Vec<f32> {
    let mono = mixdown_to_mono(samples, channels);
    resample(&mono, sample_rate, WHISPER_SAMPLE_RATE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stereo_is_averaged() {
        let stereo = [1.0, 0.0, 0.5, 0.5, -1.0, 1.0];
        assert_eq!(mixdown_to_mono(&stereo, 2), vec![0.5, 0.5, 0.0]);
    }

    #[test]
    fn mono_is_passed_through() {
        let mono = [0.1, 0.2, 0.3];
        assert_eq!(mixdown_to_mono(&mono, 1), mono.to_vec());
    }

    #[test]
    fn resampling_changes_the_length_proportionally() {
        let input: Vec<f32> = (0..48_000).map(|i| (i as f32 * 0.001).sin()).collect();
        let out = resample(&input, 48_000, 16_000);
        assert_eq!(out.len(), 16_000);
    }

    #[test]
    fn same_rate_is_a_no_op() {
        let input = vec![0.1, -0.2, 0.3];
        assert_eq!(resample(&input, 16_000, 16_000), input);
    }

    #[test]
    fn upsampling_works() {
        let input = vec![0.0, 1.0];
        let out = resample(&input, 8_000, 16_000);
        assert_eq!(out.len(), 4);
    }

    #[test]
    fn prepare_produces_16k_mono() {
        let stereo: Vec<f32> = (0..96_000)
            .map(|i| if i % 2 == 0 { 0.5 } else { -0.5 })
            .collect();
        let out = prepare_for_whisper(&stereo, 2, 48_000);
        assert_eq!(out.len(), 16_000);
    }

    #[test]
    fn level_meter_range() {
        assert_eq!(meter_level(&[0.0; 16]), 0.0);
        let loud = meter_level(&[1.0; 16]);
        assert!((loud - 1.0).abs() < 0.001, "loud = {loud}");
        let quiet = meter_level(&[0.01; 16]);
        assert!(quiet > 0.0 && quiet < 1.0, "quiet = {quiet}");
    }

    #[test]
    fn silence_is_detected() {
        assert!(is_silence(&[0.0001; 1000]));
        assert!(!is_silence(&[0.2; 1000]));
    }

    #[test]
    fn integer_conversion() {
        assert!((i16_to_f32(&[i16::MAX])[0] - 1.0).abs() < 1e-6);
        assert!((u16_to_f32(&[32_768])[0]).abs() < 1e-6);
    }

    #[test]
    fn duration_is_reported_in_seconds() {
        assert!((duration_secs(&vec![0.0; 32_000], 16_000) - 2.0).abs() < 1e-6);
    }
}
