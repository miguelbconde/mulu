# Changelog

All notable changes to Mulu are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.1] - 2026-11-21

Test script: `docs/testing/v0.2.1.md`.

### Added

- Several libraries: "+ Add a library" in the panel; each library has its own folder tree, ↻ button and date; a search covers them all, and each result says which library it comes from. A library can be taken out of Mulu: its folder and files are never touched; Mulu's index of it is deleted the next time Mulu starts (D34).
- A survey before the first reading: how many book files of each format and in each folder, with boxes to leave formats and folders out before anything is read; files only in the cloud are counted and never downloaded (D34).
- "Whole computer", on the welcome screen and in the panel: the home folder and the other disks, each its own library, with system, program, hidden and temporary folders left out (D34).

### Changed

- A folder inside a library cannot become a library (its books would be counted twice; the scope tree already narrows a search to it). A folder that holds a library can, and leaves that library out.
- A library whose folder is missing (an external disk not plugged in) is no longer checked, so its index is kept; the panel says the folder was not found.

### Fixed

- Release 0.2.0 was built but not published: its release notes file was missing. The checks of every proposal now fail when the version has no release notes or test script.

### Security

- Updated two components of the window framework after advisories published on 22 and 23 November: `url` 2.5.9 (RUSTSEC-2026-0592, a crash on deeply nested `blob:` addresses) and `cookie` 0.18.3 (RUSTSEC-2026-0594, repeated nonces in processes that fork). Mulu makes no network connections and its window shows only its own pages, so the risk to users was low; no behavior changes.

## [0.2.0] - 2026-11-21

Test script: `docs/testing/v0.2.0.md`.

### Added

- Incremental updates: each time Mulu opens, and with the new ↻ button, it checks the library for changes and reads only new and changed files. Renamed and moved books are recognized by a fingerprint of their content and keep their place in the index without being read again; files no longer in the folder leave the index (never the disk) (D16, D31).
- A panel on the left with the library, the date of the latest check and a short summary of what it found ("3 new, 1 changed, 2 removed from the index").
- The library's folders as a tree with checkboxes: unticked folders are left out of the search, instantly, and the index stays whole. Each box counts for the files directly in its folder; ticking or unticking applies to the folders below it too, so a folder can be left out while one of its subfolders stays in (D32).
- Results say when the search covered only part of the library.

### Changed

- Updating from 0.1 keeps the index: files that did not change are not read again.

## [0.1.4] - 2026-11-21

Test script: `docs/testing/v0.1.4.md`.

### Fixed

- Very long books (complete works of thousands of pages) were stopped after two minutes even while they advanced. A file is now stopped only when it reads no new page for two minutes, or after 30 minutes in all (D30).

### Changed

- "Read faster" reads one file per physical processor core: the extra logical processors only made each file slower (D30).
- The largest files are read first, so that the reading does not end with one long book read alone (D30).
- The reading report says how many files were read at once.
- When the reading stops (closing Mulu, or changing the pace), files read but not yet indexed are read again next time instead of the reading going on.

## [0.1.3] - 2026-11-20

Test script: `docs/testing/v0.1.3.md`.

### Fixed

- Scanned books with a text layer were reported as "too large": PDFium kept every page it had read in memory (over 1 GB for a 200-page book). The document is now reopened every 25 pages: 180 MB for that book, and faster (D29).

### Added

- "Read faster": while the library is being read, the reading can switch to full speed (all processors but one, normal priority) and back to the background (D29).

## [0.1.2] - 2026-11-20

Test script: `docs/testing/v0.1.2.md`.

### Changed

- PDFs are read with PDFium, Google's PDF engine, instead of pdf_oxide: in our measurements on real books, 20 to 190 times faster and with far less memory, for the same text quality. Large books no longer hit the time limit (D28).
- The memory of each reading process is also watched from outside, which covers the PDF engine (D28).

## [0.1.1] - 2026-11-20

Corrections after the first test on Windows. Test script: `docs/testing/v0.1.1.md`.

### Fixed

- "Show in folder" opened the Documents folder instead of the file's folder on Windows when the path had spaces.
- The list of files that could not be read and the help can be closed from a × button at the top (and with Esc), not only from the button at the end.

### Changed

- Faster reading of a library: several files are read at the same time, in separate reading processes (up to half of the processors, at most three). Twice as fast on two processors in our measurement (D27).
- The list of files that could not be read is grouped by reason, with the count of each reason at the top.

### Added

- "Copy a reading report for the developer": file counts, reasons, times and the slowest files (names, sizes and times only, never their content), to paste in a message.

## [0.1.0] - 2026-11-20

First version that searches, for testers. Test script: `docs/testing/v0.1.0.md`.

### Added

- Automated checks on third-party components with cargo-deny, on every change and daily: known vulnerabilities, unmaintained components, licenses and sources (D22).
- The window of version 0.1: choose the folder with your books; Mulu finds and reads them in the background, with a progress bar, and searches work at once; results with occurrence counts, passages with page or chapter and the term highlighted; files found by name; the list of files that could not be read, with the reason; open a file or show it in its folder; Ctrl+F; help (D26).
- Library catalog in SQLite; indexing resumes where it stopped; files kept only in the cloud are never read (D26).
- Search by the beginning of a word: `melancol*` finds melancolia, melancólico, melancholy (D21).
- Isolated reading process (`crates/mulu-reader`): files are read in a separate, low-priority process with a time limit and a memory allowance; a file that hangs, uses too much memory or crashes the reader is reported and the next file is read normally (D25).
- PDF reader (pdf_oxide), page by page with printed page numbers, chosen by a published comparison of five engines (`research/pdf-extraction`, D24).
- File readers (`crates/mulu-extract`) for plain text, Markdown, HTML and EPUB, with metadata, chapters in reading order, encoding detection, DRM detection and size limits against hostile files (D23).
- Decision D21 (addendum 2: features from DocFetcher, OCR languages and engine comparison, help, reference competitors) recorded; roadmap updated.

## [0.0.1] - 2026-11-19

Rehearsal version for testers. It only opens the window, to test the delivery path before search exists.

### Added

- The desktop window (Tauri 2), in the "Page" visual identity, in English, Brazilian Portuguese and Spanish (D17).
- Installers built by GitHub Actions: Windows (per-user, no administrator password) and macOS (universal, experimental), with SHA-256 checksums, a build provenance attestation and a Windows Sandbox file for testers (D18).
- Test script: `docs/testing/v0.0.1.md`.

## Earlier work (before 0.0.1)

### Added

- Project documentation: README (English and Brazilian Portuguese), roadmap, decision log, contributing guide and security policy.
- License: GNU General Public License, version 3 or later.
- Security practices and isolated testing recorded as decision D12.
- Simplified Chinese dropped from the interface languages (D14).
- Full-text index (`crates/mulu-index`): one entry per page, counts from the index, passages of 25 words on each side cut on demand, several words or exact phrases (D20).
- Refresh button for each library and a check on startup planned for 0.2 (D16).
- Text analysis for search (`crates/mulu-text`): Unicode word boundaries, character pairs for scripts written without spaces, folding of case, accents and Arabic variants (D15), with tests in ten scripts.
- Automated checks on Windows, macOS and Linux for every proposed change.
- Measured comparison of Chinese tokenization strategies, with code and results, in `research/cjk-tokenization`; decision D13.
