## Text quality: words a search would find on the right page

Share of the words of each page found on that same page, after the clean-up every Mulu reader does (hyphenated words joined, line breaks inside Chinese, Japanese and Thai removed). Words are compared as Mulu's search sees them (`mulu-text`).

### By producer

| | pdfium-rs | pdf_oxide | pdfium | hayro | pdf-extract | poppler |
|---|---|---|---|---|---|---|
| chrome | 95.3% | 99.1% | 95.2% | 68.9% | 83.4% | 98.2% |
| libreoffice | 97.0% | 93.5% | 97.0% | 84.4% | 78.2% | 97.6% |
| pdflatex | 100.0% | 100.0% | 99.6% | 98.9% | 100.0% | 100.0% |
| pdflatex-ot1 | 99.2% | 99.2% | 98.7% | 97.6% | 99.2% | 99.5% |
| reportlab | 99.4% | 99.4% | 99.4% | 21.0% | 17.8% | 17.8% |
| xelatex | 99.8% | 100.0% | 99.7% | 73.3% | 99.9% | 95.7% |
| xelatex-2col | 99.6% | 99.7% | 99.6% | 51.5% | 99.8% | 89.7% |

### By script

| | pdfium-rs | pdf_oxide | pdfium | hayro | pdf-extract | poppler |
|---|---|---|---|---|---|---|
| Arabic | 58.9% | 50.5% | 57.5% | 87.6% | 4.0% | 74.0% |
| Chinese | 100.0% | 100.0% | 100.0% | 40.6% | 70.5% | 79.5% |
| Cyrillic | 100.0% | 100.0% | 100.0% | 86.4% | 99.3% | 100.0% |
| Devanagari | 72.9% | 99.8% | 73.0% | 39.0% | 24.9% | 95.4% |
| Greek (polytonic) | 99.9% | 99.9% | 99.6% | 93.9% | 98.9% | 99.9% |
| Hebrew | 100.0% | 48.4% | 100.0% | 100.0% | 1.8% | 81.7% |
| Japanese | 99.2% | 99.2% | 99.2% | 35.4% | 62.4% | 66.1% |
| Korean | 98.3% | 98.7% | 98.3% | 37.1% | 65.7% | 65.5% |
| Latin | 99.9% | 99.9% | 99.7% | 90.1% | 99.8% | 98.0% |
| Thai | 95.9% | 99.9% | 95.9% | 66.5% | 65.9% | 99.3% |

### All files

| | pdfium-rs | pdf_oxide | pdfium | hayro | pdf-extract | poppler |
|---|---|---|---|---|---|---|
| all | 97.5% | 97.7% | 97.4% | 73.8% | 81.7% | 91.1% |

### Other measures, all files

| | pdfium-rs | pdf_oxide | pdfium | hayro | pdf-extract | poppler |
|---|---|---|---|---|---|---|
| Words found, anywhere in the file | 97.5% | 97.7% | 97.5% | 73.8% | 87.5% | 91.1% |
| Words found, raw text (no clean-up) | 96.8% | 96.9% | 96.7% | 73.0% | 86.7% | 90.9% |
| Precision (extracted words that are real) | 96.5% | 97.6% | 96.3% | 75.3% | 86.6% | 96.2% |
| Three-word phrases found (exact-phrase search) | 94.3% | 95.6% | 91.4% | 68.7% | 82.6% | 86.8% |
| Files it could not read | 0 | 0 | 0 | 0 | 3 | 0 |

## Speed and memory on a long book

Moby-Dick, 645 pages made with LibreOffice. Time includes starting the process (for PDFium, also starting Python); memory is the peak of the whole process, as reported by the engine itself (for PDFium, it includes Python; Poppler's was not measured). See the README for the cost of Python alone.

| | pdfium-rs | pdf_oxide | pdfium | hayro | pdf-extract | poppler |
|---|---|---|---|---|---|---|
| Time | 0.83 s | 1.07 s | 0.75 s | 1.50 s | 1.11 s | 0.55 s |
| Peak memory | 17 MB | 38 MB | 34 MB | 21 MB | 26 MB | not measured |
| Words found | 99.9% | 99.9% | 100.0% | 99.9% | 99.9% | 99.9% |

## Robustness on 1103 real-world PDFs (pdf.js test suite)

Many of these files are deliberately damaged or unusual: they come from bug reports. "Error" means the engine said it could not read the file (normal for a damaged or password-protected file); "crash" means the process died (panic, segmentation fault); "time limit" means it ran longer than 60 s.

| | pdfium-rs | pdf_oxide | pdfium | hayro | pdf-extract | poppler |
|---|---|---|---|---|---|---|
| Read | 1085 | 1089 | 1079 | 1085 | 1002 | 1080 |
| Error | 18 | 14 | 24 | 18 | 50 | 22 |
| Crash | 0 | 0 | 0 | 0 | 51 | 0 |
| Time limit | 0 | 0 | 0 | 0 | 0 | 1 |
| Median time per file | 6 ms | 5 ms | 69 ms | 5 ms | 5 ms | 11 ms |
| 95th percentile time | 27 ms | 26 ms | 95 ms | 19 ms | 33 ms | 31 ms |
| Median peak memory | 10 MB | 10 MB | 21 MB | 8 MB | 7 MB | not measured |
| Largest peak memory | 65 MB | 216 MB | 76 MB | 52 MB | 474 MB | not measured |

Crashes and time limits, by file:

- **pdfium-rs** (0): none
- **pdf_oxide** (0): none
- **pdfium** (0): none
- **hayro** (0): none
- **pdf-extract** (51): 90ms_rksj_h_sample.pdf, bug859204.pdf, bug860632.pdf, bug866395.pdf, issue10175.pdf, issue11549_reduced.pdf, issue11555.pdf, issue11768_reduced.pdf, issue11922_reduced.pdf, issue12823.pdf, issue13343.pdf, issue1350.pdf, issue14462_reduced.pdf, issue14821.pdf, issue18099_reduced.pdf, issue1985.pdf, issue20232.pdf, issue2128r.pdf, issue2391-1.pdf, issue2931.pdf, issue3521.pdf, issue3566.pdf, issue4402_reduced.pdf, issue4436r.pdf, issue4668.pdf, issue5701.pdf, issue5954.pdf, issue6081.pdf, issue6286.pdf, issue6342.pdf, issue6410.pdf, issue7598.pdf, issue7855.pdf, issue7872.pdf, issue8372.pdf, issue8702.pdf, issue9534_reduced.pdf, issue9915_reduced.pdf, mixedfonts.pdf, noembed-eucjp.pdf …
- **poppler** (1): bomb_giant.pdf

Files an engine could not read although at least three others could: pdfium-rs 4, pdf_oxide 7, pdfium 9, hayro 4, pdf-extract 88, poppler 11.
