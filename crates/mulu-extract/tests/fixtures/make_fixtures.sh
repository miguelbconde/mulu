#!/bin/sh
# Rebuilds the small PDF test files. Needs pdfLaTeX (with hyperref), Google Chrome,
# qpdf and img2pdf. The texts are original or from the Universal Declaration of Human
# Rights (public domain).
set -e
cd "$(dirname "$0")"
work=$(mktemp -d)

# A book with a preface numbered i, ii and chapters numbered from 1, a word hyphenated
# across lines, and old-style (OT1) accents.
cat > "$work/book.tex" <<'TEX'
\documentclass[11pt,a5paper]{article}
\usepackage[utf8]{inputenc}
\usepackage{hyperref}
\pagestyle{plain}
\begin{document}
\pagenumbering{roman}
Prefácio: a melancolia dos livros esquecidos.
\newpage
Segunda página do prefácio.
\newpage
\pagenumbering{arabic}
Considerando que o reconhecimento da dignidade inerente a todos os membros da família humana é o fundamento da liber-\\
dade, da justiça e da paz no mundo.
\newpage
A liberdade de pensamento e de expressão.
\end{document}
TEX
(cd "$work" && pdflatex -interaction=nonstopmode book.tex >/dev/null)
cp "$work/book.pdf" book.pdf

# Chinese text, made by Chrome.
cat > "$work/zh.html" <<'HTML'
<!DOCTYPE html><html lang="zh"><meta charset="utf-8"><style>body{font-family:'Noto Serif CJK SC';width:8em}</style>
<p>人人生而自由，在尊严和权利上一律平等。他们赋有理性和良心，并应以兄弟关系的精神相对待。</p></html>
HTML
google-chrome --headless --no-sandbox --disable-gpu --no-pdf-header-footer --print-to-pdf="$PWD/chinese.pdf" "file://$work/zh.html" 2>/dev/null

# Protected: needs a password to open. Restricted: opens without one (copy limits only).
qpdf --encrypt secreto dono 256 -- book.pdf password.pdf
# Old 128-bit RC4 encryption, common in older PDFs (needs the "legacy-crypto" support).
qpdf --allow-weak-crypto --encrypt "" dono 128 --print=none --extract=n --use-aes=n -- book.pdf restricted.pdf

# A scan: one page that is only an image.
python3 -c "import zlib,struct
w,h=60,40
raw=b''.join(b'\x00'+bytes([255])*w for _ in range(h))
def chunk(t,d): return struct.pack('>I',len(d))+t+d+struct.pack('>I',zlib.crc32(t+d))
open('$work/s.png','wb').write(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,0,0,0,0))+chunk(b'IDAT',zlib.compress(raw))+chunk(b'IEND',b''))"
img2pdf "$work/s.png" -o scan.pdf
rm -rf "$work"
