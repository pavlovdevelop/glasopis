//! Кодиране на записа във WAV, какъвто очаква Groq API.
//!
//! Файлът се сглобява в паметта и се изпраща директно — нищо не се записва
//! на диска.

/// Опакова моно f32 семпли като 16-битов PCM WAV.
pub fn encode_wav_pcm16(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    const CHANNELS: u16 = 1;
    const BITS: u16 = 16;

    let byte_rate = sample_rate * CHANNELS as u32 * (BITS / 8) as u32;
    let block_align = CHANNELS * (BITS / 8);
    let data_len = (samples.len() * 2) as u32;

    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");

    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // размер на fmt блока
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&CHANNELS.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&BITS.to_le_bytes());

    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let value = (clamped * i16::MAX as f32).round() as i16;
        out.extend_from_slice(&value.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_is_a_valid_riff_wave() {
        let wav = encode_wav_pcm16(&[0.0; 16_000], 16_000);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[36..40], b"data");
    }

    #[test]
    fn length_matches_the_samples() {
        let wav = encode_wav_pcm16(&[0.0; 1_000], 16_000);
        assert_eq!(wav.len(), 44 + 2_000);
        let declared = u32::from_le_bytes(wav[4..8].try_into().unwrap());
        assert_eq!(declared as usize, wav.len() - 8);
    }

    #[test]
    fn sample_rate_and_format_are_written() {
        let wav = encode_wav_pcm16(&[0.0; 8], 16_000);
        assert_eq!(u16::from_le_bytes(wav[20..22].try_into().unwrap()), 1); // PCM
        assert_eq!(u16::from_le_bytes(wav[22..24].try_into().unwrap()), 1); // моно
        assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), 16_000);
        assert_eq!(u16::from_le_bytes(wav[34..36].try_into().unwrap()), 16); // бита
    }

    #[test]
    fn samples_are_converted_and_clamped() {
        let wav = encode_wav_pcm16(&[1.0, -1.0, 2.0, -2.0, 0.0], 16_000);
        let value = |i: usize| i16::from_le_bytes(wav[44 + i * 2..46 + i * 2].try_into().unwrap());
        assert_eq!(value(0), i16::MAX);
        assert_eq!(value(1), -i16::MAX);
        assert_eq!(value(2), i16::MAX, "стойност над 1.0 се ограничава");
        assert_eq!(value(3), -i16::MAX);
        assert_eq!(value(4), 0);
    }
}
