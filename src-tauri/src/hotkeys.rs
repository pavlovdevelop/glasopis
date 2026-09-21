//! Global hotkey registration.
//!
//! Two independent shortcuts:
//!
//! * **toggle** - press once to start, once more to stop (the default
//!   `Ctrl+Alt+Space`);
//! * **push-to-talk** - record while the combination is held down.
//!
//! Both are configurable. Windows registers global hotkeys as combinations, so
//! a bare modifier key (Right Ctrl on its own) cannot be used; the default
//! push-to-talk combination is therefore `Ctrl+Alt+D`.

use glasopis_core::settings::Settings;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::dictation;
use crate::errors::{GlasopisError, Result};

/// Registers the hotkeys from the settings, replacing any previous ones.
pub fn register_all(app: &AppHandle, settings: &Settings) -> Result<()> {
    let shortcuts = app.global_shortcut();
    if let Err(err) = shortcuts.unregister_all() {
        log::warn!("предишните клавишни комбинации не бяха освободени: {err}");
    }

    let toggle = settings.hotkeys.toggle.trim().to_string();
    if !toggle.is_empty() {
        shortcuts
            .on_shortcut(toggle.as_str(), move |app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    dictation::toggle(app);
                }
            })
            .map_err(|err| {
                GlasopisError::other(format!(
                    "Клавишната комбинация „{toggle}“ не може да бъде регистрирана: {err}"
                ))
            })?;
    }

    let ptt = settings.hotkeys.push_to_talk.trim().to_string();
    if settings.hotkeys.push_to_talk_enabled
        && !ptt.is_empty()
        && ptt != settings.hotkeys.toggle.trim()
    {
        shortcuts
            .on_shortcut(ptt.as_str(), move |app, _shortcut, event| {
                match event.state {
                    ShortcutState::Pressed => dictation::start(app),
                    ShortcutState::Released => dictation::stop(app),
                }
            })
            .map_err(|err| {
                GlasopisError::other(format!(
                    "Комбинацията за задържане „{ptt}“ не може да бъде регистрирана: {err}"
                ))
            })?;
    }
    Ok(())
}

/// Checks whether an accelerator can be used, without registering it.
pub fn is_valid_accelerator(accelerator: &str) -> bool {
    use std::str::FromStr;
    tauri_plugin_global_shortcut::Shortcut::from_str(accelerator).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_hotkeys_are_valid() {
        let settings = Settings::default();
        assert!(is_valid_accelerator(&settings.hotkeys.toggle));
        assert!(is_valid_accelerator(&settings.hotkeys.push_to_talk));
    }

    #[test]
    fn nonsense_is_rejected() {
        assert!(!is_valid_accelerator("не е комбинация"));
        assert!(!is_valid_accelerator(""));
    }
}
