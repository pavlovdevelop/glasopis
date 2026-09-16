//! System-wide text insertion.
//!
//! Glasopis never integrates with a specific application. It inserts text the
//! way Windows itself does it:
//!
//! 1. **Clipboard** — put the text on the clipboard as UTF-16 and send
//!    `Ctrl+V`. This is the primary strategy because it is independent of the
//!    active keyboard layout, which is exactly what Bulgarian Cyrillic needs.
//! 2. **Keyboard simulation** — send the text as Unicode key events
//!    (`KEYEVENTF_UNICODE`). Used as a fallback and for applications that do
//!    not accept `Ctrl+V`.

use glasopis_core::settings::InjectionMode;

use crate::errors::Result;

#[cfg(windows)]
pub mod clipboard;
#[cfg(windows)]
pub mod focus;
#[cfg(windows)]
pub mod keyboard;

/// A way of getting text into the focused application.
pub trait TextInjector {
    fn name(&self) -> &'static str;
    fn insert_text(&self, text: &str) -> Result<()>;
}

/// What actually happened, so the UI can tell the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectionOutcome {
    /// The text was inserted into the focused application.
    Inserted,
    /// Insertion failed but the text is on the clipboard, ready for Ctrl+V.
    ClipboardOnly,
}

#[derive(Debug, Clone, Copy)]
pub struct InjectionOptions {
    pub mode: InjectionMode,
    pub restore_clipboard: bool,
    pub restore_delay_ms: u64,
}

#[cfg(windows)]
mod imp {
    use super::*;
    use crate::errors::GlasopisError;

    /// Pastes through the clipboard.
    pub struct ClipboardInjector {
        pub restore: bool,
        pub restore_delay_ms: u64,
    }

    impl TextInjector for ClipboardInjector {
        fn name(&self) -> &'static str {
            "clipboard"
        }

        fn insert_text(&self, text: &str) -> Result<()> {
            let previous = if self.restore {
                clipboard::read_text().ok().flatten()
            } else {
                None
            };

            clipboard::write_text(text)?;
            keyboard::send_paste()?;

            if let Some(previous) = previous {
                let delay = self.restore_delay_ms;
                // Restoring too early would make the target application paste
                // the old content, so it happens on a background thread after
                // the target had time to process Ctrl+V.
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(delay));
                    if let Err(err) = clipboard::write_text(&previous) {
                        log::warn!("клипбордът не беше възстановен: {err}");
                    }
                });
            }
            Ok(())
        }
    }

    /// Types the text with Unicode key events.
    pub struct KeyboardInjector;

    impl TextInjector for KeyboardInjector {
        fn name(&self) -> &'static str {
            "keyboard"
        }

        fn insert_text(&self, text: &str) -> Result<()> {
            keyboard::type_text(text)
        }
    }

    fn run(injector: &dyn TextInjector, text: &str) -> Result<()> {
        log::debug!("въвеждам текста чрез стратегия „{}“", injector.name());
        injector.insert_text(text)
    }

    /// Inserts `text` into the application that currently has focus.
    pub fn insert(text: &str, options: InjectionOptions) -> Result<InjectionOutcome> {
        if text.is_empty() {
            return Ok(InjectionOutcome::Inserted);
        }

        let clipboard_injector = ClipboardInjector {
            restore: options.restore_clipboard,
            restore_delay_ms: options.restore_delay_ms,
        };

        let result = match options.mode {
            InjectionMode::Clipboard => run(&clipboard_injector, text),
            InjectionMode::Keyboard => run(&KeyboardInjector, text),
            InjectionMode::Automatic => match run(&clipboard_injector, text) {
                Ok(()) => Ok(()),
                Err(err) => {
                    log::warn!(
                        "поставянето през клипборда не успя ({err}); опитвам клавиатурна симулация"
                    );
                    run(&KeyboardInjector, text)
                }
            },
        };

        match result {
            Ok(()) => Ok(InjectionOutcome::Inserted),
            Err(err) => {
                log::error!("въвеждането на текст не успя: {err}");
                // Never lose the text: leave it on the clipboard.
                match clipboard::write_text(text) {
                    Ok(()) => Ok(InjectionOutcome::ClipboardOnly),
                    Err(clipboard_err) => {
                        log::error!("текстът не можа да бъде копиран: {clipboard_err}");
                        Err(GlasopisError::InjectionFailed)
                    }
                }
            }
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use super::*;
    use crate::errors::GlasopisError;

    /// Text insertion is a Windows feature; on other platforms the rest of the
    /// application still builds so the core logic can be tested.
    pub fn insert(_text: &str, _options: InjectionOptions) -> Result<InjectionOutcome> {
        Err(GlasopisError::WindowsOnly)
    }
}

pub use imp::insert;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_can_be_built_from_settings() {
        let settings = glasopis_core::settings::Settings::default();
        let options = InjectionOptions {
            mode: settings.injection.mode,
            restore_clipboard: settings.injection.restore_clipboard,
            restore_delay_ms: settings.injection.restore_clipboard_delay_ms,
        };
        assert_eq!(options.mode, InjectionMode::Automatic);
        assert!(options.restore_clipboard);
        assert!(options.restore_delay_ms >= 300);
    }
}
