# Implementation plan

The plan the project was built against, with the current status of each phase. It is kept
up to date so anyone can see what is finished, what is only type-checked and what is left.

## Library choices

| Need | Choice | Why |
| --- | --- | --- |
| Desktop shell | Tauri 2 + React 19 + TypeScript + Vite | small installer, WebView2 already on Windows, Rust backend |
| Speech recognition | whisper.cpp via `whisper-rs` 0.16 | free, local, good Bulgarian, no API key, static linking |
| Audio capture | `cpal` 0.17 | maintained, WASAPI backend, device enumeration |
| Global hotkeys | `tauri-plugin-global-shortcut` 2 | press *and* release events → push-to-talk |
| Autostart | `tauri-plugin-autostart` 2 | per-user registry entry, no admin rights |
| Single instance | `tauri-plugin-single-instance` 2 | focuses the running instance |
| Model download | `reqwest` (blocking, Schannel TLS) + `sha2` | no OpenSSL, no extra build tooling |
| Windows APIs | `windows` 0.62 | clipboard, SendInput, foreground window |

Rejected: every cloud speech API (paid and/or key-required), Electron (size), `rdev` for
push-to-talk (a global keyboard hook is heavier than the hotkey plugin and looks like a keylogger
to antivirus software).

## Phases

| # | Phase | Status |
| --- | --- | --- |
| 1 | Foundation: workspace, Tauri app, tray, single instance, settings storage, global hotkey | done |
| 2 | Audio: device enumeration, selection, recording, level metering, 16 kHz mono pipeline | done |
| 3 | Speech: whisper.cpp integration, model loading/caching, background transcription | done |
| 4 | Windows text injection: remember focus, clipboard paste, Unicode typing, fallback | done |
| 5 | Floating overlay: listening / processing / done / error states, level meter | done |
| 6 | Model manager: catalogue, download with progress, SHA-256 verification, delete, select | done |
| 7 | Settings window: general, voice, hotkeys, insertion, privacy, history, about | done |
| 8 | Bulgarian commands: punctuation, line breaks, deletions, normalisation, dictionary | done |
| 9 | Onboarding: five steps from welcome to a test dictation | done |
| 10 | Packaging: NSIS installer, per-user install, release workflow | workflow written, needs a Windows run |
| 11 | GitHub readiness: READMEs, licenses, CI, QA checklist | done |

## What is verified, and how

- `cargo test -p glasopis-core` — 52 tests covering the command processor, the dictionary, the
  settings format and migration, the model catalogue, the history and the audio maths. Runs on
  Linux and Windows.
- `npm test` — frontend tests for the translation files, the hotkey parser and formatting.
- `cargo clippy --target x86_64-pc-windows-msvc --no-default-features --workspace -- -D warnings`
  — type-checks the Windows-specific code (clipboard, SendInput, tray, hotkeys) without a C++
  toolchain. This is what the Linux part of CI runs.
- The Windows CI job compiles the complete application **with** whisper.cpp and runs the backend
  tests. All three CI jobs (frontend, core/cross-check, Windows) pass on the current commit.

## What still needs a real Windows desktop

Compiling is not the same as working. Before tagging `v0.1.0`, the manual checklist in
[QA.md](QA.md) must be run on Windows: insertion into Notepad, Chrome, Word, VS Code and a
terminal; Cyrillic with both keyboard layouts; autostart across a reboot; microphone
disconnection; elevated-window failure handling; installer and uninstaller.
