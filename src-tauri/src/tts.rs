//! Говорим отговор от асистента, през вградения в Windows синтезатор на реч
//! (WinRT `SpeechSynthesizer`). Изцяло локално - няма мрежа, няма акаунт.
//!
//! Ако системата няма инсталиран български глас, [`speak`] връща грешка и
//! извикващият код продължава само с текстовото показване - никога не блокира
//! асистента заради липсващ говорител.

#[cfg(windows)]
mod imp {
    use windows::core::HSTRING;
    use windows::Media::Core::MediaSource;
    use windows::Media::Playback::{MediaPlaybackState, MediaPlayer};
    use windows::Media::SpeechSynthesis::{
        SpeechSynthesisStream, SpeechSynthesizer, VoiceInformation,
    };
    use windows_future::{AsyncStatus, IAsyncOperation};

    use crate::errors::{GlasopisError, Result};

    /// Blocks until `op` finishes and returns its result. WinRT async values
    /// have no blocking `.get()`/`.join()` exposed here, so this polls
    /// `Status()` - the assistant flow is already on its own background
    /// thread, so blocking it briefly is fine.
    fn wait_for<T: windows::core::RuntimeType + 'static>(
        op: windows::core::Result<IAsyncOperation<T>>,
    ) -> windows::core::Result<T> {
        let op = op?;
        loop {
            match op.Status()? {
                AsyncStatus::Completed => return op.GetResults(),
                AsyncStatus::Error | AsyncStatus::Canceled => return op.GetResults(),
                _ => std::thread::sleep(std::time::Duration::from_millis(20)),
            }
        }
    }

    /// Търси първия инсталиран глас с български език (`bg-*`).
    fn bulgarian_voice() -> Option<VoiceInformation> {
        let voices = SpeechSynthesizer::AllVoices().ok()?;
        for i in 0..voices.Size().ok()? {
            let voice = voices.GetAt(i).ok()?;
            let language = voice.Language().ok()?.to_string_lossy();
            if language.to_lowercase().starts_with("bg") {
                return Some(voice);
            }
        }
        None
    }

    pub fn is_available() -> bool {
        bulgarian_voice().is_some()
    }

    /// Синтезира `text` и изчаква изпълнението на записа да приключи, преди
    /// да върне резултат - асистентът трябва да знае кога да продължи
    /// (напр. да започне да слуша за да/не).
    pub fn speak(text: &str) -> Result<()> {
        let voice = bulgarian_voice()
            .ok_or_else(|| GlasopisError::other("Няма инсталиран български глас.".to_string()))?;

        let synthesizer = SpeechSynthesizer::new()
            .map_err(|err| GlasopisError::other(format!("Синтезаторът не стартира: {err}")))?;
        synthesizer
            .SetVoice(&voice)
            .map_err(|err| GlasopisError::other(format!("Гласът не беше избран: {err}")))?;

        let stream: SpeechSynthesisStream = wait_for(
            synthesizer.SynthesizeTextToStreamAsync(&HSTRING::from(text)),
        )
        .map_err(|err| GlasopisError::other(format!("Текстът не беше синтезиран: {err}")))?;
        let content_type = stream.ContentType().map_err(|err| {
            GlasopisError::other(format!("Синтезираният запис е повреден: {err}"))
        })?;
        let source = MediaSource::CreateFromStream(&stream, &content_type)
            .map_err(|err| GlasopisError::other(format!("Записът не беше зареден: {err}")))?;

        let player = MediaPlayer::new()
            .map_err(|err| GlasopisError::other(format!("Плейърът не стартира: {err}")))?;
        player
            .SetSource(&source)
            .map_err(|err| GlasopisError::other(format!("Записът не беше зареден: {err}")))?;

        // Изчаква изпълнението: `windows` крейтът предлага event handler-и,
        // но обикновено поле-чакане с кратък sleep е достатъчно надеждно тук
        // и избягва усложненията на cross-thread callback-и в WinRT.
        player
            .Play()
            .map_err(|err| GlasopisError::other(format!("Възпроизвеждането не тръгна: {err}")))?;

        let session = player
            .PlaybackSession()
            .map_err(|err| GlasopisError::other(format!("Сесията липсва: {err}")))?;
        // Малка начална пауза, за да навлезе плейърът в състояние Playing,
        // преди да проверяваме дали е спрял.
        std::thread::sleep(std::time::Duration::from_millis(150));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            let state = session.PlaybackState().unwrap_or(MediaPlaybackState::None);
            if state != MediaPlaybackState::Playing || std::time::Instant::now() > deadline {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(80));
        }
        Ok(())
    }
}

#[cfg(not(windows))]
mod imp {
    use crate::errors::{GlasopisError, Result};

    pub fn is_available() -> bool {
        false
    }

    pub fn speak(_text: &str) -> Result<()> {
        Err(GlasopisError::WindowsOnly)
    }
}

pub use imp::{is_available, speak};
