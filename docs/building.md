# Компилиране на Glasopis

## Windows (истинската работа)

Glasopis е Windows приложение. Пълна компилация - тази, която произвежда
`GlasopisSetup.exe` - трябва да се случи на Windows.

### Предпоставки

| Инструмент | Бележки |
| --- | --- |
| [Git](https://git-scm.com/) | |
| [Node.js 20+](https://nodejs.org/) | доставя npm |
| [Rust](https://rustup.rs/) | toolchain-ът по подразбиране `x86_64-pc-windows-msvc` |
| [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) | workload *Desktop development with C++* - необходим за Rust, и за whisper.cpp в локална компилация |
| [CMake](https://cmake.org/download/) | само за `--features whisper`; „Add to PATH“ |
| WebView2 Runtime | вече е наличен на Windows 11 и обновен Windows 10; иначе [изтеглете го](https://developer.microsoft.com/microsoft-edge/webview2/) |

### Компилиране

```bash
git clone https://github.com/pavlovdevelop/glasopis.git
cd glasopis
npm install
npm run tauri dev
```

Компилацията по подразбиране използва двигателя Groq и се компилира за няколко минути.

За локално разпознаване на вашия процесор, компилирайте с feature-а:

```bash
npm run tauri build -- --features whisper
```

Тази компилира whisper.cpp, което отнема няколко минути при първо изпълнение и изисква CMake
и C++ build tools. Двигателят след това може да се избере в Настройки → Разпознаване.

Release компилация с инсталатор:

```bash
npm run tauri build
```

Резултат:

```text
target/release/Glasopis.exe
target/release/bundle/nsis/Glasopis_0.1.0_x64-setup.exe
```

Пътищата са в `target/` в корена на хранилището, не в `src-tauri/target/`, защото Glasopis е
Cargo workspace, а workspace-ът споделя една build директория.

Release workflow-ът преименува инсталатора на `GlasopisSetup.exe`, преди да го публикува.

## Linux / macOS (частично)

Не можете да компилирате Windows приложението на Linux, но можете да работите по - и да
проверявате - голяма част от проекта:

```bash
npm install
npm run typecheck        # TypeScript
npm test                 # frontend unit тестове
npm run build             # production frontend bundle

cargo fmt --all --check
cargo test -p glasopis-core          # настройки, команди, речник, модели, аудио математика
cargo clippy -p glasopis-core -- -D warnings
```

Rust кодът, специфичен само за Windows, все пак може да се провери за типове без C/C++
toolchain, като се компилира за Windows target с изключен feature за двигателя за реч:

```bash
rustup target add x86_64-pc-windows-msvc
cargo clippy --target x86_64-pc-windows-msvc --no-default-features --workspace -- -D warnings
```

`--no-default-features` изключва feature-а `whisper`. Такава компилация **не е** използваемо
приложение: транскрипцията връща изрична грешка, че двигателят за реч не е компилиран.
Съществува единствено като помощно средство за проверка на типовете; всяка release
компилация има feature-а включен.

## Преносими бинарни файлове

`.cargo/config.toml` задава `GGML_NATIVE=OFF`. Без това, whisper.cpp се компилира с набора
инструкции на *build* машината, така че бинарен файл, произведен на сървър с AVX-512, умира с
нелегална инструкция - безшумно, процесът просто изчезва - на процесор, който няма тази
инструкция. Файлът тогава задава набора инструкции изрично: AVX2, FMA и F16C включени (почти
всеки x86-64 процесор от около 2013 г. насам ги има, и те правят разпознаването няколко пъти
по-бързо), AVX-512 изключен. Последното има значение: build сървърите често имат AVX-512,
докато потребителски Intel процесори от 12-то поколение нататък (Alder Lake, Raptor Lake) -
не, и точно това несъответствие е това, което убива процеса.

За процесор по-стар от ~2013 г., или Celeron/Pentium N или Atom, задайте и `GGML_AVX`,
`GGML_AVX2`, `GGML_FMA` и `GGML_F16C` на `OFF` - по-бавно, но работи. За собствената ви
съвременна машина, `GGML_NATIVE=ON` изстисква последните няколко процента. Никога не правете
това за бинарен файл, който разпространявате.

Два капана при промяна на тези флагове:

* `whisper-rs-sys` не декларира `cargo:rerun-if-env-changed` за `GGML_*`, така че кеширана
  `target/` тихо пази предишните флагове - компилация може да изглежда поправена, докато
  доставя стария набор инструкции. Изтрийте `target/` (и всеки CI кеш) след промяната им.
* Логовият ред „процесорни инструкции: ...“ (изписан при зареждане на модел) показва с какво
  всъщност е компилиран ggml. Проверявайте там, а не да се доверявате на build конфигурацията.

## Полезни команди

| Команда | Какво прави |
| --- | --- |
| `npm run tauri dev` | стартира приложението с hot reload |
| `npm run tauri build` | release компилация + NSIS инсталатор |
| `npm run build` | компилира само frontend-а в `dist/` |
| `npm test` | frontend тестове (vitest) |
| `cargo test -p glasopis-core` | тестове на основната логика |
| `cargo test -p glasopis` | бекенд тестове (Windows) |
| `cargo fmt --all` | форматира Rust кода |

## Отстраняване на проблеми

**`cmake` не е намерен / whisper.cpp не успява да се компилира** - инсталирайте CMake и C++
workload-а на Visual Studio Build Tools, след което отворете нов терминал, за да се обнови
`PATH`.

**`error: linker link.exe not found`** - липсват MSVC build tools; инсталирайте C++ workload-а.

**Компилацията минава, но прозорецът е бял** - WebView2 runtime-ът липсва или е остарял.

**Антивирус маркира новата компилация** - неподписани бинарни файлове от непознат издател са
чест фалшив положителен резултат; вижте бележката за SmartScreen в README.
