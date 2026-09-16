//! Моделите за разпознаване, които Groq предлага.
//!
//! Данните са от документацията на Groq (console.groq.com/docs/speech-to-text).
//! Тук стои само техническата информация; имената, които вижда потребителят,
//! идват от преводите в интерфейса.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloudModelInfo {
    /// Идентификатор, който се изпраща на API-то.
    pub id: &'static str,
    /// Дял на сгрешените думи по данни на доставчика, в проценти.
    /// По-малко е по-добре.
    pub word_error_rate: f32,
    /// Дали е оптимизиран за скорост за сметка на точност.
    pub fast: bool,
    /// Моделът, който получава нова инсталация.
    pub default: bool,
}

pub const CLOUD_CATALOG: &[CloudModelInfo] = &[
    CloudModelInfo {
        id: "whisper-large-v3-turbo",
        word_error_rate: 12.0,
        fast: true,
        default: true,
    },
    CloudModelInfo {
        id: "whisper-large-v3",
        word_error_rate: 10.3,
        fast: false,
        default: false,
    },
];

pub fn find(id: &str) -> Option<&'static CloudModelInfo> {
    CLOUD_CATALOG.iter().find(|model| model.id == id)
}

pub fn is_known(id: &str) -> bool {
    find(id).is_some()
}

pub fn default_model() -> &'static CloudModelInfo {
    CLOUD_CATALOG
        .iter()
        .find(|model| model.default)
        .expect("каталогът има модел по подразбиране")
}

/// Най-точният модел в каталога.
pub fn most_accurate() -> &'static CloudModelInfo {
    CLOUD_CATALOG
        .iter()
        .min_by(|a, b| a.word_error_rate.total_cmp(&b.word_error_rate))
        .expect("каталогът не е празен")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_consistent() {
        assert!(!CLOUD_CATALOG.is_empty());
        assert_eq!(CLOUD_CATALOG.iter().filter(|m| m.default).count(), 1);
        for model in CLOUD_CATALOG {
            assert!(!model.id.is_empty());
            assert!(model.word_error_rate > 0.0 && model.word_error_rate < 100.0);
        }
    }

    #[test]
    fn the_accurate_model_is_not_the_fast_one() {
        assert_eq!(most_accurate().id, "whisper-large-v3");
        assert!(!most_accurate().fast);
        assert_eq!(default_model().id, "whisper-large-v3-turbo");
        assert!(default_model().fast);
    }

    #[test]
    fn unknown_ids_are_rejected() {
        assert!(is_known("whisper-large-v3"));
        assert!(!is_known("gpt-4"));
        assert!(!is_known(""));
    }
}
