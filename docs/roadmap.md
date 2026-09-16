# Roadmap

v0.1.0 is about one thing: hotkey → microphone → local Bulgarian recognition → text in the
active application. Everything below comes after that works reliably, and none of it may
weaken the free, local default.

## Next

- Streaming transcription (text appears while you speak)
- Personal dictionary editor in the settings UI (the storage and the replacement engine exist)
- Richer editing commands („изтрий изречението“, „с главна буква“, „поправи последната дума“)
- Automatic text cleanup (filler words, repetitions)
- Dictation history search
- English UI polish and Bulgarian error messages translated for the English interface

## Later

- Optional GPU acceleration (Vulkan / CUDA builds of whisper.cpp)
- Custom vocabulary passed to the model as an initial prompt
- Custom voice commands and command macros
- Accessibility mode
- More languages beyond bg/en
- macOS and Linux versions

## Deliberately optional, forever

- AI text correction through a cloud model — only ever as an opt-in plugin, never a requirement,
  never in the default build.

Glasopis will not gain advertisements, premium tiers, locked features, usage quotas or accounts.
