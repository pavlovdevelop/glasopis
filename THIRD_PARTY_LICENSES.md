# Лицензи на трети страни

Самият Glasopis е лицензиран под MIT (вижте [LICENSE](LICENSE), Copyright (c) 2026 Pavel Pavlov).
Изграден е върху следния труд на трети страни. Всеки компонент по-долу е безплатен и свободен
за разпространение като част от приложение с MIT лиценз; никой от тях не изисква акаунт, API
ключ или абонамент.

Версиите са тези, фиксирани при подготовката на v0.1.0; изпълнете `cargo metadata` и `npm ls`
за точния набор във вашата работна копия.

## Услуга за разпознаване на реч

Двигателят по подразбиране е [Groq](https://groq.com), използван през неговия
OpenAI-съвместим `/openai/v1/audio/transcriptions` endpoint с модела `whisper-large-v3-turbo`.
Groq е услуга на трета страна, не зависимост на това хранилище: с Glasopis не се разпространява
никакъв код на Groq. Употребата се управлява от собствените условия за ползване и политика за
поверителност на Groq и изисква акаунт и API ключ, които потребителят създава и пази.

## Локално разпознаване на реч (опционален feature `whisper`)

| Компонент | Версия | Лиценз | Бележки |
| --- | --- | --- | --- |
| [whisper.cpp](https://github.com/ggml-org/whisper.cpp) | вграден от whisper-rs-sys | MIT | C/C++ engine за извод, статично слинкован |
| [ggml](https://github.com/ggml-org/ggml) | част от whisper.cpp | MIT | библиотека за тензори, използвана от whisper.cpp |
| [whisper-rs](https://codeberg.org/tazz4843/whisper-rs) | 0.16.0 | Unlicense (обществено достояние) | Rust bindings |
| whisper-rs-sys | 0.15.0 | Unlicense (обществено достояние) | слой за build/FFI |

### Модели за реч

В компилация с feature `whisper` Glasopis изтегля тежести на OpenAI Whisper, конвертирани във
формат ggml и публикувани от проекта whisper.cpp:

| Модел | Файл | Лиценз |
| --- | --- | --- |
| Whisper small (q5_1) | `ggml-small-q5_1.bin` | MIT (тежести на OpenAI Whisper) |
| Whisper large-v3-turbo (q5_0) | `ggml-large-v3-turbo-q5_0.bin` | MIT (тежести на OpenAI Whisper) |
| Whisper medium | `ggml-medium.bin` | MIT (тежести на OpenAI Whisper) |

OpenAI публикува моделите и кода на Whisper под MIT лиценз, който позволява локална употреба и
разпространение. Файловете на моделите **не** са пакетирани с Glasopis и **не** са commit-нати
в това хранилище; те се изтеглят от потребителя от
`https://huggingface.co/ggerganov/whisper.cpp` и се проверяват спрямо SHA-256 контролна сума.

## Rust зависимости

| Crate | Версия | Лиценз |
| --- | --- | --- |
| tauri | 2.11 | Apache-2.0 OR MIT |
| tauri-plugin-global-shortcut | 2.3 | Apache-2.0 OR MIT |
| tauri-plugin-autostart | 2.5 | Apache-2.0 OR MIT |
| tauri-plugin-single-instance | 2.4 | Apache-2.0 OR MIT |
| tauri-plugin-opener | 2.5 | Apache-2.0 OR MIT |
| tauri-plugin-log | 2.9 | Apache-2.0 OR MIT |
| cpal | 0.17 | Apache-2.0 |
| reqwest | 0.13 | MIT OR Apache-2.0 |
| native-tls (Schannel на Windows) | 0.2 | MIT OR Apache-2.0 |
| sha2 | 0.10 | MIT OR Apache-2.0 |
| windows | 0.62 | MIT OR Apache-2.0 |
| serde, serde_json | 1.x | MIT OR Apache-2.0 |
| parking_lot | 0.12 | MIT OR Apache-2.0 |
| anyhow, thiserror, log | 1.x / 2.x | MIT OR Apache-2.0 |

Преходните (transitive) зависимости на изброените по-горе носят разрешителни лицензи (MIT,
Apache-2.0, BSD, ISC, Zlib, Unicode-3.0 или Unlicense).

## Frontend зависимости

| Пакет | Версия | Лиценз |
| --- | --- | --- |
| react, react-dom | 19 | MIT |
| @tauri-apps/api, @tauri-apps/cli | 2 | MIT OR Apache-2.0 |
| vite, @vitejs/plugin-react | 7 / 5 | MIT |
| vitest | 3 | MIT |
| typescript | 5 | Apache-2.0 |

## Windows компоненти

| Компонент | Лиценз |
| --- | --- |
| Microsoft Edge WebView2 Runtime | [Microsoft Developer Services Agreement / условия за разпространение](https://developer.microsoft.com/microsoft-edge/webview2/) - разпространява се от NSIS инсталатора на Tauri, не от това хранилище |

## Шрифтове и ресурси

Не се пакетират шрифтове на трети страни; интерфейсът използва семейството Segoe UI, вече
налично в Windows, с резервен вариант системния UI шрифт. Иконата на приложението в
`src-tauri/icons/` е генерирана за този проект и е обхваната от MIT лиценза на проекта.
