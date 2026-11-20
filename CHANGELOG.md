# Changelog

All notable changes to Mulu are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Automated checks on third-party components with cargo-deny, on every change and daily: known vulnerabilities, unmaintained components, licenses and sources (D22).
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
