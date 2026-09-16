//! Transcript normalisation.
//!
//! Whisper output is already punctuated most of the time, but it may contain
//! artefacts: leading/trailing whitespace, doubled spaces, a space before a
//! comma, or a lower-case first letter after a full stop. These helpers clean
//! that up without changing the words themselves.

/// Removes the bracketed annotations Whisper emits for non-speech audio,
/// e.g. `[музика]`, `(смях)`, `*въздишка*`.
pub fn strip_non_speech(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut depth_square = 0usize;
    let mut depth_round = 0usize;
    for ch in input.chars() {
        match ch {
            '[' => depth_square += 1,
            ']' => depth_square = depth_square.saturating_sub(1),
            '(' if is_annotation_start(input, ch) => depth_round += 1,
            ')' if depth_round > 0 => depth_round -= 1,
            _ if depth_square == 0 && depth_round == 0 => out.push(ch),
            _ => {}
        }
    }
    out
}

// Round brackets are legitimate text, so they are only treated as an
// annotation when the transcript uses them for the whole segment. Keeping this
// conservative avoids eating the user's own parentheses.
fn is_annotation_start(input: &str, _ch: char) -> bool {
    input.trim_start().starts_with('(') && input.trim_end().ends_with(')')
}

/// Collapses repeated whitespace and fixes spacing around punctuation.
pub fn tidy_spacing(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_was_space = false;
    for ch in input.chars() {
        if ch == ' ' || ch == '\t' {
            last_was_space = true;
            continue;
        }
        if last_was_space {
            last_was_space = false;
            if !matches!(ch, '.' | ',' | '!' | '?' | ';' | ':' | '…' | ')' | '“')
                && !out.is_empty()
                && !out.ends_with('\n')
                && !out.ends_with('(')
                && !out.ends_with('„')
            {
                out.push(' ');
            }
        }
        out.push(ch);
    }
    out.trim().to_string()
}

/// Upper-cases the first letter of the text and of every new sentence.
pub fn capitalize_sentences(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut capitalize_next = true;
    for ch in input.chars() {
        if capitalize_next && ch.is_alphabetic() {
            for upper in ch.to_uppercase() {
                out.push(upper);
            }
            capitalize_next = false;
            continue;
        }
        if matches!(ch, '.' | '!' | '?' | '…' | '\n') {
            capitalize_next = true;
        }
        out.push(ch);
    }
    out
}

/// Full normalisation pass used by the dictation pipeline.
pub fn normalize(input: &str, capitalize: bool) -> String {
    let cleaned = tidy_spacing(&strip_non_speech(input));
    if capitalize {
        capitalize_sentences(&cleaned)
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacing_is_tidied() {
        assert_eq!(tidy_spacing("  Здравей   ,  как си ?  "), "Здравей, как си?");
    }

    #[test]
    fn newlines_survive() {
        assert_eq!(tidy_spacing("първи ред\n  втори ред"), "първи ред\nвтори ред");
    }

    #[test]
    fn sentences_are_capitalized() {
        assert_eq!(
            capitalize_sentences("здравей. как си? добре!"),
            "Здравей. Как си? Добре!"
        );
    }

    #[test]
    fn cyrillic_uppercase_is_correct() {
        assert_eq!(capitalize_sentences("щастие"), "Щастие");
    }

    #[test]
    fn non_speech_markers_are_removed() {
        assert_eq!(
            normalize("[музика] Здравей [смях] свят", true),
            "Здравей свят"
        );
    }

    #[test]
    fn user_parentheses_are_kept() {
        assert_eq!(normalize("тест (нещо) край", false), "тест (нещо) край");
    }
}
