//! The voice assistant: turns a spoken command into a whitelisted action.
//!
//! Deliberately narrow by design (see docs/architecture.md and SECURITY.md):
//! the AI model never picks a command to run — it may only pick a `target`
//! from [`KNOWN_APPS`], and even that is re-validated here rather than
//! trusted at face value. Anything else comes back as [`ProposedAction::Unknown`],
//! which the assistant flow turns into "не разбрах" and does nothing.

use serde::{Deserialize, Serialize};

/// An application the assistant is allowed to open.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KnownApp {
    /// Stable id; this is the only thing the AI model is allowed to return.
    pub id: &'static str,
    /// Shown in the settings UI.
    pub label_bg: &'static str,
    /// Passed to `ShellExecuteW` — a bare executable name Windows resolves
    /// through the "App Paths" registry, not a hardcoded install path.
    pub exe: &'static str,
    /// Spoken forms a user might say, given to the AI model as context.
    pub aliases: &'static [&'static str],
}

/// The starting whitelist. Deliberately small: every entry here is something
/// `ShellExecuteW` can open by name alone, with no arguments and no shell
/// interpretation of user-controlled text.
pub const KNOWN_APPS: &[KnownApp] = &[
    KnownApp {
        id: "chrome",
        label_bg: "Chrome",
        exe: "chrome.exe",
        aliases: &["chrome", "хром", "гугъл хром", "google chrome"],
    },
    KnownApp {
        id: "notepad",
        label_bg: "Бележник",
        exe: "notepad.exe",
        aliases: &["бележник", "notepad", "тефтерче"],
    },
    KnownApp {
        id: "explorer",
        label_bg: "Файлове",
        exe: "explorer.exe",
        aliases: &["файлове", "проводник", "explorer", "файловия мениджър"],
    },
    KnownApp {
        id: "calculator",
        label_bg: "Калкулатор",
        exe: "calc.exe",
        aliases: &["калкулатор", "калкулатора"],
    },
    KnownApp {
        id: "word",
        label_bg: "Word",
        exe: "winword.exe",
        aliases: &["word", "уърд"],
    },
    KnownApp {
        id: "excel",
        label_bg: "Excel",
        exe: "excel.exe",
        aliases: &["excel", "ексел"],
    },
    KnownApp {
        id: "outlook",
        label_bg: "Outlook",
        exe: "outlook.exe",
        aliases: &["outlook", "аутлук", "пощата"],
    },
];

pub fn find_known_app(id: &str) -> Option<&'static KnownApp> {
    KNOWN_APPS.iter().find(|app| app.id == id)
}

/// Longest search query the assistant will type - long enough for a real
/// sentence, short enough that a model going off the rails cannot turn this
/// into a way to type arbitrary long text through a "search".
const MAX_SEARCH_QUERY_LEN: usize = 200;

/// What the AI model proposed, already validated against [`KNOWN_APPS`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposedAction {
    /// Open `app`; `question` is what the assistant should ask before
    /// actually doing it (spoken and/or shown to the user).
    OpenApp {
        app: &'static KnownApp,
        question: String,
    },
    /// Open Chrome (if needed), focus its address bar and search for `query`;
    /// `question` is what the assistant should ask before actually doing it.
    SearchChrome { query: String, question: String },
    /// The model could not match the command to anything on the whitelist,
    /// or its response could not be parsed/validated.
    Unknown,
}

impl ProposedAction {
    /// The confirmation question to ask before acting, or `None` for
    /// [`ProposedAction::Unknown`] - there is nothing to confirm.
    pub fn question(&self) -> Option<&str> {
        match self {
            ProposedAction::OpenApp { question, .. } => Some(question),
            ProposedAction::SearchChrome { question, .. } => Some(question),
            ProposedAction::Unknown => None,
        }
    }
}

/// The system prompt sent to the AI model — instructs it to answer with a
/// single JSON object, and lists exactly the app ids it is allowed to use.
pub fn build_system_prompt() -> String {
    let mut apps = String::new();
    for app in KNOWN_APPS {
        apps.push_str(&format!(
            "- \"{}\" ({}, изговаряно и като: {})\n",
            app.id,
            app.label_bg,
            app.aliases.join(", ")
        ));
    }
    format!(
        "Ти превеждаш ЕДНА гласова команда на потребителя в JSON обект, описващ \
действие на компютъра. Отговори САМО с валиден JSON, без обяснения.\n\n\
Ако командата означава \"отвори <приложение>\" и приложението е в списъка по-долу, отговори:\n\
{{\"action\": \"open_app\", \"target\": \"<точното id от списъка>\", \"question\": \"<кратък \
въпрос на български, питащ дали да отвориш точно това приложение>\"}}\n\n\
Ако командата означава \"потърси/провери в Chrome/Google <нещо>\", отговори:\n\
{{\"action\": \"search_chrome\", \"query\": \"<кратък текст за търсене>\", \"question\": \"<кратък \
въпрос на български, питащ дали да потърсиш точно това>\"}}\n\n\
Ако командата не съвпада с нищо познато, е неясна, или иска нещо друго (напр. писане на \
произволен текст, кликане, промяна на настройки), отговори:\n\
{{\"action\": \"unknown\"}}\n\n\
Позволени приложения (target трябва да е точно едно от тези id стойности):\n{apps}"
    )
}

#[derive(Debug, Deserialize)]
struct RawResponse {
    action: String,
    target: Option<String>,
    query: Option<String>,
    question: Option<String>,
}

/// Parses and validates the AI model's raw JSON reply. Never trusts `target`
/// without checking it against [`KNOWN_APPS`] — a model that hallucinates an
/// unknown id, or returns anything that is not valid JSON, becomes
/// [`ProposedAction::Unknown`] rather than an error.
pub fn parse_model_response(raw: &str) -> ProposedAction {
    let Ok(parsed) = serde_json::from_str::<RawResponse>(raw.trim()) else {
        return ProposedAction::Unknown;
    };
    match parsed.action.as_str() {
        "open_app" => {
            let Some(target) = parsed.target.as_deref() else {
                return ProposedAction::Unknown;
            };
            let Some(app) = find_known_app(target) else {
                return ProposedAction::Unknown;
            };
            let question = parsed
                .question
                .filter(|q| !q.trim().is_empty())
                .unwrap_or_else(|| format!("Да отворя ли {}?", app.label_bg));
            ProposedAction::OpenApp { app, question }
        }
        "search_chrome" => {
            let query = parsed
                .query
                .map(|q| q.trim().to_string())
                .filter(|q| !q.is_empty());
            let Some(query) = query else {
                return ProposedAction::Unknown;
            };
            // A model going off the rails should not be able to turn "search
            // for X" into typing an arbitrarily long block of text.
            let query: String = query.chars().take(MAX_SEARCH_QUERY_LEN).collect();
            let question = parsed
                .question
                .filter(|q| !q.trim().is_empty())
                .unwrap_or_else(|| format!("Да потърся ли \"{query}\" в Chrome?"));
            ProposedAction::SearchChrome { query, question }
        }
        _ => ProposedAction::Unknown,
    }
}

/// A spoken yes/no reply to the assistant's confirmation question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confirmation {
    Yes,
    No,
    /// Neither a clear yes nor a clear no; treated as "no" (never execute on
    /// an ambiguous reply).
    Unclear,
}

// Two-letter words ("да", "не") are matched exactly — as a *prefix* they
// would also match unrelated words ("нещо" starts with "не"). Longer stems
// are matched as a prefix so conjugated forms ("разбирам" / "разбира се")
// still count.
const AFFIRMATIVE_EXACT: &[&str] = &["да", "ок", "окей"];
const AFFIRMATIVE_PREFIX: &[&str] = &[
    "давай",
    "добре",
    "разбира",
    "потвържда",
    "точно",
    "именно",
    "хайде",
];
const NEGATIVE_EXACT: &[&str] = &["не"];
const NEGATIVE_PREFIX: &[&str] = &["недей", "спри", "отказ", "чакай", "нищо"];

fn word_matches(word: &str, exact: &[&str], prefix: &[&str]) -> bool {
    exact.contains(&word) || prefix.iter().any(|stem| word.starts_with(stem))
}

/// Classifies a short transcript as yes/no by whole-word match (never a
/// substring match — "нещо" must not trigger on "не").
pub fn classify_confirmation(text: &str) -> Confirmation {
    let words: Vec<String> = text
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect();

    let has_negative = words
        .iter()
        .any(|w| word_matches(w, NEGATIVE_EXACT, NEGATIVE_PREFIX));
    if has_negative {
        return Confirmation::No;
    }
    let has_affirmative = words
        .iter()
        .any(|w| word_matches(w, AFFIRMATIVE_EXACT, AFFIRMATIVE_PREFIX));
    if has_affirmative {
        return Confirmation::Yes;
    }
    Confirmation::Unclear
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_app_ids_are_unique() {
        let mut ids: Vec<&str> = KNOWN_APPS.iter().map(|a| a.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), KNOWN_APPS.len());
    }

    #[test]
    fn every_known_app_has_aliases() {
        for app in KNOWN_APPS {
            assert!(!app.aliases.is_empty(), "{} has no aliases", app.id);
        }
    }

    #[test]
    fn valid_open_app_response_is_parsed() {
        let raw = r#"{"action":"open_app","target":"chrome","question":"Да отворя ли Chrome?"}"#;
        match parse_model_response(raw) {
            ProposedAction::OpenApp { app, question } => {
                assert_eq!(app.id, "chrome");
                assert_eq!(question, "Да отворя ли Chrome?");
            }
            other => panic!("expected OpenApp, got {other:?}"),
        }
    }

    #[test]
    fn open_app_without_question_gets_a_default_one() {
        let raw = r#"{"action":"open_app","target":"notepad"}"#;
        match parse_model_response(raw) {
            ProposedAction::OpenApp { question, .. } => assert!(question.contains("Бележник")),
            other => panic!("expected OpenApp, got {other:?}"),
        }
    }

    #[test]
    fn valid_search_chrome_response_is_parsed() {
        let raw = r#"{"action":"search_chrome","query":"котки","question":"Да потърся ли котки?"}"#;
        match parse_model_response(raw) {
            ProposedAction::SearchChrome { query, question } => {
                assert_eq!(query, "котки");
                assert_eq!(question, "Да потърся ли котки?");
            }
            other => panic!("expected SearchChrome, got {other:?}"),
        }
    }

    #[test]
    fn search_chrome_without_question_gets_a_default_one() {
        let raw = r#"{"action":"search_chrome","query":"времето"}"#;
        match parse_model_response(raw) {
            ProposedAction::SearchChrome { question, .. } => assert!(question.contains("времето")),
            other => panic!("expected SearchChrome, got {other:?}"),
        }
    }

    #[test]
    fn search_chrome_without_a_query_is_unknown() {
        assert_eq!(
            parse_model_response(r#"{"action":"search_chrome"}"#),
            ProposedAction::Unknown
        );
        assert_eq!(
            parse_model_response(r#"{"action":"search_chrome","query":"   "}"#),
            ProposedAction::Unknown
        );
    }

    #[test]
    fn an_overlong_search_query_is_truncated_not_rejected() {
        let long_query = "а".repeat(500);
        let raw = format!(r#"{{"action":"search_chrome","query":"{long_query}"}}"#);
        match parse_model_response(&raw) {
            ProposedAction::SearchChrome { query, .. } => {
                assert_eq!(query.chars().count(), MAX_SEARCH_QUERY_LEN);
            }
            other => panic!("expected SearchChrome, got {other:?}"),
        }
    }

    #[test]
    fn unknown_action_is_unknown() {
        assert_eq!(
            parse_model_response(r#"{"action":"unknown"}"#),
            ProposedAction::Unknown
        );
    }

    #[test]
    fn a_hallucinated_target_is_rejected() {
        let raw = r#"{"action":"open_app","target":"delete_system32"}"#;
        assert_eq!(parse_model_response(raw), ProposedAction::Unknown);
    }

    #[test]
    fn garbage_is_unknown_not_an_error() {
        assert_eq!(
            parse_model_response("не е JSON изобщо"),
            ProposedAction::Unknown
        );
        assert_eq!(parse_model_response(""), ProposedAction::Unknown);
    }

    #[test]
    fn yes_no_are_classified_by_whole_word() {
        assert_eq!(classify_confirmation("да, давай"), Confirmation::Yes);
        assert_eq!(classify_confirmation("не, недей"), Confirmation::No);
        assert_eq!(classify_confirmation("добре звучи"), Confirmation::Yes);
    }

    #[test]
    fn unrelated_speech_is_unclear_not_yes() {
        assert_eq!(
            classify_confirmation("нещо съвсем друго"),
            Confirmation::Unclear
        );
        assert_eq!(classify_confirmation(""), Confirmation::Unclear);
    }

    #[test]
    fn similar_looking_words_do_not_false_positive() {
        // "нещо" (something) starts with "не" (no) as a raw substring, but
        // must not be classified as a negative reply.
        assert_eq!(classify_confirmation("нещо"), Confirmation::Unclear);
    }

    #[test]
    fn negative_wins_when_both_appear() {
        // Safety default: an ambiguous "да, не всъщност" must never execute.
        assert_eq!(classify_confirmation("да, не всъщност"), Confirmation::No);
    }
}
