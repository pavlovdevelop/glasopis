//! Windows clipboard access (CF_UNICODETEXT).
//!
//! UTF-16 text on the clipboard is what makes Glasopis independent of the
//! active keyboard layout: Cyrillic arrives in the target application exactly
//! as it was recognised, even when the user has an English layout selected.

use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
    SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};
use windows::Win32::System::Ole::CF_UNICODETEXT;

use crate::errors::{GlasopisError, Result};

/// The clipboard is a shared, single-owner resource: another application may
/// hold it for a few milliseconds (clipboard managers are notorious for this),
/// so opening it is retried before giving up.
const OPEN_ATTEMPTS: u32 = 10;
const OPEN_RETRY_MS: u64 = 20;

fn with_clipboard<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    let mut last_error = None;
    for attempt in 0..OPEN_ATTEMPTS {
        match unsafe { OpenClipboard(None) } {
            Ok(()) => {
                let result = f();
                unsafe {
                    let _ = CloseClipboard();
                }
                return result;
            }
            Err(err) => {
                last_error = Some(err);
                if attempt + 1 < OPEN_ATTEMPTS {
                    std::thread::sleep(std::time::Duration::from_millis(OPEN_RETRY_MS));
                }
            }
        }
    }
    Err(GlasopisError::other(format!(
        "Клипбордът е зает от друго приложение ({}).",
        last_error
            .map(|e| e.to_string())
            .unwrap_or_else(|| "неизвестна грешка".into())
    )))
}

/// Writes UTF-16 text to the clipboard.
pub fn write_text(text: &str) -> Result<()> {
    let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes = std::mem::size_of_val(utf16.as_slice());

    with_clipboard(|| unsafe {
        EmptyClipboard().map_err(|e| {
            GlasopisError::other(format!("Клипбордът не може да бъде изчистен: {e}"))
        })?;

        let handle: HGLOBAL = GlobalAlloc(GMEM_MOVEABLE, bytes)
            .map_err(|e| GlasopisError::other(format!("Недостатъчна памет за клипборда: {e}")))?;

        let ptr = GlobalLock(handle) as *mut u16;
        if ptr.is_null() {
            let _ = GlobalFree(Some(handle));
            return Err(GlasopisError::other(
                "Паметта за клипборда не може да бъде заключена.".to_string(),
            ));
        }
        std::ptr::copy_nonoverlapping(utf16.as_ptr(), ptr, utf16.len());
        // GlobalUnlock reports "failure" when the lock count reaches zero,
        // which is the normal case here.
        let _ = GlobalUnlock(handle);

        match SetClipboardData(CF_UNICODETEXT.0 as u32, Some(HANDLE(handle.0))) {
            Ok(_) => Ok(()), // Windows now owns the memory.
            Err(err) => {
                let _ = GlobalFree(Some(handle));
                Err(GlasopisError::other(format!(
                    "Текстът не можа да бъде поставен в клипборда: {err}"
                )))
            }
        }
    })
}

/// Reads the current clipboard text, if it holds any.
pub fn read_text() -> Result<Option<String>> {
    with_clipboard(|| unsafe {
        if IsClipboardFormatAvailable(CF_UNICODETEXT.0 as u32).is_err() {
            return Ok(None);
        }
        let handle = match GetClipboardData(CF_UNICODETEXT.0 as u32) {
            Ok(handle) => handle,
            Err(_) => return Ok(None),
        };
        let global = HGLOBAL(handle.0);
        let ptr = GlobalLock(global) as *const u16;
        if ptr.is_null() {
            return Ok(None);
        }
        let capacity = GlobalSize(global) / std::mem::size_of::<u16>();
        let slice = std::slice::from_raw_parts(ptr, capacity);
        let len = slice.iter().position(|unit| *unit == 0).unwrap_or(capacity);
        let text = String::from_utf16_lossy(&slice[..len]);
        let _ = GlobalUnlock(global);
        Ok(Some(text))
    })
}
