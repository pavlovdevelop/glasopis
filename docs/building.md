# Building Glasopis

## Windows (the real thing)

Glasopis is a Windows application. A full build — the one that produces
`GlasopisSetup.exe` — must happen on Windows.

### Prerequisites

| Tool | Notes |
| --- | --- |
| [Git](https://git-scm.com/) | |
| [Node.js 20+](https://nodejs.org/) | ships npm |
| [Rust](https://rustup.rs/) | the default `x86_64-pc-windows-msvc` toolchain |
| [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) | workload *Desktop development with C++* — required by Rust and by whisper.cpp |
| [CMake](https://cmake.org/download/) | whisper.cpp is built with CMake; "Add to PATH" |
| WebView2 Runtime | already present on Windows 11 and updated Windows 10; otherwise [download it](https://developer.microsoft.com/microsoft-edge/webview2/) |

### Build

```bash
git clone https://github.com/pavlovdevelop/glasopis.git
cd glasopis
npm install
npm run tauri dev
```

The first `tauri dev` compiles whisper.cpp, which takes a few minutes. Later builds are cached.

Release build with installer:

```bash
npm run tauri build
```

Output:

```text
target/release/Glasopis.exe
target/release/bundle/nsis/Glasopis_0.1.0_x64-setup.exe
```

The paths are in `target/` at the root of the repository, not in `src-tauri/target/`, because
Glasopis is a Cargo workspace and a workspace shares one build directory.

The release workflow renames the installer to `GlasopisSetup.exe` before publishing it.

## Linux / macOS (partial)

You cannot build the Windows application on Linux, but you can work on — and verify — a large
part of the project:

```bash
npm install
npm run typecheck        # TypeScript
npm test                 # frontend unit tests
npm run build            # production frontend bundle

cargo fmt --all --check
cargo test -p glasopis-core          # settings, commands, dictionary, models, audio math
cargo clippy -p glasopis-core -- -D warnings
```

The Windows-only Rust code can still be type-checked without a C/C++ toolchain by compiling for
the Windows target with the speech engine feature disabled:

```bash
rustup target add x86_64-pc-windows-msvc
cargo clippy --target x86_64-pc-windows-msvc --no-default-features --workspace -- -D warnings
```

`--no-default-features` turns off the `whisper` feature. Such a build is **not** a usable
application: transcription returns an explicit error saying the speech engine was not compiled
in. It exists purely as a type-checking aid; every release build has the feature on.

## Portable binaries

`.cargo/config.toml` sets `GGML_NATIVE=OFF`. Without it, whisper.cpp is compiled with the
instruction set of the *build* machine, so a binary produced on a server with AVX-512 dies with
an illegal instruction — silently, the process simply disappears — on a CPU that lacks it.
`OFF` keeps ggml's portable defaults (AVX2/FMA/F16C), which every x86-64 CPU since ~2013 has.

If you build only for your own machine and want the last few percent of speed, set
`GGML_NATIVE=ON` in your environment. Never do that for a binary you distribute.

## Useful commands

| Command | What it does |
| --- | --- |
| `npm run tauri dev` | run the app with hot reload |
| `npm run tauri build` | release build + NSIS installer |
| `npm run build` | build only the frontend into `dist/` |
| `npm test` | frontend tests (vitest) |
| `cargo test -p glasopis-core` | core logic tests |
| `cargo test -p glasopis` | backend tests (Windows) |
| `cargo fmt --all` | format the Rust code |

## Troubleshooting

**`cmake` not found / whisper.cpp fails to build** — install CMake and the C++ workload of the
Visual Studio Build Tools, then open a fresh terminal so `PATH` is updated.

**`error: linker link.exe not found`** — the MSVC build tools are missing; install the C++
workload.

**The build succeeds but the window is white** — the WebView2 runtime is missing or outdated.

**Antivirus flags the fresh build** — unsigned binaries from an unknown publisher are a common
false positive; see the SmartScreen note in the README.
