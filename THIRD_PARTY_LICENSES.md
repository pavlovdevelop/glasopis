# Third-party licenses

Glasopis itself is MIT licensed (see [LICENSE](LICENSE), Copyright (c) 2026 Pavel Pavlov).
It is built on the following third-party work. Every component below is free of charge and
free to redistribute as part of an MIT-licensed application; none of them requires an account,
an API key or a subscription.

Versions are the ones pinned when v0.1.0 was prepared; run `cargo metadata` and `npm ls` for the
exact set in your checkout.

## Speech recognition

| Component | Version | License | Notes |
| --- | --- | --- | --- |
| [whisper.cpp](https://github.com/ggml-org/whisper.cpp) | vendored by whisper-rs-sys | MIT | C/C++ inference engine, statically linked |
| [ggml](https://github.com/ggml-org/ggml) | part of whisper.cpp | MIT | tensor library used by whisper.cpp |
| [whisper-rs](https://codeberg.org/tazz4843/whisper-rs) | 0.16.0 | Unlicense (public domain) | Rust bindings |
| whisper-rs-sys | 0.15.0 | Unlicense (public domain) | build/FFI layer |

### Speech models

Glasopis downloads OpenAI Whisper weights converted to the ggml format and published by the
whisper.cpp project:

| Model | File | License |
| --- | --- | --- |
| Whisper small (q5_1) | `ggml-small-q5_1.bin` | MIT (OpenAI Whisper weights) |
| Whisper large-v3-turbo (q5_0) | `ggml-large-v3-turbo-q5_0.bin` | MIT (OpenAI Whisper weights) |
| Whisper medium | `ggml-medium.bin` | MIT (OpenAI Whisper weights) |

OpenAI released the Whisper models and code under the MIT license, which permits local use and
redistribution. The model files are **not** bundled with Glasopis and are **not** committed to
this repository; they are downloaded by the user from
`https://huggingface.co/ggerganov/whisper.cpp` and verified against a SHA-256 checksum.

## Rust dependencies

| Crate | Version | License |
| --- | --- | --- |
| tauri | 2.11 | Apache-2.0 OR MIT |
| tauri-plugin-global-shortcut | 2.3 | Apache-2.0 OR MIT |
| tauri-plugin-autostart | 2.5 | Apache-2.0 OR MIT |
| tauri-plugin-single-instance | 2.4 | Apache-2.0 OR MIT |
| tauri-plugin-opener | 2.5 | Apache-2.0 OR MIT |
| tauri-plugin-log | 2.9 | Apache-2.0 OR MIT |
| cpal | 0.17 | Apache-2.0 |
| reqwest | 0.13 | MIT OR Apache-2.0 |
| native-tls (Schannel on Windows) | 0.2 | MIT OR Apache-2.0 |
| sha2 | 0.10 | MIT OR Apache-2.0 |
| windows | 0.62 | MIT OR Apache-2.0 |
| serde, serde_json | 1.x | MIT OR Apache-2.0 |
| parking_lot | 0.12 | MIT OR Apache-2.0 |
| anyhow, thiserror, log | 1.x / 2.x | MIT OR Apache-2.0 |

Transitive dependencies of the above carry permissive licenses (MIT, Apache-2.0, BSD, ISC,
Zlib, Unicode-3.0 or Unlicense).

## Frontend dependencies

| Package | Version | License |
| --- | --- | --- |
| react, react-dom | 19 | MIT |
| @tauri-apps/api, @tauri-apps/cli | 2 | MIT OR Apache-2.0 |
| vite, @vitejs/plugin-react | 7 / 5 | MIT |
| vitest | 3 | MIT |
| typescript | 5 | Apache-2.0 |

## Windows components

| Component | License |
| --- | --- |
| Microsoft Edge WebView2 Runtime | [Microsoft Developer Services Agreement / redistributable terms](https://developer.microsoft.com/microsoft-edge/webview2/) — redistributed by the Tauri NSIS installer, not by this repository |

## Fonts and assets

No third-party fonts are bundled; the interface uses the Segoe UI family already present on
Windows, falling back to the system UI font. The application icon in `src-tauri/icons/` was
generated for this project and is covered by the project's MIT license.
