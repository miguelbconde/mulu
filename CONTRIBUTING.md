# Contributing to Mulu

Thank you for your interest. Mulu is at an early stage: the design is settled (see [DECISIONS.md](DECISIONS.md) and [ROADMAP.md](ROADMAP.md)) and the first code is about to be written.

## Ways to help now

- **Test on macOS.** We need volunteers with a Mac to follow the test script of each version. Open an issue titled "macOS tester" to join.
- **Review a translation.** The interface will be in English, Brazilian Portuguese, Spanish and Simplified Chinese. A translation is considered ready only after a native speaker has reviewed it.
- **Offer test files.** Public-domain books in less common formats, scripts or languages help us test. Please do not send copyrighted material.
- **Report problems and ideas** by opening an issue. For security problems, follow [SECURITY.md](SECURITY.md) instead, and do not open a public issue.

## Principles every change must respect

- **Privacy.** No telemetry, accounts, ads or network connections. The only exception is an optional update check, off by default.
- **Library integrity.** Mulu only reads the user's files. Any action that renames or moves files needs explicit confirmation, a preview and an undo log. Never delete permanently; never overwrite.
- **Security.** Treat every file as untrusted; never run embedded macros or scripts; parse files in an isolated process with time and memory limits; prefer memory-safe languages and libraries; no administrator privileges.
- **Few dependencies.** Each new dependency needs a written reason in the pull request, a compatible license (no AGPL or proprietary code), and a pinned version.
- **Accessibility.** Everything works with the keyboard, has adequate contrast and is announced by screen readers.
- **Honesty.** Do not describe something as working unless it has been tested. Performance claims need a published, reproducible method.
- **No DRM removal** and nothing that facilitates piracy.

## How changes are made

- All code, comments, documentation and commit messages are in English.
- The `main` branch is protected: every change goes through a pull request, and automated checks must pass once they exist.
- Commit messages start with a short summary line in the imperative ("Add EPUB reader"), followed by a blank line and the reason for the change.
- Tests are required for new behavior, including corrupted files, accented and non-Latin file names, long paths, and large libraries.

## License

By contributing, you agree that your contribution is licensed under the GNU General Public License, version 3 or later, like the rest of the project.
