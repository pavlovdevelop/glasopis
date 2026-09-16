//! Catalogue of the local speech models Glasopis can download.
//!
//! All models are OpenAI Whisper weights converted to the ggml format and
//! published by the whisper.cpp project on Hugging Face. They are free to
//! download, require no account and no API key, and run entirely on the user's
//! computer (see `THIRD_PARTY_LICENSES.md`).

use serde::{Deserialize, Serialize};

/// How Glasopis presents a model to the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    /// Fast, lowest RAM/CPU cost, lowest accuracy.
    Fast,
    /// Default: good Bulgarian accuracy at a reasonable speed.
    Balanced,
    /// Best Bulgarian accuracy, slowest.
    Accurate,
}

impl Tier {
    pub fn label_bg(self) -> &'static str {
        match self {
            Tier::Fast => "Бърз",
            Tier::Balanced => "Балансиран",
            Tier::Accurate => "Висока точност",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Stable identifier used in the settings file.
    pub id: &'static str,
    /// Technical model name shown to advanced users.
    pub technical_name: &'static str,
    pub tier: Tier,
    /// File name inside `%LOCALAPPDATA%\Glasopis\models\`.
    pub file_name: &'static str,
    pub url: &'static str,
    /// Exact download size in bytes (from the Hugging Face LFS metadata).
    pub size_bytes: u64,
    /// SHA-256 of the file, verified after download.
    pub sha256: &'static str,
    /// Rough amount of RAM the model needs while transcribing.
    pub ram_mb: u32,
    pub recommended: bool,
}

impl ModelInfo {
    pub fn size_mb(&self) -> u64 {
        self.size_bytes / (1024 * 1024)
    }
}

const BASE_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/";

/// Every model Glasopis knows about.
pub const CATALOG: &[ModelInfo] = &[
    ModelInfo {
        id: "small-q5_1",
        technical_name: "whisper small (q5_1)",
        tier: Tier::Fast,
        file_name: "ggml-small-q5_1.bin",
        url: concat!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/",
            "ggml-small-q5_1.bin"
        ),
        size_bytes: 190_085_487,
        sha256: "ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb",
        ram_mb: 600,
        recommended: true,
    },
    ModelInfo {
        id: "large-v3-turbo-q5_0",
        technical_name: "whisper large-v3-turbo (q5_0)",
        tier: Tier::Balanced,
        file_name: "ggml-large-v3-turbo-q5_0.bin",
        url: concat!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/",
            "ggml-large-v3-turbo-q5_0.bin"
        ),
        size_bytes: 574_041_195,
        sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2",
        ram_mb: 1600,
        recommended: false,
    },
    ModelInfo {
        id: "medium",
        technical_name: "whisper medium (f16)",
        tier: Tier::Accurate,
        file_name: "ggml-medium.bin",
        url: concat!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/",
            "ggml-medium.bin"
        ),
        size_bytes: 1_533_763_059,
        sha256: "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
        ram_mb: 2600,
        recommended: false,
    },
];

/// The model a fresh installation suggests.
///
/// The small model, not the most accurate one: on a laptop it transcribes a
/// sentence in well under a second, and a voice typing tool that makes you wait
/// is a tool you stop using.
pub fn default_model() -> &'static ModelInfo {
    find("small-q5_1").expect("default model is part of the catalog")
}

pub fn find(id: &str) -> Option<&'static ModelInfo> {
    CATALOG.iter().find(|m| m.id == id)
}

/// The base URL every download must start with; used as a safety check so a
/// corrupted settings file cannot point the downloader at an arbitrary host.
pub fn is_trusted_url(url: &str) -> bool {
    url.starts_with(BASE_URL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_ids_are_unique() {
        let mut ids: Vec<&str> = CATALOG.iter().map(|m| m.id).collect();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count);
    }

    #[test]
    fn every_model_has_a_checksum_and_trusted_url() {
        for m in CATALOG {
            assert_eq!(m.sha256.len(), 64, "{} has a malformed checksum", m.id);
            assert!(
                is_trusted_url(m.url),
                "{} is not hosted on the trusted host",
                m.id
            );
            assert!(m.size_bytes > 0);
        }
    }

    #[test]
    fn exactly_one_recommended_model() {
        assert_eq!(CATALOG.iter().filter(|m| m.recommended).count(), 1);
        assert!(default_model().recommended);
    }

    #[test]
    fn size_mb_is_human_sized() {
        assert_eq!(default_model().size_mb(), 181);
    }
}
