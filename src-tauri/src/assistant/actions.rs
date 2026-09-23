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

#[cfg(windows)]
mod imp {
    use windows::core::HSTRING;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    use crate::errors::{GlasopisError, Result};

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
}

#[cfg(not(windows))]
mod imp {
    use crate::errors::{GlasopisError, Result};

    pub fn open(_exe: &str) -> Result<()> {
        Err(GlasopisError::WindowsOnly)
    }
}
