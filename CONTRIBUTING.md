# Contributing to Glasopis

Благодарим! / Thank you! Contributions in Bulgarian or English are equally welcome.

## Ground rules

Glasopis must stay **free of charge and honest about what it does**. Recognition may run in the
cloud (the default engine is Groq) or locally (whisper.cpp), but:

- the local path must keep working — a change that makes local-only use impossible is not
  acceptable;
- nothing in the product may require payment, and a provider that costs money to use for normal
  dictation does not belong in the default build;
- whenever audio leaves the machine, the interface and the documentation must say so plainly.

Two more rules that follow from the product:

- No telemetry, no analytics, no crash reporting without explicit opt-in.
- Recognized text is inserted, never executed. Glasopis does not press Enter for the user.

## Getting started

See [docs/building.md](docs/building.md) for the toolchain. In short:

```bash
npm install
npm run tauri dev
```

Most logic that does not need Windows lives in `crates/glasopis-core` and can be developed and
tested on any platform.

## Before opening a pull request

```bash
cargo fmt --all
cargo clippy --target x86_64-pc-windows-msvc --workspace -- -D warnings   # or the host target on Windows
cargo test --workspace
npm run typecheck
npm test
npm run build
```

Please also:

- Add tests for anything in `glasopis-core` — that crate is where the text behaviour lives.
- Keep user-visible strings in `src/i18n/bg.json` **and** `src/i18n/en.json`; a test enforces
  that both files have the same keys.
- Write Bulgarian error messages for anything the user can see.
- Explain *why* in the commit message when a change is not obvious.

## Reporting bugs

Open an issue with:

- your Windows version and whether it is 10 or 11,
- the target application (Chrome, Word, ...),
- the selected model and microphone,
- what you dictated and what appeared,
- the relevant part of `%LOCALAPPDATA%\com.glasopis.app\logs\` (it contains no transcripts).

## Adding a speech model

Models must be free to download, usable without an account or key, runnable locally, and
compatible with redistribution of Glasopis. Add the entry to `crates/glasopis-core/src/models.rs`
with the exact size and SHA-256 checksum, and record the license in
[THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).

## Code of conduct

By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).
