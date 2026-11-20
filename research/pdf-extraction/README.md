# Which engine should Mulu use to read PDFs? (decision D24)

*Experiment run on 2026-11-20. Result: Mulu reads PDFs with [pdf_oxide](https://github.com/yfedoseev/pdf_oxide) 0.3.78, inside its isolated reading process, with PDFium kept as the measured fallback. See [DECISIONS.md](../../DECISIONS.md), D24.*

## The question

PDF is the main format of most personal libraries. A PDF stores glyphs at positions, not words and lines; the reader has to rebuild the text. What matters for Mulu is search: **are the words of each page found, on that page, and are phrases kept in order?** Also: does the reader survive damaged and hostile files, how fast is it, how much memory and installer space does it cost, and how does it fit Mulu's principles (memory-safe languages, few dependencies, no network, open licenses)?

## Engines compared

| Engine | Language | License | How it would ship |
|---|---|---|---|
| [pdf_oxide](https://crates.io/crates/pdf_oxide) 0.3.78 | Rust | MIT or Apache-2.0 | inside Mulu's program |
| [PDFium](https://pdfium.googlesource.com/pdfium/) (the engine in Chrome), through [pypdfium2](https://pypi.org/project/pypdfium2/) 5.13 here, [pdfium-render](https://crates.io/crates/pdfium-render) in Mulu | C++ | BSD-3-Clause / Apache-2.0 | a separate library file, prebuilt |
| [hayro](https://github.com/LaurenzV/hayro) 0.9 (interpreter) + a simple layout written for this test ([`src/hayro_text.rs`](src/hayro_text.rs)) | Rust | MIT or Apache-2.0 | inside Mulu's program |
| [pdf-extract](https://crates.io/crates/pdf-extract) 0.12.1 (on lopdf) | Rust | MIT | inside Mulu's program |
| Poppler `pdftotext` (reference only) | C++ | GPL | not an option: heavy to ship on Windows and macOS |

Left out: MuPDF (AGPL, C) and engines that need a network service or a large AI model.

## Test material

1. **61 PDFs with known text** ([`make_corpus.py`](make_corpus.py)). The text is the Universal Declaration of Human Rights in 17 languages and scripts (Latin languages, polytonic Greek, Russian, Hebrew, Arabic, Hindi, Thai, Chinese simplified and traditional, Japanese, Korean), one chunk per page, so the right text of every page is known. The PDFs are made the main ways real PDFs are made: **LibreOffice**, **Google Chrome** (printing a web page), **XeLaTeX** (one and two columns), **pdfLaTeX** with modern fonts and with the old OT1 encoding of many academic PDFs, and **ReportLab** (programs that generate PDFs, including Chinese, Japanese and Korean with fonts not embedded). Plus Moby-Dick, 645 pages, for speed.
2. **1,103 real PDFs** from the [pdf.js test suite](https://github.com/mozilla/pdf.js/tree/master/test/pdfs): many come from bug reports and are damaged, unusual or hostile on purpose. Their text is not known; they measure crashes, time limits, time and memory.

Neither set is redistributed here: the script rebuilds the first; the second is in the pdf.js repository.

## Method

[`run.py`](run.py) runs every engine on every file in its own process, with a 60-second limit. [`src/main.rs`](src/main.rs) splits both the known text and the extracted text into words exactly as Mulu's search does (`mulu-text`: case, accents, Unicode word boundaries, character pairs for Chinese and Japanese) and counts:

- **words found on the right page**: the main measure, what a search would find and attribute to the right page;
- words found anywhere in the file; **precision** (share of extracted words that are real);
- **three-word phrases found**: what an exact-phrase search needs (word order, no words glued or split).

Before counting, the same clean-up is applied to every engine, the one Mulu's reader will do: words hyphenated at the end of a line are joined; line breaks inside Chinese, Japanese or Thai text are removed; accents that old TeX fonts draw as separate characters (`Na¸c˜oes`) are put back on their letter. Raw figures without clean-up are also reported.

[`analyze.py`](analyze.py) writes [`results/summary.md`](results/summary.md) from [`results/runs.jsonl`](results/runs.jsonl).

## Results

Full tables in [`results/summary.md`](results/summary.md).

| | pdf_oxide | PDFium | hayro + simple layout | pdf-extract | Poppler (reference) |
|---|---|---|---|---|---|
| Words found on the right page, all 61 files | **97.7%** | 97.4% | 73.8% | 81.7% | 91.1% |
| Three-word phrases found | **95.6%** | 91.4% | 68.7% | 82.6% | 86.8% |
| Precision | **97.6%** | 96.3% | 75.3% | 86.6% | 96.2% |
| Real PDFs read (of 1,103) | **1,089** | 1,079 | 1,085 | 1,002 | 1,080 |
| Crashes | **0** | **0** | **0** | 51 | 0 |
| Time limits (60 s) | **0** | **0** | **0** | 0 | 1 |
| Median time per real PDF | **5 ms** | 69 ms¹ | 5 ms | 5 ms | 11 ms |
| 645-page book | 1.07 s | 0.75 s¹ | 1.50 s | 1.11 s | 0.55 s |
| Largest peak memory, real PDFs | 216 MB | 76 MB¹ | 52 MB | 474 MB | – |
| Added to the program, Windows-style size compressed² | ≈ 3.2 MB | ≈ 3.9 MB (`pdfium.dll`) | – | ≈ 0.7 MB | – |
| New components in Mulu³ | 53 Rust components | pdfium-render and its helpers, plus a C++ library | – | – | – |

¹ Measured through Python; Python alone takes about 22 MB and about 0.1 s to start. ² pdf_oxide and pdf-extract: the code they add to a small Rust program on Linux, compressed; PDFium: `pdfium-win-x64.tgz`, chromium/8165, from [pdfium-binaries](https://github.com/bblanchon/pdfium-binaries). ³ Counted in Mulu's own list of components, with pdf_oxide's optional parts turned off (only old-encryption support kept).

By script (words found on the right page):

| | pdf_oxide | PDFium | hayro | pdf-extract | Poppler |
|---|---|---|---|---|---|
| Latin | 99.9% | 99.7% | 90.1% | 99.8% | 98.0% |
| Greek (polytonic) | 99.9% | 99.6% | 93.9% | 98.9% | 99.9% |
| Cyrillic | 100.0% | 100.0% | 86.4% | 99.3% | 100.0% |
| Chinese | 100.0% | 100.0% | 40.6% | 70.5% | 79.5% |
| Japanese | 99.2% | 99.2% | 35.4% | 62.4% | 66.1% |
| Korean | 98.7% | 98.3% | 37.1% | 65.7% | 65.5% |
| Thai | 99.9% | 95.9% | 66.5% | 65.9% | 99.3% |
| Devanagari (Hindi) | 99.8% | 73.0% | 39.0% | 24.9% | 95.4% |
| Hebrew | 48.4% | 100.0% | 100.0% | 1.8% | 81.7% |
| Arabic | 50.5% | 57.5% | 87.6% | 4.0% | 74.0% |

## Reading the results

- **pdf_oxide and PDFium are clearly ahead**, and close to each other: about 97–98% of words on the right page, no crash and no time limit on 1,103 hostile files. pdf_oxide keeps phrases better (95.6% against 91.4%) and reads Hindi and Thai better; PDFium reads Hebrew and Arabic better.
- **Right-to-left scripts are weak everywhere.** pdf_oxide reads the Hebrew and Arabic made by LibreOffice in visual order (each line backwards): 2% and 16% found; Chrome's PDFs are fine (95% and 85%). PDFium fails on Chrome's Arabic (37%). No engine is good at both; this is measured again, and corrected, with the other languages in version 0.8 (D15).
- **hayro** is a careful, memory-safe interpreter, but it gives glyphs, not text: words, lines and reading order would be Mulu's own code to write and maintain. The simple layout written for this test reaches 74%; closing the gap is a project in itself.
- **pdf-extract** crashed on 51 real files and missed most non-Latin text.
- **Old TeX PDFs** (OT1 encoding) lose accents in every engine but Poppler; the shared clean-up recovers them (88% → 99%).

## Limits

- The text corpus is synthetic: real libraries also hold scanned books (OCR, version 0.5), PDFs with broken font tables and complex layouts (footnotes, marginalia). The 1,103 real files measure robustness, not text quality.
- PDFium was measured through Python, not through pdfium-render; same engine, but times and memory include Python.
- One run on a 2-core cloud machine; times are indicative.
- pdf_oxide's own benchmarks were not used; every number here was measured by this script.
