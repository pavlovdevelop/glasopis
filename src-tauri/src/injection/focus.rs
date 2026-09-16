//! Remembering and restoring the window that the user was typing in.
//!
//! Glasopis shows a floating overlay while recording. Even though that overlay
//! is created without activation, the safest approach is to remember the
//! foreground window *before* anything of Glasopis appears and to put the focus
//! back there right before the text is inserted.

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindow,
    SetForegroundWindow, ShowWindow, SW_RESTORE,
};

/// A remembered window handle. Stored as `isize` so it can be moved between
/// threads (`HWND` is a raw pointer and therefore not `Send`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetWindow(pub isize);

impl TargetWindow {
    fn hwnd(self) -> HWND {
        HWND(self.0 as *mut std::ffi::c_void)
    }

    pub fn is_valid(self) -> bool {
        self.0 != 0 && unsafe { IsWindow(Some(self.hwnd())) }.as_bool()
    }

    /// Window title, for the logs and the overlay.
    pub fn title(self) -> Option<String> {
        if !self.is_valid() {
            return None;
        }
        let mut buffer = [0u16; 256];
        let len = unsafe { GetWindowTextW(self.hwnd(), &mut buffer) };
        if len <= 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buffer[..len as usize]))
    }

    /// Brings the remembered window back to the foreground.
    ///
    /// Windows only allows the foreground thread to change the foreground
    /// window, so the usual `AttachThreadInput` trick is used.
    pub fn restore_focus(self) -> bool {
        if !self.is_valid() {
            return false;
        }
        let hwnd = self.hwnd();
        unsafe {
            if GetForegroundWindow() == hwnd {
                return true;
            }
            if IsIconic(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            }
            let target_thread = GetWindowThreadProcessId(hwnd, None);
            let current_thread = GetCurrentThreadId();
            let attached = target_thread != 0
                && target_thread != current_thread
                && AttachThreadInput(current_thread, target_thread, true).as_bool();

            let ok = SetForegroundWindow(hwnd).as_bool();

            if attached {
                let _ = AttachThreadInput(current_thread, target_thread, false);
            }
            ok
        }
    }
}

/// The window that currently has focus.
pub fn current_foreground_window() -> Option<TargetWindow> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return None;
    }
    Some(TargetWindow(hwnd.0 as isize))
}
