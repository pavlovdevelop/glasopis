//! The text pipeline that sits between the speech engine and the injector.
//!
//! ```text
//! raw transcript -> personal dictionary -> voice commands -> normalisation
//! ```
//!
//! It is a pure function, which makes the whole text behaviour of Glasopis
//! testable without a microphone or a model.

use crate::{commands, dictionary::Dictionary, settings::Settings, text};

#[derive(Debug, Clone)]
pub struct PipelineOptions {
    pub voice_commands: bool,
    pub capitalize: bool,
    pub dictionary: Dictionary,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        Self {
            voice_commands: true,
            capitalize: true,
            dictionary: Dictionary::default(),
        }
    }
}

impl From<&Settings> for PipelineOptions {
    fn from(settings: &Settings) -> Self {
        Self {
            voice_commands: settings.voice.voice_commands,
            capitalize: settings.voice.capitalize_sentences,
            dictionary: settings.dictionary.clone(),
        }
    }
}

/// Turns the raw engine output into the text that gets inserted.
pub fn process_transcript(raw: &str, options: &PipelineOptions) -> String {
    let cleaned = text::normalize(raw, false);
    if cleaned.is_empty() {
        return String::new();
    }
    let with_dictionary = options.dictionary.apply(&cleaned);
    let with_commands = commands::process(&with_dictionary, options.voice_commands);
    text::normalize(&with_commands, options.capitalize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::DictionaryEntry;

    #[test]
    fn full_pipeline_bulgarian() {
        let out = process_transcript(
            "здравей запетая как си въпросителен знак",
            &PipelineOptions::default(),
        );
        assert_eq!(out, "Здравей, как си?");
    }

    #[test]
    fn dictionary_runs_before_commands() {
        let options = PipelineOptions {
            dictionary: Dictionary::new(vec![DictionaryEntry {
                spoken: "гит хъб".into(),
                written: "GitHub".into(),
            }]),
            ..PipelineOptions::default()
        };
        let out = process_transcript("качи проекта в гит хъб точка", &options);
        assert_eq!(out, "Качи проекта в GitHub.");
    }

    #[test]
    fn already_punctuated_speech_is_left_alone() {
        let out = process_transcript(
            "Днес трябва да изпратя документите на клиента.",
            &PipelineOptions::default(),
        );
        assert_eq!(out, "Днес трябва да изпратя документите на клиента.");
    }

    #[test]
    fn empty_input_produces_empty_output() {
        assert_eq!(process_transcript("   ", &PipelineOptions::default()), "");
        assert_eq!(
            process_transcript("[музика]", &PipelineOptions::default()),
            ""
        );
    }

    #[test]
    fn multiline_dictation() {
        let out = process_transcript(
            "здравей нов ред това е втори ред нов параграф трети абзац точка",
            &PipelineOptions::default(),
        );
        assert_eq!(out, "Здравей\nТова е втори ред\n\nТрети абзац.");
    }

    #[test]
    fn options_follow_the_settings() {
        let mut settings = Settings::default();
        settings.voice.voice_commands = false;
        settings.voice.capitalize_sentences = false;
        let options = PipelineOptions::from(&settings);
        assert_eq!(
            process_transcript("здравей точка", &options),
            "здравей точка"
        );
    }
}
