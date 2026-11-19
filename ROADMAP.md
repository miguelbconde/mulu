# Roadmap

Every version ships with installers built automatically by GitHub Actions, a step-by-step test script, and measured performance numbers. Every version is tested from scratch (download, install, first search), and the time and number of clicks are recorded.

This roadmap was adopted on 2026-11-19 (see [DECISIONS.md](DECISIONS.md), D10). Any change of scope produces a new addendum before code is written.

| Version | Scope |
|---|---|
| **0.1 — Minimal** | First launch with a single action: "Choose the folder with your books". Indexing of PDF, EPUB, TXT, Markdown and HTML in the background, with a progress bar; search works while indexing runs. Words and exact phrases, insensitive to accents and letter case. One result per file, with the number of hits, 25 words of context before and after, the term highlighted, and the page or section when available. A simple list of files that could not be read, with the reason. Interface in English and Brazilian Portuguese, in the "Page" visual identity. File reading isolated in a separate process with time and memory limits. |
| **0.2 — Libraries and scope** | Several libraries. A quick survey before indexing (file counts by format and folder, with checkboxes to exclude). A folder tree with checkboxes that changes only the search scope. Incremental updates. Cloud files not yet downloaded are listed as such and never downloaded in bulk. "Whole computer" option, excluding system, program, hidden and temporary folders. AND, OR, NOT and proximity operators. |
| **0.3 — Reading notes** | Select one hit, all hits in a file, or hits across files, and export them to DOCX, ODT, RTF, Markdown or TXT, with 25, 50 or 100 words of context. Built-in passage preview. Open the file at the matching page when the reader allows it. Alternative sort orders (hits relative to text length, title, author, year). |
| **0.4 — Formats and report** | DOCX, ODT, RTF, FB2, MOBI/AZW3 without DRM, DOC and DJVU. Detection of protected, password-protected and corrupted files. Scan report listing files not indexed, by reason. |
| **0.5 — OCR** | First, a measured comparison of OCR engines (Tesseract and at least one alternative) on a public page set in all 11 languages, with published results. Then: OCR offered from the scan report; language detection from a page sample, with manual correction; background processing that can be paused and resumed; recognized text stored only in the index, never in the file. All 11 language packs bundled in the installer. |
| **0.6 — Catalog** | Author, title and year (from file name, folder and embedded metadata). Filters by author, year, format, folder and tags. Duplicate detection. |
| **0.7 — Organization** | Rename by configurable templates; move or copy files from a library to a chosen folder, by format, keeping or flattening subfolders. Always with preview, confirmation, an undo log, a suffix for repeated names, and verification of the copy before the original is removed. The index updates without reindexing. |
| **0.8 — Languages** | Interface in Spanish, reviewed by a native speaker. Measured and published search quality for each target language, with special attention to non-Latin scripts. |
| **0.9 — Finish** | Accessibility review (keyboard, contrast, screen readers). Dark mode ("Night"). Performance report with 100,000 files and a published comparison with the main competitors on the same machine and library. Wide macOS test round with volunteers. |
| **1.0** | Stabilization. Signed installers (Windows) and signed, notarized installers (macOS). Microsoft Store release. Complete documentation. Optional update check, off by default. |
| **After 1.0** | Chromebook, as a dedicated app. |

## Before 0.1

- ~~Choose the word segmentation for Chinese text (D9).~~ Done: character pairs (D13).
- Set up the automatic installer builds for Windows and macOS.

## Platforms

- **Windows:** reference platform, tested by the maintainer on every version.
- **macOS:** built from the same code from 0.1 on, labeled "experimental" until volunteers test it on real Macs. In the Mac App Store edition, Apple's sandbox limits "whole computer" to the user's home folder and the disks the user picks.
- **Chromebook:** after 1.0.
