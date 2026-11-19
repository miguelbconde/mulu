# Changelog

All notable changes to Mulu are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Project documentation: README (English and Brazilian Portuguese), roadmap, decision log, contributing guide and security policy.
- License: GNU General Public License, version 3 or later.
- Security practices and isolated testing recorded as decision D12.
- Simplified Chinese dropped from the interface languages (D14).
- Text analysis for search (`crates/mulu-text`): Unicode word boundaries, character pairs for scripts written without spaces, folding of case, accents and Arabic variants (D15), with tests in ten scripts.
- Automated checks on Windows, macOS and Linux for every proposed change.
- Measured comparison of Chinese tokenization strategies, with code and results, in `research/cjk-tokenization`; decision D13.
