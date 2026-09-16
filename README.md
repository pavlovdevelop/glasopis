# 🇧🇬 Glasopis

**Говориш. То пише.**

Free and open-source system-wide Bulgarian voice typing for Windows.

Speak Bulgarian and type anywhere.

[Български README](README.bg.md) · [Architecture](docs/architecture.md) · [Building](docs/building.md) · [Privacy](docs/privacy.md)

---

Glasopis is a small Windows utility that lives in the system tray. Put the cursor in *any*
text field — Chrome, Word, Outlook, VS Code, Discord, an ERP form — press a global hotkey,
speak Bulgarian, and the recognized text is typed into that field. There is no per-app
integration: Glasopis inserts text the way Windows itself does.

Speech recognition runs **locally** with [whisper.cpp](https://github.com/ggml-org/whisper.cpp).
Your voice never leaves your computer.

## 100% Free

Glasopis is designed to remain completely free for everyday use.

- No subscription
- No API key
- No paid speech recognition
- No cloud required
- No usage limits
- No account required
- Local/offline speech recognition

Once a speech model is downloaded, Glasopis performs its core voice-typing functionality
locally on your computer. Internet is only needed to download the application and a model.

## Screenshots

> Screenshots are added with the first tagged release.

| Settings | Floating microphone | Model manager |
| --- | --- | --- |
| _screenshots/settings.png_ | _screenshots/overlay.png_ | _screenshots/models.png_ |

## Features

- **System-wide voice typing** — works in any Windows application that accepts keyboard input.
- **Bulgarian first** — `bg-BG` is the default language; the interface is Bulgarian by default.
- **Layout independent Cyrillic** — text is inserted as Unicode, so Cyrillic arrives correctly
  even when the active Windows keyboard layout is English.
- **Two recording modes** — toggle (`Ctrl+Alt+Space`) and push-to-talk (`Ctrl+Alt+D`).
- **Bulgarian voice commands** — „точка“, „запетая“, „въпросителен знак“, „нов ред“,
  „нов параграф“, „изтрий последната дума“, „изтрий последното изречение“ and more.
- **Automatic punctuation** from the speech model, with the voice commands as an explicit override.
- **Personal dictionary** — map what you say to how it should be written (`гит хъб` → `GitHub`).
- **Model manager** — download, verify (SHA-256), select and delete speech models.
- **Floating microphone overlay** with a live level meter that never steals keyboard focus.
- **Private by design** — no telemetry, no accounts, history off by default, audio kept in memory.

## Installation

1. Download `GlasopisSetup.exe` from the [Releases](https://github.com/pavlovdevelop/glasopis/releases) page.
2. Run it. The installer does not require administrator rights (per-user install).
3. On first launch the onboarding wizard helps you pick a microphone, download a speech model
   and confirm the hotkey.

> Community builds are **not code-signed** (a certificate costs money, and Glasopis is free).
> Windows SmartScreen may therefore show "Windows protected your PC" — choose
> *More info → Run anyway*. You can always build from source instead.

## Usage

1. Click into any text field.
2. Press `Ctrl + Alt + Space`.
3. Speak Bulgarian. The floating window shows `Слушам...`.
4. Press `Ctrl + Alt + Space` again.
5. Glasopis transcribes locally (`Обработвам...`) and inserts the text where your cursor was.

Glasopis never presses Enter for you and never executes what you dictate — it only inserts text.

### Keyboard shortcuts

| Action | Default | Configurable |
| --- | --- | --- |
| Start / stop dictation | `Ctrl + Alt + Space` | yes |
| Push-to-talk (hold) | `Ctrl + Alt + D` (off by default) | yes |

Windows registers global hotkeys as *combinations*, so a bare modifier (Right Ctrl alone)
cannot be used as a push-to-talk key.

## Supported Windows versions

- Windows 11 x64 (primary target)
- Windows 10 x64 (version 1809 or newer, with WebView2 installed — the installer adds it)

## Privacy

- Speech is recognized locally by whisper.cpp. Nothing is uploaded.
- Recordings live in memory only and are released right after recognition.
- Dictation history is **off** by default and never stores audio.
- No telemetry, no accounts, no ads. See [docs/privacy.md](docs/privacy.md).

## Offline usage

Internet is used for exactly two things: downloading Glasopis and downloading a speech model.
After that, dictation works with the network cable unplugged.

## Building from source

```bash
git clone https://github.com/pavlovdevelop/glasopis.git
cd glasopis
npm install
npm run tauri dev      # development
npm run tauri build    # Windows installer in src-tauri/target/release/bundle/nsis/
```

Prerequisites (Windows): Git, Node.js 20+, Rust (MSVC toolchain), Visual Studio Build Tools
with the C++ workload, CMake, WebView2. Full instructions — including what can be built and
tested on Linux — are in [docs/building.md](docs/building.md).

## Project architecture

```text
glasopis/
├── src/                     React + TypeScript UI (settings, onboarding, overlay)
├── crates/glasopis-core/    platform independent logic (settings, commands, models, audio math)
├── src-tauri/               Windows application: audio, speech, injection, tray, hotkeys
└── docs/                    architecture, privacy, building, QA, roadmap
```

The details — and why each decision was made — are in [docs/architecture.md](docs/architecture.md).

## Contributing

Pull requests are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) and run
`cargo fmt`, `cargo clippy`, `cargo test` and `npm test` before opening one.

## Roadmap

Planned, in rough order: streaming transcription, richer editing commands, personal dictionary UI,
optional GPU acceleration, English UI polish, dictation history search. See
[docs/roadmap.md](docs/roadmap.md). Cloud or paid AI processing will always remain optional and
will never replace the free local mode.

## License

MIT © 2026 Pavel Pavlov. See [LICENSE](LICENSE) and
[THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).
