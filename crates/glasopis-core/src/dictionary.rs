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
                let matches = phrase
                    .iter()
                    .enumerate()
                    .all(|(k, word)| normalize_token(tokens[i + k]) == *word);
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

/// Whisper приема кратък текст като контекст и се старае да следва
/// изписването в него. Ограничението е около 224 токена, затова подсказката
/// се реже.
const MAX_PROMPT_CHARS: usize = 600;

/// Сглобява подсказката за разпознаването: думите, които потребителят иска
/// изписани по определен начин, плюс допълнителни термини.
///
/// Точно това кара модела да напише `dev сървъра`, а не `дев сървъра`.
pub fn build_prompt(dictionary: &Dictionary, extra_terms: &str) -> String {
    let mut terms: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    let mut push =
        |term: &str, terms: &mut Vec<String>, seen: &mut std::collections::HashSet<String>| {
            let term = term.trim();
            if term.is_empty() {
                return;
            }
            if seen.insert(term.to_lowercase()) {
                terms.push(term.to_string());
            }
        };

    for entry in &dictionary.entries {
        push(&entry.written, &mut terms, &mut seen);
    }
    for term in extra_terms.split([',', '\n', ';']) {
        push(term, &mut terms, &mut seen);
    }

    let mut prompt = String::new();
    for term in terms {
        let addition = if prompt.is_empty() {
            term
        } else {
            format!(", {term}")
        };
        if prompt.len() + addition.len() > MAX_PROMPT_CHARS {
            break;
        }
        prompt.push_str(&addition);
    }
    prompt
}

const PUNCT: &[char] = &[
    '.', ',', '!', '?', ';', ':', '…', '"', '„', '“', '\'', '(', ')',
];

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
            DictionaryEntry {
                spoken: "гит хъб".into(),
                written: "GitHub".into(),
            },
            DictionaryEntry {
                spoken: "софт про нео".into(),
                written: "SoftProNeo".into(),
            },
            DictionaryEntry {
                spoken: "гласопис".into(),
                written: "Glasopis".into(),
            },
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
    fn prompt_lists_the_preferred_spellings() {
        let prompt = build_prompt(&dict(), "");
        assert!(prompt.contains("GitHub"));
        assert!(prompt.contains("SoftProNeo"));
        assert!(prompt.contains("Glasopis"));
    }

    #[test]
    fn prompt_takes_extra_terms_and_deduplicates() {
        let prompt = build_prompt(&dict(), "dev, npm, github, React");
        assert!(prompt.contains("dev"));
        assert!(prompt.contains("npm"));
        assert!(prompt.contains("React"));
        assert_eq!(
            prompt.to_lowercase().matches("github").count(),
            1,
            "повтореният термин влиза веднъж: {prompt}"
        );
    }

    #[test]
    fn prompt_is_empty_without_input() {
        assert_eq!(build_prompt(&Dictionary::default(), "   "), "");
    }

    #[test]
    fn prompt_is_capped() {
        let long = (0..200)
            .map(|i| format!("термин{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        assert!(build_prompt(&Dictionary::default(), &long).len() <= 600);
    }

    #[test]
    fn empty_dictionary_is_identity() {
        assert_eq!(
            Dictionary::default().apply("нищо не се сменя"),
            "нищо не се сменя"
        );
    }

    #[test]
    fn does_not_match_inside_words() {
        assert_eq!(dict().apply("гласописец"), "гласописец");
    }
}
