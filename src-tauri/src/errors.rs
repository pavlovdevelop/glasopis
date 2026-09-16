//! Application errors with user facing Bulgarian messages.

#[derive(Debug, thiserror::Error)]
pub enum GlasopisError {
    #[error("Не е открит микрофон.")]
    NoMicrophone,
    #[error("Glasopis няма достъп до микрофона. Проверете Настройки → Поверителност → Микрофон.")]
    MicrophoneAccessDenied,
    #[error("Избраният микрофон не е наличен. Изберете друг от настройките.")]
    MicrophoneUnavailable,
    #[error("Не е избран модел за разпознаване на реч.")]
    NoModelSelected,
    #[error("Файлът на модела липсва или е повреден. Изтеглете модела отново.")]
    ModelMissingOrCorrupted,
    #[error("Неуспешно разпознаване на речта.")]
    #[cfg_attr(not(feature = "whisper"), allow(dead_code))]
    TranscriptionFailed,
    #[error("Не успях да въведа текста в активното приложение. Текстът е копиран в клипборда.")]
    InjectionFailed,
    #[error("Няма връзка с интернет за изтегляне на модела.")]
    NetworkUnavailable,
    #[error("Тази функция изисква Windows.")]
    #[cfg_attr(windows, allow(dead_code))]
    WindowsOnly,
    #[error("{0}")]
    Other(String),
}

impl GlasopisError {
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}

impl From<anyhow::Error> for GlasopisError {
    fn from(value: anyhow::Error) -> Self {
        Self::Other(value.to_string())
    }
}

impl From<std::io::Error> for GlasopisError {
    fn from(value: std::io::Error) -> Self {
        Self::Other(value.to_string())
    }
}

impl From<tauri::Error> for GlasopisError {
    fn from(value: tauri::Error) -> Self {
        Self::Other(value.to_string())
    }
}

impl serde::Serialize for GlasopisError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Tauri commands return the Bulgarian message straight to the UI.
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, GlasopisError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_are_bulgarian() {
        assert!(GlasopisError::NoMicrophone.to_string().contains("микрофон"));
        assert!(GlasopisError::NoModelSelected.to_string().contains("модел"));
    }

    #[test]
    fn injection_error_mentions_the_clipboard_fallback() {
        assert!(GlasopisError::InjectionFailed
            .to_string()
            .contains("клипборда"));
    }

    #[test]
    fn errors_serialize_as_their_message() {
        let json = serde_json::to_string(&GlasopisError::NoMicrophone).unwrap();
        assert_eq!(json, "\"Не е открит микрофон.\"");
    }
}
