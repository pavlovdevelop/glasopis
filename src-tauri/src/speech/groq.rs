//! Разпознаване на реч през Groq (`whisper-large-v3-turbo`).
//!
//! Записът се изпраща като WAV в паметта — на диска не се пише нищо. Ключът е
//! на потребителя и се въвежда в настройките.

use glasopis_core::wav::encode_wav_pcm16;

use crate::errors::{GlasopisError, Result};

const ENDPOINT: &str = "https://api.groq.com/openai/v1/audio/transcriptions";
/// Достатъчно за дълга диктовка при бавна връзка, но не безкрайно.
const TIMEOUT_SECONDS: u64 = 120;

pub fn transcribe(
    samples: &[f32],
    language: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
) -> Result<String> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(GlasopisError::MissingApiKey);
    }

    let wav = encode_wav_pcm16(samples, glasopis_core::audio::WHISPER_SAMPLE_RATE);
    log::info!(
        "изпращам {:.1} s аудио ({} KB) към Groq, модел {model}",
        samples.len() as f32 / glasopis_core::audio::WHISPER_SAMPLE_RATE as f32,
        wav.len() / 1024
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!("Glasopis/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECONDS))
        .build()
        .map_err(|err| GlasopisError::other(format!("Неуспешна мрежова заявка: {err}")))?;

    let file = reqwest::blocking::multipart::Part::bytes(wav)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|err| GlasopisError::other(format!("Неуспешно подготвяне на записа: {err}")))?;

    let mut form = reqwest::blocking::multipart::Form::new()
        .part("file", file)
        .text("model", model.to_string())
        .text("language", language.to_string())
        .text("response_format", "json")
        .text("temperature", "0");

    // Контекстът кара модела да изпише „dev сървъра“ вместо „дев сървъра“.
    if !prompt.trim().is_empty() {
        log::debug!("контекст за модела: {prompt}");
        form = form.text("prompt", prompt.to_string());
    }

    let started = std::time::Instant::now();
    let response = client
        .post(ENDPOINT)
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .map_err(|err| {
            log::error!("мрежова грешка към Groq: {err}");
            if err.is_timeout() {
                GlasopisError::other(
                    "Groq не отговори навреме. Проверете интернет връзката.".to_string(),
                )
            } else {
                GlasopisError::NetworkUnavailable
            }
        })?;

    let status = response.status();
    let body = response.text().unwrap_or_default();
    if !status.is_success() {
        return Err(api_error(status, &body));
    }

    let parsed: serde_json::Value = serde_json::from_str(&body)
        .map_err(|_| GlasopisError::other("Groq върна неочакван отговор.".to_string()))?;
    let text = parsed
        .get("text")
        .and_then(|value| value.as_str())
        .ok_or_else(|| GlasopisError::other("Groq върна отговор без текст.".to_string()))?;

    log::info!(
        "разпознаването отне {:.1} s",
        started.elapsed().as_secs_f32()
    );
    Ok(text.trim().to_string())
}

/// Превежда грешките на API-то в съобщения, които казват какво да се направи.
fn api_error(status: reqwest::StatusCode, body: &str) -> GlasopisError {
    let detail = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get("error")
                .and_then(|error| error.get("message"))
                .and_then(|message| message.as_str())
                .map(str::to_string)
        })
        .unwrap_or_default();

    match status.as_u16() {
        401 | 403 => GlasopisError::InvalidApiKey,
        413 => GlasopisError::other(
            "Записът е твърде дълъг за Groq. Диктувайте на по-кратки части.".to_string(),
        ),
        429 => GlasopisError::RateLimited,
        500..=599 => GlasopisError::other(
            "Groq има временен проблем. Опитайте отново след малко.".to_string(),
        ),
        _ => {
            log::error!("Groq върна {status}: {body}");
            if detail.is_empty() {
                GlasopisError::other(format!("Groq върна грешка {status}."))
            } else {
                GlasopisError::other(format!("Groq върна грешка: {detail}"))
            }
        }
    }
}

/// Проверява ключа с минимална заявка, за да може настройките да кажат
/// веднага дали е валиден.
pub fn check_api_key(api_key: &str, model: &str) -> Result<()> {
    // Половин секунда тишина е достатъчна за валидна заявка.
    let silence = vec![0.0f32; glasopis_core::audio::WHISPER_SAMPLE_RATE as usize / 2];
    transcribe(&silence, "bg", api_key, model, "").map(|_| ())
}
