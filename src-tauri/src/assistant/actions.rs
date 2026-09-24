//! Executes a [`glasopis_core::assistant::ProposedAction`] - and only that.
//!
//! This is the one place in the assistant flow that actually touches the
//! system. It never receives free-form text from the AI model: only a
//! `&'static KnownApp` already validated against the whitelist in
//! `glasopis_core::assistant`.

use glasopis_core::assistant::KnownApp;

use crate::errors::Result;

/// Opens `app.exe` and returns the Bulgarian confirmation message to show
/// and speak.
pub fn open_app(app: &KnownApp) -> Result<String> {
    imp::open(app.exe)?;
    Ok(format!("Отворих {}.", app.label_bg))
}

/// Opens Chrome (if it is not already running), focuses its address bar and
/// searches for `query`. `query` is free text from the AI model, but it is
/// only ever *typed* - never interpreted as a command, path or shell
/// argument - the same guarantee dictation already gives the rest of the
/// system.
pub fn search_chrome(query: &str) -> Result<String> {
    imp::search_chrome(query)?;
    Ok(format!("Търся \"{query}\" в Chrome."))
}

#[cfg(windows)]
mod imp {
    use windows::core::HSTRING;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    use crate::errors::{GlasopisError, Result};
    use crate::injection::{focus, keyboard};

    /// How long to wait for Chrome's window to appear after a cold start,
    /// before giving up on typing into it.
    const WINDOW_WAIT_ATTEMPTS: u32 = 15;
    const WINDOW_WAIT_DELAY_MS: u64 = 200;

    /// `exe` is always one of the hardcoded `KnownApp::exe` values - a bare
    /// executable name, never anything derived from what the user said or
    /// what the AI model returned. Windows resolves it through the registry
    /// "App Paths" the same way the Start menu / Run dialog do.
    pub fn open(exe: &str) -> Result<()> {
        let operation = HSTRING::from("open");
        let file = HSTRING::from(exe);
        // SAFETY: all arguments are valid, owned HSTRINGs kept alive for the
        // call; `ShellExecuteW` does not retain them past return.
        let result = unsafe { ShellExecuteW(None, &operation, &file, None, None, SW_SHOWNORMAL) };
        // ShellExecuteW returns a pseudo-HINSTANCE; values <= 32 are error codes.
        if (result.0 as usize) <= 32 {
            return Err(GlasopisError::other(format!(
                "Windows не можа да отвори {exe}."
            )));
        }
        Ok(())
    }

    /// Window activation succeeding does not mean the browser is done
    /// rendering and ready to react to `Ctrl+L` - especially true for a
    /// freshly-launched, still cold-starting process. A short settle pause
    /// here is what actually made `Ctrl+L` reliably land on the address bar
    /// in practice, rather than a keystroke or two vanishing into a window
    /// that was foreground but not yet interactive.
    const SETTLE_AFTER_ACTIVATION_MS: u64 = 300;

    pub fn search_chrome(query: &str) -> Result<()> {
        open("chrome.exe")?;
        if !wait_for_chrome_window() {
            return Err(GlasopisError::other(
                "Chrome не се отвори навреме.".to_string(),
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(SETTLE_AFTER_ACTIVATION_MS));
        keyboard::send_focus_address_bar()?;
        keyboard::type_text(query)?;
        keyboard::send_enter()?;
        Ok(())
    }

    /// Polls for Chrome's window and brings it to the foreground - a fresh
    /// launch needs a moment before the window (and therefore keyboard
    /// input) exists at all.
    fn wait_for_chrome_window() -> bool {
        for _ in 0..WINDOW_WAIT_ATTEMPTS {
            if focus::activate_process_window("chrome.exe") {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(WINDOW_WAIT_DELAY_MS));
        }
        false
    }
}

#[cfg(not(windows))]
mod imp {
    use crate::errors::{GlasopisError, Result};

    pub fn open(_exe: &str) -> Result<()> {
        Err(GlasopisError::WindowsOnly)
    }

    pub fn search_chrome(_query: &str) -> Result<()> {
        Err(GlasopisError::WindowsOnly)
    }
}
