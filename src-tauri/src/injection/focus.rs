//! Remembering and restoring the window that the user was typing in.
//!
//! Glasopis shows a floating overlay while recording. Even though that overlay
//! is created without activation, the safest approach is to remember the
//! foreground window *before* anything of Glasopis appears and to put the focus
//! back there right before the text is inserted.

use windows::core::{BOOL, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM};
use windows::Win32::System::Threading::{
    AttachThreadInput, GetCurrentThreadId, OpenProcess, QueryFullProcessImageNameW,
    PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindow, GetWindowTextW, GetWindowThreadProcessId,
    IsIconic, IsWindow, IsWindowVisible, SetForegroundWindow, ShowWindow, GW_OWNER, SW_RESTORE,
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

/// The executable file name (e.g. `"chrome.exe"`) that owns `hwnd`, or `None`
/// if it cannot be determined - a window closing mid-lookup is normal, not
/// an error worth surfacing.
fn owning_exe_name(hwnd: HWND) -> Option<String> {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    if pid == 0 {
        return None;
    }
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let mut buffer = [0u16; 260];
    let mut len = buffer.len() as u32;
    let result = unsafe {
        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut len,
        )
    };
    unsafe {
        let _ = CloseHandle(process);
    }
    result.ok()?;
    let path = String::from_utf16_lossy(&buffer[..len as usize]);
    path.rsplit(['\\', '/'])
        .next()
        .map(|name| name.to_lowercase())
}

struct WindowSearch {
    exe_name: String,
    found: Option<HWND>,
}

unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let search = unsafe { &mut *(lparam.0 as *mut WindowSearch) };
    let is_top_level = match unsafe { GetWindow(hwnd, GW_OWNER) } {
        Ok(owner) => owner.0.is_null(),
        Err(_) => true,
    };
    if is_top_level
        && unsafe { IsWindowVisible(hwnd) }.as_bool()
        && owning_exe_name(hwnd).as_deref() == Some(search.exe_name.as_str())
    {
        search.found = Some(hwnd);
        return BOOL(0); // Stop enumeration - a match was found.
    }
    BOOL(1) // Keep looking.
}

/// Finds a visible, top-level window belonging to `exe_name` (e.g.
/// `"chrome.exe"`) and brings it to the foreground. Used by the assistant's
/// whitelisted actions that must type into a *specific* application, rather
/// than whatever currently has focus.
pub fn activate_process_window(exe_name: &str) -> bool {
    let mut search = WindowSearch {
        exe_name: exe_name.to_lowercase(),
        found: None,
    };
    let _ = unsafe {
        EnumWindows(
            Some(enum_window_proc),
            LPARAM(std::ptr::addr_of_mut!(search) as isize),
        )
    };
    match search.found {
        Some(hwnd) => TargetWindow(hwnd.0 as isize).restore_focus(),
        None => false,
    }
}
