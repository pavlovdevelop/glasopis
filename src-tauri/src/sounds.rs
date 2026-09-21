//! Short feedback tones for the start and the end of a dictation.
//!
//! Deliberately tiny: no audio files are shipped and no playback library is
//! pulled in - the two tones come from the Windows `Beep` API and are played on
//! a background thread so they never delay recording.

#[cfg(windows)]
pub fn play_start() {
    beep(880, 70);
}

#[cfg(windows)]
pub fn play_stop() {
    beep(520, 70);
}

#[cfg(windows)]
fn beep(frequency: u32, duration_ms: u32) {
    std::thread::spawn(move || unsafe {
        use windows::Win32::System::Diagnostics::Debug::Beep;
        let _ = Beep(frequency, duration_ms);
    });
}

#[cfg(not(windows))]
pub fn play_start() {}

#[cfg(not(windows))]
pub fn play_stop() {}
