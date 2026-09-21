//! Optional local dictation history.
//!
//! Disabled by default. Only the recognised text and a timestamp are stored -
//! never audio. The file lives next to the settings in the application data
//! folder and can be cleared from the settings window.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Unix timestamp in seconds.
    pub timestamp: u64,
    pub text: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct History {
    pub entries: Vec<HistoryEntry>,
}

impl History {
    /// Adds an entry, keeping the newest first and trimming to `limit`.
    pub fn push(&mut self, entry: HistoryEntry, limit: usize) {
        if entry.text.trim().is_empty() {
            return;
        }
        self.entries.insert(0, entry);
        if limit > 0 && self.entries.len() > limit {
            self.entries.truncate(limit);
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn load_or_default(path: &std::path::Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|raw| Self::from_json(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save_to(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = self
            .to_json()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(ts: u64, text: &str) -> HistoryEntry {
        HistoryEntry {
            timestamp: ts,
            text: text.to_string(),
        }
    }

    #[test]
    fn newest_entry_is_first() {
        let mut h = History::default();
        h.push(entry(1, "първо"), 10);
        h.push(entry(2, "второ"), 10);
        assert_eq!(h.entries[0].text, "второ");
    }

    #[test]
    fn limit_is_respected() {
        let mut h = History::default();
        for i in 0..10 {
            h.push(entry(i, &format!("текст {i}")), 3);
        }
        assert_eq!(h.entries.len(), 3);
        assert_eq!(h.entries[0].text, "текст 9");
    }

    #[test]
    fn empty_text_is_not_stored() {
        let mut h = History::default();
        h.push(entry(1, "   "), 10);
        assert!(h.entries.is_empty());
    }

    #[test]
    fn clearing_removes_everything() {
        let mut h = History::default();
        h.push(entry(1, "нещо"), 10);
        h.clear();
        assert!(h.entries.is_empty());
    }

    #[test]
    fn json_round_trip() {
        let mut h = History::default();
        h.push(entry(42, "Здравей"), 10);
        assert_eq!(History::from_json(&h.to_json().unwrap()).unwrap(), h);
    }
}
