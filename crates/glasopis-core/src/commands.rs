//! Bulgarian voice command processing.
//!
//! The processor takes the raw transcript produced by the speech engine and
//! turns spoken commands ("нов ред", "запетая", "изтрий последната дума") into
//! the corresponding text operations. It is intentionally decoupled from the
//! speech engine: its input is a plain string and its output is a plain string.

use std::collections::HashMap;

/// A single text operation produced by the command processor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// A normal word of the transcript.
    Word(String),
    /// Punctuation glued to the previous word, e.g. `.` `,` `?`.
    AttachedPunctuation(&'static str),
    /// A symbol that is separated by spaces, e.g. a dash.
    SpacedSymbol(&'static str),
    /// An opening symbol: space before, no space after (e.g. `(`).
    OpeningSymbol(&'static str),
    /// A single line break.
    NewLine,
    /// An empty line (new paragraph).
    NewParagraph,
    /// Remove the last word written so far.
    DeleteLastWord,
    /// Remove the last sentence written so far.
    DeleteLastSentence,
}

/// The command table: spoken phrase (already lower-cased, punctuation stripped)
/// mapped to the action it triggers.
///
/// Longer phrases are matched before shorter ones, so `точка и запетая` wins
/// over `точка`.
pub fn bulgarian_commands() -> Vec<(&'static str, Action)> {
    vec![
        // --- punctuation -------------------------------------------------
        ("точка и запетая", Action::AttachedPunctuation(";")),
        ("въпросителен знак", Action::AttachedPunctuation("?")),
        ("въпросителна", Action::AttachedPunctuation("?")),
        ("удивителен знак", Action::AttachedPunctuation("!")),
        ("удивителна", Action::AttachedPunctuation("!")),
        ("удивителен", Action::AttachedPunctuation("!")),
        ("многоточие", Action::AttachedPunctuation("…")),
        ("двоеточие", Action::AttachedPunctuation(":")),
        ("запетая", Action::AttachedPunctuation(",")),
        ("запетайка", Action::AttachedPunctuation(",")),
        ("точка", Action::AttachedPunctuation(".")),
        // --- symbols -----------------------------------------------------
        ("тире", Action::SpacedSymbol("–")),
        ("отваряща скоба", Action::OpeningSymbol("(")),
        ("затваряща скоба", Action::AttachedPunctuation(")")),
        ("отваряща кавичка", Action::OpeningSymbol("„")),
        ("затваряща кавичка", Action::AttachedPunctuation("“")),
        ("процент", Action::AttachedPunctuation("%")),
        // --- layout ------------------------------------------------------
        ("нов параграф", Action::NewParagraph),
        ("нов абзац", Action::NewParagraph),
        ("нов ред", Action::NewLine),
        ("следващ ред", Action::NewLine),
        // --- editing -----------------------------------------------------
        ("изтрий последната дума", Action::DeleteLastWord),
        ("изтрий последното изречение", Action::DeleteLastSentence),
    ]
}

/// Characters that may surround a token as "decoration" produced by the
/// automatic punctuation of the speech model.
const TRIM: &[char] = &[
    '.', ',', '!', '?', ';', ':', '…', '"', '„', '“', '\'', '(', ')', '-', '–', '—',
];

/// Normalises a token before it is compared with the command table.
fn key(token: &str) -> String {
    token.trim_matches(TRIM).to_lowercase()
}

struct Table {
    /// phrase length (in words) -> map of joined phrase -> action
    by_len: Vec<(usize, HashMap<String, Action>)>,
    max_len: usize,
}

impl Table {
    fn build(entries: Vec<(&'static str, Action)>) -> Self {
        let mut grouped: HashMap<usize, HashMap<String, Action>> = HashMap::new();
        let mut max_len = 1;
        for (phrase, action) in entries {
            let words: Vec<String> = phrase.split_whitespace().map(key).collect();
            let len = words.len();
            max_len = max_len.max(len);
            grouped
                .entry(len)
                .or_default()
                .insert(words.join(" "), action);
        }
        let mut by_len: Vec<(usize, HashMap<String, Action>)> = grouped.into_iter().collect();
        by_len.sort_by(|a, b| b.0.cmp(&a.0)); // longest first
        Self { by_len, max_len }
    }

    fn lookup(&self, tokens: &[&str], at: usize) -> Option<(usize, Action)> {
        for (len, map) in &self.by_len {
            if at + len > tokens.len() {
                continue;
            }
            let candidate = tokens[at..at + len]
                .iter()
                .map(|t| key(t))
                .collect::<Vec<_>>()
                .join(" ");
            if let Some(action) = map.get(&candidate) {
                return Some((*len, action.clone()));
            }
        }
        let _ = self.max_len;
        None
    }
}

/// Splits a raw transcript into a stream of [`Action`]s.
///
/// When `enabled` is `false` every token is emitted as a plain word, which is
/// what the user gets when voice commands are switched off in the settings.
pub fn parse(transcript: &str, enabled: bool) -> Vec<Action> {
    let tokens: Vec<&str> = transcript.split_whitespace().collect();
    if !enabled {
        return tokens
            .iter()
            .map(|t| Action::Word((*t).to_string()))
            .collect();
    }
    let table = Table::build(bulgarian_commands());
    let mut out = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        match table.lookup(&tokens, i) {
            Some((len, action)) => {
                out.push(action);
                i += len;
            }
            None => {
                out.push(Action::Word(tokens[i].to_string()));
                i += 1;
            }
        }
    }
    out
}

/// Applies a stream of actions to a string buffer.
pub fn apply(actions: &[Action]) -> String {
    let mut buf = String::new();
    for action in actions {
        match action {
            Action::Word(w) => {
                push_spaced(&mut buf, w);
            }
            Action::AttachedPunctuation(p) => {
                trim_trailing_spaces(&mut buf);
                // The speech model often already wrote punctuation for the
                // spoken command itself ("Здравей, запетая, как си"), so drop a
                // duplicate before adding the requested one.
                while buf.ends_with(['.', ',', ';', ':', '!', '?', '…']) {
                    buf.pop();
                }
                buf.push_str(p);
            }
            Action::SpacedSymbol(s) => push_spaced(&mut buf, s),
            Action::OpeningSymbol(s) => {
                push_spaced(&mut buf, s);
            }
            Action::NewLine => {
                trim_trailing_spaces(&mut buf);
                buf.push('\n');
            }
            Action::NewParagraph => {
                trim_trailing_spaces(&mut buf);
                while buf.ends_with('\n') {
                    buf.pop();
                }
                if !buf.is_empty() {
                    buf.push_str("\n\n");
                }
            }
            Action::DeleteLastWord => delete_last_word(&mut buf),
            Action::DeleteLastSentence => delete_last_sentence(&mut buf),
        }
    }
    buf
}

fn push_spaced(buf: &mut String, word: &str) {
    let needs_space = match buf.chars().last() {
        None => false,
        Some(c) => !matches!(c, ' ' | '\n' | '(' | '„'),
    };
    if needs_space {
        buf.push(' ');
    }
    buf.push_str(word);
}

fn trim_trailing_spaces(buf: &mut String) {
    while buf.ends_with(' ') {
        buf.pop();
    }
}

fn delete_last_word(buf: &mut String) {
    while buf.ends_with(|c: char| c.is_whitespace()) {
        buf.pop();
    }
    while !buf.is_empty() && !buf.ends_with(|c: char| c.is_whitespace()) {
        buf.pop();
    }
    trim_trailing_spaces(buf);
}

fn delete_last_sentence(buf: &mut String) {
    while buf.ends_with(|c: char| c.is_whitespace() || matches!(c, '.' | '!' | '?' | '…')) {
        buf.pop();
    }
    while !buf.is_empty() && !buf.ends_with(['.', '!', '?', '…']) {
        buf.pop();
    }
    trim_trailing_spaces(buf);
}

/// Convenience wrapper: parse + apply.
pub fn process(transcript: &str, enabled: bool) -> String {
    apply(&parse(transcript, enabled))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_is_untouched() {
        assert_eq!(process("Здравей как си", true), "Здравей как си");
    }

    #[test]
    fn punctuation_commands() {
        assert_eq!(
            process("Здравей запетая как си въпросителен знак", true),
            "Здравей, как си?"
        );
        assert_eq!(process("добре точка", true), "добре.");
        assert_eq!(process("супер удивителен знак", true), "супер!");
    }

    #[test]
    fn semicolon_wins_over_full_stop() {
        assert_eq!(process("едно точка и запетая две", true), "едно; две");
    }

    #[test]
    fn model_punctuation_around_command_is_ignored() {
        // Whisper often writes the spoken command with its own punctuation.
        assert_eq!(process("Здравей, запетая, как си", true), "Здравей, как си");
    }

    #[test]
    fn new_line_and_paragraph() {
        assert_eq!(process("първи ред нов ред втори ред", true), "първи ред\nвтори ред");
        assert_eq!(
            process("първи абзац нов параграф втори абзац", true),
            "първи абзац\n\nвтори абзац"
        );
    }

    #[test]
    fn brackets_and_quotes() {
        assert_eq!(
            process("тест отваряща скоба нещо затваряща скоба", true),
            "тест (нещо)"
        );
        assert_eq!(
            process("той каза отваряща кавичка здравей затваряща кавичка", true),
            "той каза „здравей“"
        );
    }

    #[test]
    fn delete_commands() {
        assert_eq!(process("едно две изтрий последната дума три", true), "едно три");
        assert_eq!(
            process("Първо изречение точка Второ изречение точка изтрий последното изречение", true),
            "Първо изречение."
        );
    }

    #[test]
    fn commands_can_be_disabled() {
        assert_eq!(process("Здравей запетая как си", false), "Здравей запетая как си");
    }

    #[test]
    fn dash_is_spaced() {
        assert_eq!(process("едно тире две", true), "едно – две");
    }
}
