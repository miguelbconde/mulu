"""Text of each page with PDFium (the engine in Chrome), through pypdfium2.
Prints {"pages": [...]} like the Rust harness. Usage: python3 pdfium_extract.py FILE.pdf"""

import json
import sys

import pypdfium2 as pdfium

document = pdfium.PdfDocument(sys.argv[1])
pages = []
for page in document:
    textpage = page.get_textpage()
    pages.append(textpage.get_text_range())
    textpage.close()
    page.close()
print(json.dumps({"pages": pages}, ensure_ascii=False))

# Peak memory of this process (see report_peak_memory in src/main.rs).
for line in open("/proc/self/status"):
    if line.startswith("VmHWM:"):
        print(f"MAXRSS_KB={line.split()[1]}", file=sys.stderr)
