# Поверителност / Privacy

## Накратко (BG)

**В режима по подразбиране Glasopis изпраща записа към Groq за разпознаване.** Ако това не ви
устройва, компилацията с feature `whisper` разпознава изцяло на вашия компютър.

- По подразбиране разпознаването се извършва от Groq (`whisper-large-v3-turbo`) на техни сървъри,
  според [техните условия](https://groq.com/privacy-policy/). Изпраща се самият запис и езикът,
  нищо друго — нито името ви, нито кое приложение сте ползвали.
- В локален режим разпознаването се извършва от whisper.cpp на вашия процесор и нищо не се
  изпраща никъде.
- Записът съществува само в оперативната памет по време на разпознаването и се освобождава
  веднага след това. Glasopis не записва аудио файлове на диска.
- Няма телеметрия, няма профили, няма реклами, няма API ключове.
- Историята на диктовките е **изключена** по подразбиране. Когато я включите, се пази само
  текстът и часът — никога аудио. Може да я изчистите с един бутон.
- Логовете съдържат техническа информация (например колко секунди е записът и коя грешка е
  възникнала), но не и разпознатия текст.

## In short (EN)

By default Glasopis sends the recording to Groq for recognition. A build with the `whisper`
feature recognizes everything locally and uploads nothing.

## What leaves your computer, and when

| Action | Network use |
| --- | --- |
| Installing Glasopis | download from GitHub Releases (your browser) |
| A dictation in the default mode | HTTPS request to `api.groq.com` carrying the recording |
| Downloading a speech model (local mode) | HTTPS request to `huggingface.co` for the model file |
| A dictation in local mode | none |
| Recording, text processing, text insertion | none |

Glasopis has no update checker, no crash reporter and no analytics in v0.1.0. If an update
check is ever added it will be opt-in and documented here.

## The clipboard

The default insertion strategy puts the recognized text on the Windows clipboard and sends
`Ctrl+V`. Consequences you should know about:

- Clipboard history (Win+V) and third-party clipboard managers will see the dictated text,
  because that is how the Windows clipboard works.
- The previous clipboard content is restored ~800 ms after pasting (configurable, can be
  switched off).
- If you prefer that nothing touches the clipboard, set the insertion mode to
  „Симулация на клавиатура“ (keyboard simulation) in Settings → Въвеждане на текст.

## Files

| File | Contains |
| --- | --- |
| `%APPDATA%\com.glasopis.app\settings.json` | your settings, the personal dictionary **and your Groq API key in plain text** |
| `%APPDATA%\com.glasopis.app\history.json` | only if you enabled history: transcripts + timestamps |
| `%LOCALAPPDATA%\com.glasopis.app\models\` | downloaded speech models |
| `%LOCALAPPDATA%\com.glasopis.app\logs\` | technical logs, no transcripts |

Deleting these folders removes everything Glasopis stores.

The API key is stored unencrypted, like most desktop tools do it; anyone with access to your
Windows account can read it. Protecting it with the Windows credential store is on the roadmap.
If a key leaks, revoke it at [console.groq.com/keys](https://console.groq.com/keys).

## Microphone permission

Windows 10/11 asks for microphone access per application
(Settings → Privacy & security → Microphone). If access is denied, Glasopis shows
„Glasopis няма достъп до микрофона.“ and records nothing.
