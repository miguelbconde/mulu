# Roadmap

Every version ships with installers built automatically by GitHub Actions, a step-by-step test script, and measured performance numbers. Every version is tested from scratch (download, install, first search), and the time and number of clicks are recorded.

This roadmap was adopted on 2026-11-19 (see [DECISIONS.md](DECISIONS.md), D10) and updated by D14 and D21. Any change of scope produces a new addendum before code is written.

Published goals on every version: installer under 50 MB, memory use, and a comparison with the reference competitors, AnyTXT Searcher and DocFetcher, on the same machine and library (D21).

| Version | Scope |
|---|---|
| **0.1 — Minimal** | First launch with a single action: "Choose the folder with your books". Indexing of PDF, EPUB, TXT, Markdown and HTML in the background, with a progress bar; search works while indexing runs. Words, exact phrases and prefixes (`melancol*`), insensitive to accents and letter case. Text in every script is indexed (D15). One result per file, with the number of hits, 25 words of context before and after, the term highlighted, and the page or section when available. A simple list of files that could not be read, with the reason; those files can still be found by name. Ctrl+F returns to the search box. The start of the built-in help ("?"). Interface in English, Brazilian Portuguese and Spanish (D14), in the "Page" visual identity. File reading isolated in a separate process with time and memory limits. |
| **0.2 — Libraries and scope** (in three releases: 0.2.0 incremental updates, refresh button and scope tree; 0.2.1 several libraries, survey and whole computer (D34); 0.2.2 search signs and advanced search panel, pause and resume) | Several libraries. A quick survey before indexing (file counts by format and folder, with checkboxes to exclude). A folder tree with checkboxes that changes only the search scope. Incremental updates. A refresh button and last-update date next to each library, and a check for changes on startup (D16). Cloud files not yet downloaded are listed as such and never downloaded in bulk. "Whole computer" option, excluding system, program, hidden and temporary folders. Search signs instead of operator words (D33): `"..."` exact phrase, `-` without, `|` either, `~N` within N words, `*` word beginning, `( )` grouping, `name:` file name only. An "Advanced search" panel that writes the signs in the search box as its fields are filled, and a "How to search" glossary with clickable examples. Pause and resume indexing, even across restarts. |
| **0.3 — Reading notes and research projects** (in two releases: 0.3.0 reading notes, 0.3.1 research projects, D35) | Select one hit, all hits in a file, or hits across files, and export them to DOCX, ODT, RTF, Markdown or TXT, with 25, 50 or 100 words of context. Built-in passage preview, with arrows between occurrences. Open the file at the matching page when the reader allows it. Alternative sort orders (hits relative to text length, title, author, year). Research projects: subtopics in a tree gathering books, saved searches that report what is new, and frozen passages, each with an optional note; a project as search scope; exported as reading notes; saved to a single file; a bin to restore deleted items. |
| **0.4 — Formats and report** | DOCX, ODT, RTF, FB2, MOBI/AZW3 without DRM, DOC and DJVU. Detection of protected, password-protected and corrupted files. A saved web page (`page.html` + `page_files`) treated as one document. Scan report listing files not indexed, by reason. |
| **0.5 — OCR** | First, a measured comparison of OCR engines (Tesseract, PaddleOCR, Kraken, ocrs), language by language, on good and degraded public pages, published in `research/ocr-benchmark` (D21). Then: OCR offered from the scan report; language detection from a page sample, with manual correction; background processing that can be paused and resumed; recognized text stored only in the index, never in the file, and marked as OCR in results. Bundled languages: Portuguese, Spanish, English, French, German, Italian, Latin, modern and ancient Greek. Extra packs, "experimental" until measured, added by opening a downloaded file: Chinese (including vertical), Japanese, Korean, Cyrillic, Arabic, Hebrew, Devanagari. |
| **0.6 — Catalog** | Author, title and year (from file name, folder and embedded metadata). Filters by author, year, format, folder and tags. Duplicate detection. |
| **0.7 — Organization** | Rename by configurable templates; move or copy files from a library to a chosen folder, by format, keeping or flattening subfolders. Always with preview, confirmation, an undo log, a suffix for repeated names, and verification of the copy before the original is removed. The index updates without reindexing. |
| **0.8 — Languages** | Spanish interface reviewed by a native speaker. Measured and published search quality for each target language, with special attention to non-Latin scripts. |
| **0.9 — Finish** | Accessibility review (keyboard, contrast, screen readers). Dark mode ("Night"). Performance report with 100,000 files and a published comparison with the main competitors on the same machine and library. Wide macOS test round with volunteers. |
| **1.0** | Stabilization. Signed installers (Windows) and signed, notarized installers (macOS). Microsoft Store release. Complete documentation. Optional update check, off by default. |
| **After 1.0** | Chromebook, as a dedicated app. |

## Before 0.1

- ~~Choose the word segmentation for Chinese text (D9).~~ Done: character pairs (D13).
- ~~Index every script by the Unicode rules (D15).~~ Done: `crates/mulu-text`.
- ~~Set up the automatic installer builds for Windows and macOS.~~ Done: rehearsal version 0.0.1 (D18).
- ~~Automated vulnerability checks on dependencies.~~ Done: cargo-deny on every change and daily (D22).

## Platforms

- **Windows:** reference platform, tested by the maintainer on every version.
- **macOS:** built from the same code from 0.1 on, labeled "experimental" until volunteers test it on real Macs. In the Mac App Store edition, Apple's sandbox limits "whole computer" to the user's home folder and the disks the user picks.
- **Chromebook:** after 1.0.
