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
- Test versions are installed and tried inside Windows Sandbox, against a copy of the library mapped read-only, with networking disabled. A ready-made Sandbox configuration file ships with each test version. *(Changed by D19: a separate Windows account, because Windows Sandbox fails on the maintainer's computer.)*
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
- **When a version is published:** when a reviewed pull request that changes the version number is merged into `main`. The release workflow then builds, attests and publishes the version and creates its tag. No one pushes tags or builds by hand.
- **Rehearsal release:** version 0.0.1 only opens the window, to test the delivery path (build, download, install, first launch) before search exists.

Known cost: the window toolkit (Tauri 2) brings about 400 Rust packages across all platforms into the lock file; each platform compiles only its own. Automated vulnerability checks on them are planned before 0.1.

### D19. Test versions are tried in a separate Windows account (maintainer)

Windows Sandbox, chosen in D12, fails to start on the maintainer's Windows 11 Pro ("Value does not fall within the expected range") even with virtualization enabled and after updating Windows and the Sandbox app and re-enabling the feature. A virtual machine was considered and declined: Microsoft no longer offers ready-made Windows virtual machines, an evaluation copy of Windows expires every 90 days, and it needs about 60 GB of disk.

Test versions that read files are therefore tried in a separate, standard Windows account that cannot see the tester's personal files, against a copy of the library placed in Public Documents. Windows Sandbox remains an option for testers on whose computers it works. This is a separation of accounts, not a disposable machine; Mulu's own read-only design, provenance attestations and automated tests remain the main protections. Instructions: [docs/testing/test-account.md](docs/testing/test-account.md).

### D20. How the index stores and counts (developer)

- **One index entry per page** (or per chapter or section, for formats without fixed pages), with the file it belongs to. This is what lets every passage show its page or section.
- **Counts come from the index itself.** The number of occurrences in each file is read from the index's lists of positions, without reading the stored text. Exact phrases are counted by checking that each word sits at its expected position after the first.
- **Passages are cut on demand.** Only the passages on screen (the first ones of each file) are cut from the stored text; the rest are fetched when the user asks for more.
- **Several words without quotes** match files that contain all of them, anywhere in the file, not necessarily on the same page (project instructions, section 5).

Why: the first version cut a passage for every occurrence, which took 1.8 seconds for a very common Chinese character in only 3,080 pages. With counts from the index and passages on demand, the same search takes 22 ms; with ten times more text (30,800 pages, 1.2 million occurrences) counting takes 2 ms and the first screen of results is ready in at most 50 ms (single run on the development machine; formal benchmarks come with 0.1). Tantivy's own phrase counters are internal to the library, so Mulu reads the public position lists and counts itself; a test checks that these counts always equal the passages found in the text.

### D21. Addendum 2 accepted: features from DocFetcher, OCR, help and competitors (maintainer)

Decided on 2026-11-19, in the evening.

- **Taken from DocFetcher:** prefix search (`melancol*`) in scripts with spaces between words; finding files by name even when their content cannot be read; pausing and resuming indexing, even across restarts; arrows between occurrences in the preview, and Ctrl+F back to the search box; a saved web page (`page.html` + `page_files`) treated as one document. **Left out:** compressed archives (size and the risk of archives that expand to fill the disk), exclusion patterns, configurable extensions, manual term weights, range queries, memory tuning, a background folder watcher (D16) and fuzzy search (may return if research on old spellings needs it).
- **OCR:** bundled, with the light ("fast") models: published measurements show the heavy Tesseract models cost about four times more space and time for equal or slightly better accuracy. Quality comes from image preparation, the right language and the best engine per language. Passages that come from OCR are marked as such.
- **OCR languages in the installer:** Portuguese, Spanish, English, French, German, Italian, Latin, modern Greek and ancient Greek. **Extras on request, "experimental" until measured:** Chinese (simplified, traditional, vertical), Japanese, Korean, Cyrillic, Arabic, Hebrew, Devanagari. Extras are added through the browser: the user downloads a pack from Mulu's page and opens it; Mulu verifies it is authentic and installs it, with no network connection of its own.
- **OCR engine comparison before the 0.5 code:** Tesseract (fast and best models), PaddleOCR PP-OCRv5, Kraken and ocrs, language by language, on good and degraded pages; main measure: the share of words a search would find; results published in `research/ocr-benchmark`.
- **Published goal:** installer under 50 MB, measured on every version together with memory use.
- **Help:** a "?" button with a short manual and FAQ in the interface languages, searchable with Mulu's own search, offline; error messages with "learn more". Starts in 0.1.
- **Reference competitors:** AnyTXT Searcher (main one, the closest to Mulu: fast search on Windows with OCR) and DocFetcher. Comparison published on every version, with a reproducible method, preferably on GitHub's Windows machines.

*Changes D5 (OCR language list) and the roadmap (D10).*

## 2026-11-20

### D22. Automated checks on dependencies with cargo-deny (developer)

Every proposed change runs `cargo-deny` against `Cargo.lock`: known vulnerabilities and unmaintained or withdrawn components (RustSec advisory database), licenses compatible with GPL-3.0-or-later, and components coming only from crates.io. A daily run catches advisories published after a change was merged.

Alternatives considered: `cargo-audit` (the RustSec project's own tool) checks only advisories; `cargo-deny` uses the same database and adds licenses and sources, so one tool covers both. The GitHub action is pinned to an exact commit (D12).

The check covers the components built for Windows and macOS, the systems Mulu is distributed for. Linux is used only by the automated tests; its window components (GTK 3, whose bindings are no longer maintained, including the `glib` component of D18) are not shipped to anyone. No advisory is ignored; any future exception needs a written reason in `deny.toml` and an entry here.

### D23. Readers for plain text, Markdown, HTML and EPUB (developer)

A new component, `crates/mulu-extract`, turns each file into its metadata and a list of sections (pages, or chapters for formats without fixed pages), which become index entries (D20).

- **HTML and EPUB chapters:** the tokenizer of html5ever, the HTML parser of the Servo browser engine, already inside Mulu through Tauri. Only the tokenizer is used (no document tree), so memory stays proportional to the text; scripts and styles are skipped and never run. Alternative considered: Cloudflare's `lol_html`, a fast streaming rewriter, but a new component built for rewriting pages rather than reading them.
- **EPUB:** read by Mulu's own short code on top of `zip` and `quick-xml` (the latter already inside Mulu through Tauri), following the package's reading order. Alternatives considered: ready-made EPUB libraries (`epub-parser`, `rbook`); they bring more components and leave out what Mulu needs most: size checks on the bytes actually unpacked (against "zip bombs") and the distinction between DRM and the harmless font obfuscation some publishers use. Books with DRM (Adobe, Apple, Barnes & Noble, Readium LCP) are listed as protected; Mulu never tries to remove the protection.
- **Character encodings:** byte-order mark, then the encoding the file declares, then UTF-8, then a guess by `chardetng`, the detector Firefox uses (Windows-1252, Shift_JIS, GBK, Big5, KOI8-R and others), decoding with `encoding_rs`, also from Firefox.
- **Markdown** is read as plain text in 0.1: its marks are punctuation, which the search already ignores. **Plain text** has no pages, unless it carries page-break characters (form feed), as some converters produce.

Tests: books in several scripts and encodings, file names in Latin, Greek, Cyrillic and Chinese, damaged and truncated files, a 50 MB "zip bomb" stopped by the limits, and 2,000 rounds of random or damaged input in every format, with no crash. Cost: twelve new components in the build: `zip`, `chardetng`, `encoding_rs` and nine small helpers they use (two only on macOS), all passing the D22 checks; the ZIP compression formats Mulu does not need (zopfli, bzip2, zstd and others) are left out.

### D24. PDFs are read with pdf_oxide (developer)

Chosen by a measured comparison, published with its code in `research/pdf-extraction`: five engines on 61 PDFs with known text in 17 languages and scripts (made by LibreOffice, Chrome, XeLaTeX, pdfLaTeX and ReportLab) and on the 1,103 real, often damaged, PDFs of the pdf.js test suite.

| | pdf_oxide | PDFium | hayro + own layout | pdf-extract |
|---|---|---|---|---|
| Words found on the right page | 97.7% | 97.4% | 73.8% | 81.7% |
| Three-word phrases found | 95.6% | 91.4% | 68.7% | 82.6% |
| Crashes on 1,103 real PDFs | 0 | 0 | 0 | 51 |
| Median time per real PDF | 5 ms | 69 ms (via Python) | 5 ms | 5 ms |

Why pdf_oxide: the best measured text and phrase quality, no crash and no time limit on hostile files, the fastest, and written in Rust: it ships inside Mulu's program. PDFium (the engine in Chrome) came a close second; it was set aside because it is C++ (not memory-safe) and would ship as a separate, prebuilt library. hayro, a careful pure-Rust interpreter, gives glyphs rather than text: words, lines and reading order would be Mulu's own code to write and maintain (our simple attempt reached 74%). pdf-extract crashed on 51 files.

How Mulu uses it: one section per page, with the page's printed label when the PDF defines one (preface "iv", book page "1" on the 15th page of the file); copy-restricted PDFs open as in any reader; password-protected ones are listed as protected; scans without text are listed for OCR (0.5). A shared clean-up joins words hyphenated at line ends, removes line breaks inside Chinese, Japanese and Thai text, and puts back the accents old TeX fonts draw separately (88% to 99% of words found in those files).

Costs and limits: 53 new Rust components, among them two flagged by RustSec as unmaintained (`chrono`, `ttf-parser`; no known vulnerability), allowed in `deny.toml` with this reason; a young project with one main maintainer and frequent releases, so updates are pinned and reviewed; one damaged file used 216 MB of memory, so the separate reading process (next step of 0.1) caps time and memory; PDFs over 512 MB (usually scans) are skipped until 0.5; right-to-left text from LibreOffice comes out backwards (Hebrew 2%, Arabic 16% found; Chrome's PDFs are fine), to be corrected and measured in 0.8 with the other languages. The reader sits behind Mulu's own interface, so PDFium can replace it if pdf_oxide stops being maintained. Metadata (title, author, year) is read in 0.6, with the catalog.

### D25. Files are read in a separate, limited process (developer)

A new component, `crates/mulu-reader`, runs every file reader in a **reading process**: Mulu's own program started with `--read-worker`. Mulu sends it one file path at a time and receives the text back; the reading process never writes anything and never opens a window.

- **Time:** each file has a limit (2 minutes; the 645-page test book takes about 1 second). Past it, Mulu stops the process, reports the file as "took too long" and starts a new process for the next file. The process also stops itself if it is still busy 10 seconds after the limit, so it never outlives Mulu; and it ends when Mulu closes.
- **Memory:** every allocation of the reading process is counted, with an allowance of 1 GB; past it, the allocation is refused and the process ends; the file is reported as too large. One damaged PDF in the D24 tests needed 216 MB.
- **Crash:** any other stop is reported for that file only, and the next file goes to a new process.
- **Priority:** the reading process runs at low priority (Windows "below normal"; `nice` on macOS and Linux), so the computer stays responsive while a library is indexed.

Alternatives considered for memory: the systems' own limits (Windows job objects, `setrlimit` on Linux and macOS). They need system calls written with unchecked ("unsafe") code in Mulu, and macOS does not enforce memory limits on a process, so a separate memory watcher would still be needed there. Counting allocations inside the process works the same on all three systems and needs no unsafe code in Mulu: it uses the small `cap` component, with no dependencies of its own. It covers everything the readers allocate, since they are all written in Rust. A process started per file was also considered: simpler, but starting a process costs tens of milliseconds on Windows, minutes over a library of tens of thousands of files.

Tests: a file over the memory allowance, a reader that runs out of memory while working, a file over the time limit, a reading process killed from outside, each followed by a normal file.

## Open

- Apple Developer Program enrollment (needed to notarize macOS builds): cost to be presented to the maintainer before the macOS test round.
