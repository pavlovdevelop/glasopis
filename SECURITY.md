# Security Policy

## Supported versions

Glasopis is in early development. Security fixes are made on the latest released version.

| Version | Supported |
| --- | --- |
| 0.1.x | ✅ |
| older | ❌ |

## Reporting a vulnerability

Please report security issues **privately** through GitHub's
[private vulnerability reporting](https://github.com/pavlovdevelop/glasopis/security/advisories/new)
rather than in a public issue. If that is not available to you, open an issue asking for a
private contact channel without including details of the problem.

Please include:

- what an attacker can do,
- the steps to reproduce it,
- the affected version and your Windows version.

You can expect an acknowledgement within a few days and an honest estimate of when a fix will
ship. This is a volunteer project; there is no bug bounty.

## Threat model

What Glasopis is designed to protect:

- **Your voice.** Audio is processed locally and kept in memory. A build that uploads audio
  would be a critical vulnerability.
- **Your dictated text.** It goes to the clipboard and the focused window, and — only if you
  enable history — to a local file. It is never sent anywhere.
- **The model download.** Downloads only happen over HTTPS from the whisper.cpp model repository
  and every file is verified against a SHA-256 checksum compiled into the application. A URL
  outside the expected host is refused.

What it deliberately does not do:

- Glasopis never executes recognized text as a command and never presses Enter on your behalf.
- Glasopis does not request administrator rights. As a result it cannot insert text into
  applications running elevated; in that case the text is left on the clipboard.

## Unsigned builds

Official release binaries are not code-signed, because a certificate costs money and the project
is free. Verify what you download: prefer building from source, and treat any Glasopis installer
that does not come from the GitHub Releases page of this repository as untrusted.
