# Manual QA checklist

Automated tests cover the text pipeline, the settings format and the frontend helpers. The
parts that need a real Windows desktop are checked by hand before a release. Tick everything
below on a clean Windows 11 x64 machine and, if possible, once on Windows 10 x64.

## Installation

- [ ] `GlasopisSetup.exe` installs without administrator rights.
- [ ] Glasopis appears in the Start menu and in the system tray.
- [ ] Uninstalling from "Apps & features" removes the application.
- [ ] Settings survive an uninstall/reinstall cycle (they live in `%APPDATA%`).

## First run

- [ ] The onboarding wizard opens automatically.
- [ ] The microphone list shows the real devices; the level meter moves when you speak.
- [ ] The recommended model downloads, shows progress and is verified.
- [ ] A corrupted download is rejected (rename a `.bin` file, re-select it, expect an error).
- [ ] After finishing, the wizard does not appear again.

## Core dictation flow

For each application: click into the text field, press `Ctrl+Alt+Space`, dictate
„Здравей, това е тест на български език.“, press the hotkey again.

- [ ] Notepad
- [ ] Google Chrome — address bar and a `<textarea>` (e.g. a chat box)
- [ ] Microsoft Edge
- [ ] Microsoft Word
- [ ] Excel (a cell)
- [ ] Outlook (message body)
- [ ] VS Code / Cursor (editor and terminal)
- [ ] Claude Code / a terminal prompt — text is inserted, **Enter is not pressed**
- [ ] Discord / Telegram / Messenger
- [ ] A browser form field on a random website

## Bulgarian text quality

- [ ] Cyrillic is correct with the **English** keyboard layout active.
- [ ] Cyrillic is correct with the **Bulgarian** keyboard layout active.
- [ ] „точка“, „запетая“, „въпросителен знак“ produce `.` `,` `?`.
- [ ] „нов ред“ and „нов параграф“ produce one and two line breaks.
- [ ] „изтрий последната дума“ removes the previous word.
- [ ] Sentences start with a capital letter.
- [ ] A multiline dictation is inserted as multiline text.

## Modes and settings

- [ ] Push-to-talk records while held and stops on release.
- [ ] Changing the toggle hotkey takes effect immediately, without a restart.
- [ ] Insertion mode "Симулация на клавиатура" also inserts Cyrillic correctly.
- [ ] Turning off the floating window keeps dictation working.
- [ ] Switching the interface language to English translates the UI.
- [ ] All settings are still there after restarting the application.
- [ ] All settings are still there after restarting Windows.

## Microphone edge cases

- [ ] Selecting a specific microphone is respected.
- [ ] Unplugging the selected USB microphone produces a Bulgarian error, not a crash.
- [ ] Connecting/disconnecting a Bluetooth headset while running does not crash the app.
- [ ] Denying microphone permission in Windows settings shows
      „Glasopis няма достъп до микрофона.“

## System integration

- [ ] Tray menu: every item opens the right screen.
- [ ] Double-clicking the tray icon opens the settings window.
- [ ] Closing the settings window keeps Glasopis running in the tray.
- [ ] "Стартирай с Windows" survives a reboot and starts minimized.
- [ ] Launching Glasopis a second time focuses the existing instance.
- [ ] "Изход" really exits (no process left behind).

## Failure handling

- [ ] With no model selected, starting a dictation opens the model page with a clear message.
- [ ] Silence produces „Не беше чута реч.“ instead of invented text.
- [ ] Insertion into an elevated window (e.g. an admin PowerShell) fails gracefully: the text is
      on the clipboard and the message says so.
- [ ] With the network disconnected, the app starts and dictation still works.

## Performance

- [ ] Idle CPU usage is ~0%.
- [ ] The UI stays responsive while „Обработвам...“ is shown.
- [ ] A 30 second dictation is transcribed in a reasonable time on the balanced model.
- [ ] Multiple monitors and 150% DPI scaling: the overlay appears in the right place.
