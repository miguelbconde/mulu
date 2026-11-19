# Decisions

What was decided, when, by whom, and why. New decisions are appended; earlier ones are never rewritten, only marked as changed by a later entry.

"Maintainer" is the repository owner, [@miguelbconde](https://github.com/miguelbconde), who decides on use, appearance, priority, cost and risk. "Developer" is Claude, who decides on technical matters after comparing at least two options.

## 2026-11-17

### D1. Phase 1 proposal accepted (maintainer)

Rust + Tauri + Tantivy + SQLite; license GPL-3.0-or-later; versions 0.1 to 1.0. File reading runs in an isolated process with time and memory limits.

Why this stack: it combines search speed, a small installer, memory safety and full typographic freedom for the interface. The cost is a more demanding language for future contributors, offset by readable, commented code and documentation. Alternatives considered: C#/.NET + Avalonia + Lucene.NET; Java + Lucene.

*Changed by D10: the roadmap was redone, and OCR moved before 1.0.*

### D2. Extremely simple installation and first launch (maintainer)

People in the humanities often struggle with technology, so simplicity has the same weight as the core principles. One file to download; installation in two clicks, per user, with no administrator password and no technical questions; no mandatory settings; plain-language messages, including errors; every version tested from scratch, with time and clicks recorded; updates never force a reinstall or an unnecessary reindex. An unsigned installer triggers a Windows warning, so signing becomes a priority (see D11).

*Changed by D5: the first launch offers two options, a chosen folder or the whole computer.*

### D3. Worldwide scope (maintainer)

The maintainer's own library is only a test bed. No design decision is based on its characteristics; personal habits, such as a renaming pattern, become configurable options.

### D4. Visual identity: "Page" (maintainer)

Chosen among three directions. White background, a book serif typeface, hairlines, and a single stamp red (#B3261E) for 目录, counts and the main button; small sans-serif labels. The "Night" direction (dark background, amber accents) becomes the dark mode. The yellow highlighter color is reserved for search hits. Fonts are bundled, because Mulu makes no network connections.

## 2026-11-19

### D5. Project instructions, version 2 (maintainer)

Scope changes: macOS in addition to Windows; interface also in Spanish and Simplified Chinese; whole-computer scan as an option; libraries with search scope by subfolder; OCR in the main flow, in 11 languages; reading notes in DOCX, ODT, RTF, Markdown and TXT, with 25, 50 or 100 words of context; moving or copying files out of a library.

Adjustments made while adopting them: "no security warnings" is a goal with a defined path, not a promise, because part of it depends on Microsoft and Apple; "whole computer" excludes system, program, hidden and temporary folders; search works while indexing runs; the scan report counts every file not indexed, by reason; the old .doc format is replaced by RTF as an export format; Modern and Ancient Greek, and Simplified and Traditional Chinese, are separate OCR languages; a platform is announced as supported only after a person has tested it on a real device.

### D6. Free and paid editions (maintainer)

One complete, open-source program. Free on the project page; paid in the app stores, where the price buys a more convenient installation, automatic updates and support for the project. No locked features, no account, no license key. Why: it is the only paid model that does not contradict openness and "no account, no connection". Prices and store accounts are decided near 1.0.

### D7. Chromebook after 1.0 (maintainer)

A dedicated app for the new Android-based ChromeOS. Known limit: apps there only reach folders the user authorizes, so there is no whole-computer scan. Why: the platform is in transition.

### D8. OCR before 1.0 (maintainer)

OCR became part of the main flow (the scan report), so it comes right after search and reading notes.

### D9. Chinese word segmentation chosen before 0.1 (developer)

The index stores text already split into words. Changing the splitting rule later would force every user to rebuild the index, which D2 forbids without need. The choice (between options such as jieba and lindera, both used with Tantivy) comes from a short test before any code; the full quality measurement is part of 0.8.

### D10. Roadmap redone (maintainer)

Ten steps to 1.0: 0.1 minimal; 0.2 libraries and scope; 0.3 reading notes; 0.4 formats and scan report; 0.5 OCR; 0.6 catalog; 0.7 organization; 0.8 languages; 0.9 finish; 1.0 release. Chromebook after 1.0. See [ROADMAP.md](ROADMAP.md).

Why this order: the most important use (gathering everything the library holds about a term) comes first; changing files on disk, the riskiest part, comes last, once reading and search are stable. macOS installers are built from 0.1 and stay "experimental" until volunteers test them. In the Mac App Store edition, Apple's sandbox does not allow full disk access, so "whole computer" becomes the home folder plus the disks the user picks.

### D11. Windows code signing, in stages (maintainer)

1. Early test versions: unsigned installers, for the maintainer and volunteer testers only, with the test script explaining the Windows warning.
2. Right after 0.1: apply to the SignPath Foundation for free open-source signing. It requires an OSI-approved license, a public repository, builds on GitHub Actions, two-factor authentication and at least one prior release. SignPath also verifies that signed files were built from the public repository.
3. Near 1.0: Microsoft Store release as an MSIX package, which Microsoft signs for free. Tauri does not build MSIX itself, so the packaging step will be added to the project's build pipeline. Testers can receive versions through a private Store audience.
4. Only if SignPath declines: consider a paid certificate (about US$150 to 300 a year), near 1.0.

Why: no cost now, and nothing is locked in, since signing is applied to finished installers. Known limits: SignPath approval is not guaranteed; with it, the publisher shown is SignPath Foundation; a new program can still trigger warnings until it builds reputation; only the Store avoids them entirely; publishing a Store listing to the public cannot be reverted to a private audience.

## Open

- Apple Developer Program enrollment (needed to notarize macOS builds): cost to be presented to the maintainer before the macOS test round.
