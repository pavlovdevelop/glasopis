# Architecture

Glasopis is a Tauri 2 desktop application. The code is split into three layers:

```text
┌─────────────────────────────────────────────────────────────┐
│ src/               React + TypeScript                       │
│                    settings, onboarding, floating overlay   │
├─────────────────────────────────────────────────────────────┤
│ src-tauri/         Rust, Windows specific                   │
│                    audio · speech · injection · hotkeys     │
│                    tray · model manager · settings storage  │
├─────────────────────────────────────────────────────────────┤
│ crates/glasopis-core/  Rust, platform independent           │
│                    settings model · Bulgarian commands      │
│                    dictionary · model catalogue · audio math│
└─────────────────────────────────────────────────────────────┘
```

`glasopis-core` exists so the parts that decide what text the user gets can be unit tested on
any platform, including the Linux CI runner. It has no dependency on Tauri, Windows or a
sound card.

## The dictation flow

```text
hotkey (Ctrl+Alt+Space)
  └─ remember the foreground window        injection::focus
  └─ start recording                       audio::recorder (cpal, own thread)
       └─ level events to the overlay      glasopis://level
hotkey again
  └─ stop recording → Vec<f32> 16 kHz mono audio helpers in glasopis-core
  └─ background worker thread
       ├─ whisper.cpp transcription        speech::whisper_engine
       ├─ personal dictionary              core::dictionary
       ├─ Bulgarian voice commands         core::commands
       ├─ normalisation / capitalisation   core::text
       ├─ optional history entry           history_store
       ├─ restore the remembered window    injection::focus
       └─ insert the text                  injection::clipboard / keyboard
```

Nothing heavy happens on the UI thread: recording lives on its own thread (a WASAPI stream is
not `Send`), and transcription runs on a second one. The windows only receive events.

## Key decisions

### Why Tauri 2 instead of Electron or WinUI

A tray utility should be small and cheap to run. Tauri uses the WebView2 runtime that Windows
already has, which keeps the installer in the tens of megabytes and the idle memory low, while
the backend stays plain Rust — where the Windows APIs we need (SendInput, clipboard,
foreground window) are one call away.

### Why whisper.cpp through `whisper-rs`

Bulgarian support in free, locally executable speech recognition is effectively Whisper.
whisper.cpp runs Whisper on the CPU with no Python, no CUDA and no service. `whisper-rs`
(Unlicense) is a maintained binding and links the library statically, so the installer ships one
executable. The trade-off is a C/C++ toolchain requirement when building from source.

No cloud backend exists in the code. The `SpeechEngine` trait would allow another local engine
later; it is not a hook for a paid API.

### Why the clipboard is the primary insertion strategy

`Ctrl+V` pastes UTF-16 text. That makes the result independent of the active Windows keyboard
layout, which is the whole point for Cyrillic: with an English layout active, sending scan codes
would produce Latin letters. The previous clipboard content is read before pasting and restored
on a timer afterwards (default 800 ms) so the target application has time to process the paste.

Keyboard simulation (`KEYEVENTF_UNICODE`) is the fallback and can also be selected explicitly for
applications that do not accept `Ctrl+V`.

### Why the foreground window is remembered before anything is shown

The overlay is created without activation, but a floating window is still a window. Glasopis
stores the `HWND` that had focus when the hotkey was pressed and calls `SetForegroundWindow`
(with the usual `AttachThreadInput` dance) right before inserting, so the text lands where the
user was typing.

### Why voice commands are a separate module

The command processor takes a `&str` and returns a `String`. It knows nothing about Whisper,
audio or Windows, which makes the entire text behaviour of the product testable:
`cargo test -p glasopis-core` covers punctuation, new lines, deletions, the dictionary and the
normalisation rules.

### Why models are downloaded instead of bundled

A usable multilingual model is 180 MB – 1.5 GB. Bundling one would make the installer enormous
and would force everyone to take the same accuracy/speed trade-off. The model manager downloads
from the whisper.cpp repository on Hugging Face (no account, no key), verifies the SHA-256
checksum recorded in `glasopis-core::models`, and stores the file in
`%LOCALAPPDATA%\Glasopis\models\`. A download URL that does not start with the known host is
rejected.

## Data locations

| What | Where |
| --- | --- |
| Settings | `%APPDATA%\com.glasopis.app\settings.json` |
| History (opt-in) | `%APPDATA%\com.glasopis.app\history.json` |
| Speech models | `%LOCALAPPDATA%\com.glasopis.app\models\` |
| Logs | `%LOCALAPPDATA%\com.glasopis.app\logs\` |

Settings are written atomically (temporary file + rename) and every field has a default, so a
file from an older version keeps working and a damaged file falls back to the defaults instead of
blocking startup.

## Threads

| Thread | Purpose |
| --- | --- |
| main | Tauri event loop, tray, windows |
| `glasopis-recorder` | owns the cpal input stream for the duration of one dictation |
| `glasopis-level` | pushes the input level to the overlay every 60 ms |
| `glasopis-transcribe` | whisper.cpp + text pipeline + insertion |
| `glasopis-model-download` | one per model download |

The speech model is kept loaded between dictations (`EngineHolder`) and released when the user
selects a different model, changes the thread count or deletes the file.

## Known limitations (v0.1.0)

- Text cannot be inserted into an application running elevated unless Glasopis is elevated too
  (Windows UIPI blocks the input). The text stays on the clipboard and the user is told.
- A bare modifier key cannot be a global hotkey, so push-to-talk uses a combination.
- Recognition is batch, not streaming: the text appears after you stop speaking.
- Recording is capped at 5 minutes per dictation.
- whisper.cpp runs in-process, so a hard failure inside it (an unsupported CPU instruction, an
  out-of-memory abort with a large model) terminates Glasopis instead of showing an error. This
  is why release builds must keep `GGML_NATIVE=OFF` (see docs/building.md). Running the engine
  in a separate process would make this recoverable and is on the roadmap.
- Backend error messages are Bulgarian even when the interface language is English.
- `нов ред` inside a single dictation is inserted as a line break in the text; the text is then
  pasted as a whole, so applications that submit on Enter are unaffected.
