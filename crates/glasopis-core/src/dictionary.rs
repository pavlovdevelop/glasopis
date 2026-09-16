//! Personal dictionary: spoken form -> preferred written form.
//!
//! Whisper reliably mis-writes brand names ("гит хъб" instead of "GitHub").
//! The dictionary is a simple, case-insensitive, word-boundary aware phrase
//! replacement that runs before the command processor.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DictionaryEntry {
    /// What the user says, e.g. `гит хъб`.
    pub spoken: String,
    /// What should be written, e.g. `GitHub`.
    pub written: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dictionary {
    pub entries: Vec<DictionaryEntry>,
}

impl Dictionary {
    pub fn new(entries: Vec<DictionaryEntry>) -> Self {
        Self { entries }
    }

    /// Replaces every known spoken form in `input`.
    ///
    /// Matching is done on whitespace separated tokens so that a replacement
    /// never happens in the middle of a longer word. Longer phrases win over
    /// shorter ones.
    pub fn apply(&self, input: &str) -> String {
        if self.entries.is_empty() {
            return input.to_string();
        }
        let mut sorted: Vec<&DictionaryEntry> = self
            .entries
            .iter()
            .filter(|e| !e.spoken.trim().is_empty())
            .collect();
        sorted.sort_by_key(|e| std::cmp::Reverse(e.spoken.split_whitespace().count()));

        let tokens: Vec<&str> = input.split_whitespace().collect();
        let mut out: Vec<String> = Vec::with_capacity(tokens.len());
        let mut i = 0;
        'outer: while i < tokens.len() {
            for entry in &sorted {
                let phrase: Vec<String> = entry
                    .spoken
                    .split_whitespace()
                    .map(|w| w.to_lowercase())
                    .collect();
                if phrase.is_empty() || i + phrase.len() > tokens.len() {
                    continue;
                }
                let matches = phrase.iter().enumerate().all(|(k, word)| {
                    normalize_token(tokens[i + k]) == *word
                });
                if matches {
                    // Keep trailing punctuation of the last matched token.
                    let tail = trailing_punctuation(tokens[i + phrase.len() - 1]);
                    out.push(format!("{}{}", entry.written, tail));
                    i += phrase.len();
                    continue 'outer;
                }
            }
            out.push(tokens[i].to_string());
            i += 1;
        }
        out.join(" ")
    }
}

const PUNCT: &[char] = &['.', ',', '!', '?', ';', ':', '…', '"', '„', '“', '\'', '(', ')'];

fn normalize_token(token: &str) -> String {
    token.trim_matches(PUNCT).to_lowercase()
}

fn trailing_punctuation(token: &str) -> String {
    token
        .chars()
        .rev()
        .take_while(|c| PUNCT.contains(c))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dict() -> Dictionary {
        Dictionary::new(vec![
            DictionaryEntry { spoken: "гит хъб".into(), written: "GitHub".into() },
            DictionaryEntry { spoken: "софт про нео".into(), written: "SoftProNeo".into() },
            DictionaryEntry { spoken: "гласопис".into(), written: "Glasopis".into() },
        ])
    }

    #[test]
    fn replaces_multiword_phrases() {
        assert_eq!(dict().apply("качи го в гит хъб"), "качи го в GitHub");
    }

    #[test]
    fn longest_match_wins() {
        assert_eq!(dict().apply("софт про нео е фирма"), "SoftProNeo е фирма");
    }

    #[test]
    fn keeps_trailing_punctuation() {
        assert_eq!(dict().apply("отвори гласопис."), "отвори Glasopis.");
    }

    #[test]
    fn empty_dictionary_is_identity() {
        assert_eq!(Dictionary::default().apply("нищо не се сменя"), "нищо не се сменя");
    }

    #[test]
    fn does_not_match_inside_words() {
        assert_eq!(dict().apply("гласописец"), "гласописец");
    }
}
