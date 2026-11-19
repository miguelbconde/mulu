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

### D12. Security practices and isolated testing (maintainer and developer)

*Decided on 2026-11-17 and first recorded on an unmerged branch; carried over here.*

Threats considered: a compromised third-party dependency, a hijacked maintainer account, a tampered installer, a malicious document, and mistakes in code that no second person reviews.

- Few dependencies, with pinned versions and a lock file; an automated audit for known vulnerabilities runs on every change.
- Installers are built only by GitHub Actions from the public code, never on a personal computer, and carry a build provenance attestation and published checksums.
- Workflows run with minimal permissions; third-party actions are pinned to an exact commit.
- An automated test fails if Mulu attempts any network connection.
- Two-step verification on the maintainer's GitHub account.
- The `main` branch is protected; every change goes through a pull request.
- Test versions are installed and tried inside Windows Sandbox, against a copy of the library mapped read-only, with networking disabled. A ready-made Sandbox configuration file ships with each test version.
- OpenSSF Scorecard is adopted as an independent measure once the code exists.

### D13. Chinese text is indexed as character pairs (developer; result of D9)

Han characters are indexed as overlapping pairs (bigrams), with a separate single-character field without positions for one-character searches. A search for several characters is a phrase of pairs, so it finds exactly the documents that contain that string, as Ctrl+F would. A position gap at punctuation keeps phrases from matching across sentence breaks.

Why: in a measured comparison on modern and classical Chinese, in Simplified and Traditional characters, bigrams found 100% of the relevant documents with no false matches. Dictionary segmentation (jieba) found 76% for dictionary words, 58% for text fragments and 24% for single characters. One-character terms were just as exact but up to 13 times slower on the slowest queries. Bigrams need no dictionary, so no dictionary file is shipped and no dictionary update can force users to rebuild their index. Cost: the index is about twice the size of a jieba index (about 1.1 times the size of the text), and indexing is slower. Method, code and results: [research/cjk-tokenization](research/cjk-tokenization/README.md).

### D14. No Chinese interface for now (maintainer)

The interface languages are English, Brazilian Portuguese and Spanish. Simplified Chinese is dropped from the interface, because a weak translation would be the first thing a Chinese-speaking user sees, and a native reviewer is not guaranteed. The interface stays ready for new languages.

Search inside Chinese texts stays (D13): it costs almost nothing, does not affect other languages, and adding it later would force every user to rebuild the index. Chinese OCR stays planned, but it is confirmed only if the measured comparison in 0.5 shows acceptable quality; otherwise it is labeled experimental or dropped. More generally, no language is announced as supported before its quality has been measured and published.

### D15. Every script is indexed by the Unicode rules from 0.1 (maintainer and developer)

Text in every writing system is indexed from the first version, so that no script needs a rebuilt index later. No script is announced as supported before its search quality has been measured and published (version 0.8), as with Chinese in D14.

How (developer): words are found with the Unicode word boundary rules (UAX #29); scripts written without spaces (Chinese, Japanese, Korean, Thai, Lao, Khmer, Myanmar) use the character pairs of D13; apostrophes split words ("homme" in "l'homme"); case, compatibility forms and optional marks are folded in Latin, Greek, Cyrillic, Hebrew and Arabic, but Indic vowel signs are kept; common Arabic spelling variants are unified.

Why: a quick test showed that naive splitting cuts Hindi words at the virama (हिन्दी became two fragments), keeps Arabic vowel marks that searchers do not type, and leaves Japanese sentences as one block. Libraries compared: the small, widely used `unicode-segmentation` and `unicode-normalization` crates, and ICU4X, the Unicode Consortium's library. ICU4X segments unspaced scripts with dictionaries or statistical models that add several megabytes and could change between versions, forcing a rebuilt index; character pairs need no data. Known limit: Arabic prefixes such as al- are not yet removed, so a bare word is not found inside its prefixed form; light stemming will be measured in 0.8.

### D16. A refresh button for each library, and a check on startup (maintainer)

Each library shows, next to its name, a refresh button and the date of its last update. Mulu also checks for changes each time it opens. A refresh only reprocesses what changed (new, modified, renamed or deleted files), runs in the background while search keeps working, and ends with a short summary ("3 new, 1 changed, 2 removed from the index"). Original files are never touched: "removed" applies only to the index. The button works with the keyboard and is announced by screen readers.

Why: automatic change detection is not fully reliable in cloud-synced folders, on external disks, or in very large libraries, so users need a simple, dependable way to say "check again". Continuous watching of folders while Mulu is open was considered and postponed: it costs more resources and is less reliable in cloud folders; it may be measured later.

### D17. The interface uses plain web standards, without npm (developer)

The window's content is written in plain HTML, CSS and JavaScript, with no framework, bundler or package from npm. Alternatives considered: a framework such as Svelte or SolidJS built with Vite, which is faster to write but pulls hundreds of npm packages into the build.

Why: npm is going through a wave of self-propagating supply-chain attacks (Shai-Hulud and its variants; in August 2026, ChainDrop compromised more than 1,300 packages). With no npm packages, none of that code can reach Mulu's build or its users. The cost is more hand-written code, kept small and readable.

### D18. How installers are built (developer)

- **Windows:** an NSIS installer that installs for the current user only, so no administrator password is asked (D2). On the rare Windows 10 computer without Microsoft's web view (WebView2, built into Windows 11), the installer downloads it from Microsoft once; Mulu itself never connects to the network.
- **macOS:** a universal app (Apple silicon and Intel) in a disk image, built on GitHub's Intel macOS runner. Not signed or notarized until the Apple Developer Program decision; labeled experimental.
- **Fonts:** Newsreader and Inter (SIL Open Font License) are bundled, cut down to Latin, Greek and Cyrillic; the 目录 mark uses only those two characters cut from Noto Serif SC. Other scripts use the system's fonts.
- **Window security:** the page may load only Mulu's own files (content security policy, checked by a test); the window may use only Tauri's basic core features.
- **Provenance:** every release lists SHA-256 checksums and carries a build provenance attestation from GitHub Actions (D12).
- **Rehearsal release:** version 0.0.1 only opens the window, to test the delivery path (build, download, install, first launch) before search exists.

Known cost: the window toolkit (Tauri 2) brings about 400 Rust packages across all platforms into the lock file; each platform compiles only its own. Automated vulnerability checks on them are planned before 0.1.

## Open

- Apple Developer Program enrollment (needed to notarize macOS builds): cost to be presented to the maintainer before the macOS test round.
