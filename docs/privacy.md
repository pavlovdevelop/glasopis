# Поверителност / Privacy

## Накратко (BG)

**Glasopis обработва гласа локално на вашия компютър. Аудиозаписите не се изпращат към
външни сървъри.**

- Разпознаването се извършва от whisper.cpp на вашия процесор.
- Записът съществува само в оперативната памет по време на разпознаването и се освобождава
  веднага след това. Glasopis не записва аудио файлове на диска.
- Няма телеметрия, няма профили, няма реклами, няма API ключове.
- Историята на диктовките е **изключена** по подразбиране. Когато я включите, се пази само
  текстът и часът — никога аудио. Може да я изчистите с един бутон.
- Логовете съдържат техническа информация (например колко секунди е записът и коя грешка е
  възникнала), но не и разпознатия текст.

## In short (EN)

Glasopis performs speech recognition locally. Audio never leaves the machine.

## What leaves your computer, and when

| Action | Network use |
| --- | --- |
| Installing Glasopis | download from GitHub Releases (your browser) |
| Downloading a speech model | HTTPS request to `huggingface.co` for the model file |
| Everything else — recording, recognition, text insertion | none |

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
| `%APPDATA%\com.glasopis.app\settings.json` | your settings, including the personal dictionary |
| `%APPDATA%\com.glasopis.app\history.json` | only if you enabled history: transcripts + timestamps |
| `%LOCALAPPDATA%\com.glasopis.app\models\` | downloaded speech models |
| `%LOCALAPPDATA%\com.glasopis.app\logs\` | technical logs, no transcripts |

Deleting these folders removes everything Glasopis stores.

## Microphone permission

Windows 10/11 asks for microphone access per application
(Settings → Privacy & security → Microphone). If access is denied, Glasopis shows
„Glasopis няма достъп до микрофона.“ and records nothing.
