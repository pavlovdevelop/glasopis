//! Simulated keyboard input (`SendInput`).

use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_CONTROL, VK_L, VK_LWIN, VK_MENU, VK_RETURN, VK_RWIN,
    VK_SHIFT, VK_V,
};

use crate::errors::{GlasopisError, Result};

/// Small pause that gives the target application time to process the input.
const AFTER_PASTE_MS: u64 = 30;

fn key_input(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if up {
                    KEYEVENTF_KEYUP
                } else {
                    Default::default()
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn unicode_input(unit: u16, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: unit,
                dwFlags: if up {
                    KEYEVENTF_UNICODE | KEYEVENTF_KEYUP
                } else {
                    KEYEVENTF_UNICODE
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send(inputs: &[INPUT]) -> Result<()> {
    let sent = unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent as usize != inputs.len() {
        // The usual cause is a target window running elevated while Glasopis
        // is not (UIPI blocks the input).
        return Err(GlasopisError::other(
            "Windows блокира въвеждането в активното приложение (вероятно работи като администратор).".to_string(),
        ));
    }
    Ok(())
}

fn is_down(vk: VIRTUAL_KEY) -> bool {
    (unsafe { GetKeyState(vk.0 as i32) } as u16 & 0x8000) != 0
}

/// Releases modifiers the user may still be holding (the dictation hotkey
/// itself, most of the time) so they do not turn `Ctrl+V` into something else.
fn release_stuck_modifiers() {
    let mut inputs = Vec::new();
    for vk in [VK_MENU, VK_SHIFT, VK_LWIN, VK_RWIN] {
        if is_down(vk) {
            inputs.push(key_input(vk, true));
        }
    }
    if !inputs.is_empty() {
        let _ = send(&inputs);
    }
}

/// Sends `Ctrl+V` to the focused window.
pub fn send_paste() -> Result<()> {
    release_stuck_modifiers();
    // Make sure Ctrl starts from a known state.
    if is_down(VK_CONTROL) {
        let _ = send(&[key_input(VK_CONTROL, true)]);
    }
    send(&[
        key_input(VK_CONTROL, false),
        key_input(VK_V, false),
        key_input(VK_V, true),
        key_input(VK_CONTROL, true),
    ])?;
    std::thread::sleep(std::time::Duration::from_millis(AFTER_PASTE_MS));
    Ok(())
}

/// Sends `Ctrl+L`, which focuses the address/search bar in every major
/// browser (Chrome, Edge, Firefox) - a known, predictable place to type a
/// search query into, rather than guessing where a search box is on the page.
pub fn send_focus_address_bar() -> Result<()> {
    release_stuck_modifiers();
    if is_down(VK_CONTROL) {
        let _ = send(&[key_input(VK_CONTROL, true)]);
    }
    send(&[
        key_input(VK_CONTROL, false),
        key_input(VK_L, false),
        key_input(VK_L, true),
        key_input(VK_CONTROL, true),
    ])?;
    std::thread::sleep(std::time::Duration::from_millis(AFTER_PASTE_MS));
    Ok(())
}

/// Sends `Enter`.
pub fn send_enter() -> Result<()> {
    send(&[key_input(VK_RETURN, false), key_input(VK_RETURN, true)])?;
    std::thread::sleep(std::time::Duration::from_millis(AFTER_PASTE_MS));
    Ok(())
}

/// Types `text` as Unicode key events.
///
/// This works regardless of the active keyboard layout because the characters
/// are sent as UTF-16 code units, not as scan codes.
pub fn type_text(text: &str) -> Result<()> {
    release_stuck_modifiers();
    // Chunked so a long dictation does not overflow the input queue.
    const CHUNK: usize = 64;
    let mut inputs: Vec<INPUT> = Vec::with_capacity(CHUNK * 2);

    for ch in text.chars() {
        if ch == '\n' {
            inputs.push(key_input(VK_RETURN, false));
            inputs.push(key_input(VK_RETURN, true));
        } else if ch == '\r' {
            continue;
        } else {
            let mut buffer = [0u16; 2];
            for unit in ch.encode_utf16(&mut buffer) {
                inputs.push(unicode_input(*unit, false));
                inputs.push(unicode_input(*unit, true));
            }
        }
        if inputs.len() >= CHUNK * 2 {
            send(&inputs)?;
            inputs.clear();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    if !inputs.is_empty() {
        send(&inputs)?;
    }
    Ok(())
}
