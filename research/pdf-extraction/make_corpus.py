"""Builds the PDF test corpus for the PDF reader comparison (decision D24).

Every PDF is made from a text whose words are known, one chunk per page, so the
extracted text of each page can be compared with what is really on it.

Texts: the Universal Declaration of Human Rights in 17 languages and scripts (public
domain, from the NLTK "udhr2" corpus), plus a public-domain novel for the speed test.
Producers: LibreOffice, Google Chrome, XeLaTeX, pdfLaTeX (modern T1 and old OT1 font
encodings) and ReportLab, so that the corpus covers the main ways PDFs are made.

Usage: python3 make_corpus.py DOWNLOAD_DIR OUTPUT_DIR
DOWNLOAD_DIR must hold the unpacked NLTK corpora "udhr2" and "gutenberg":
  https://raw.githubusercontent.com/nltk/nltk_data/gh-pages/packages/corpora/udhr2.zip
  https://raw.githubusercontent.com/nltk/nltk_data/gh-pages/packages/corpora/gutenberg.zip
Requires: LibreOffice, TeX Live (xelatex, pdflatex, polyglossia, babel), Google
Chrome, ReportLab, and the Noto CJK, FreeFont and DejaVu fonts.
"""

import html
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

# (corpus name, udhr2 file, BCP 47 tag, script kind)
LANGUAGES = [
    ("por", "por_BR", "pt-BR", "latin"),
    ("spa", "spa", "es", "latin"),
    ("fra", "fra", "fr", "latin"),
    ("deu", "deu", "de", "latin"),
    ("ita", "ita", "it", "latin"),
    ("lat", "lat", "la", "latin"),
    ("eng", "eng", "en", "latin"),
    ("ell", "ell_polytonic", "el", "greek"),
    ("rus", "rus", "ru", "cyrillic"),
    ("heb", "heb", "he", "rtl"),
    ("arb", "arb", "ar", "rtl"),
    ("hin", "hin", "hi", "indic"),
    ("tha", "tha", "th", "unspaced"),
    ("zhs", "cmn_hans", "zh-Hans", "cjk"),
    ("zht", "cmn_hant", "zh-Hant", "cjk"),
    ("jpn", "jpn", "ja", "cjk"),
    ("kor", "kor", "ko", "latin"),  # Korean separates words with spaces
]

UNSPACED = {"cjk", "unspaced"}


def paragraphs(path: Path) -> list[str]:
    text = path.read_text(encoding="utf-8", errors="replace")
    paras = [re.sub(r"\s+", " ", p).strip() for p in re.split(r"\n\s*\n|\n\t|\n {2,}", text)]
    return [p for p in paras if p and "�" not in p]


def chunk(paras: list[str], kind: str, size: int) -> list[list[str]]:
    """Groups paragraphs into pages of roughly `size` words (or 2.5×size characters
    for scripts without spaces)."""
    measure = (lambda p: len(p) / 2.5) if kind in UNSPACED else (lambda p: len(p.split()))
    pages, current, used = [], [], 0.0
    for p in paras:
        if current and used + measure(p) > size:
            pages.append(current)
            current, used = [], 0.0
        current.append(p)
        used += measure(p)
    if current:
        pages.append(current)
    return pages


def write_truth(out: Path, name: str, pages: list[list[str]], producer: str, lang: str):
    truth = {"producer": producer, "lang": lang, "pages": ["\n".join(p) for p in pages]}
    (out / f"{name}.truth.json").write_text(json.dumps(truth, ensure_ascii=False, indent=1))


# ---------------------------------------------------------------- HTML: LibreOffice, Chrome

FONT_FOR = {
    "cjk": "'Noto Serif CJK SC', 'Noto Sans CJK SC'",
    "rtl": "'DejaVu Sans', 'FreeSerif'",
    "indic": "'FreeSerif', 'FreeSans'",
    "unspaced": "'Norasi', 'Loma', 'FreeSerif'",
}


def page_html(pages: list[list[str]], lang: str, kind: str) -> str:
    direction = "rtl" if kind == "rtl" else "ltr"
    font = FONT_FOR.get(kind, "'Liberation Serif', 'DejaVu Serif', 'FreeSerif'")
    body = []
    for i, page in enumerate(pages):
        # On the first paragraph of the page: LibreOffice honors page breaks only there.
        for j, p in enumerate(page):
            style = ' style="page-break-before: always"' if i and not j else ""
            body.append(f"<p{style}>{html.escape(p)}</p>")
    return (
        f'<!DOCTYPE html><html lang="{lang}" dir="{direction}"><head><meta charset="utf-8">'
        f"<style>@page {{ size: A4; margin: 2cm }} body {{ font-family: {font}; font-size: 11pt;"
        f" text-align: justify }}</style></head><body>{''.join(body)}</body></html>"
    )


def libreoffice(html_text: str, out_pdf: Path, work: Path):
    source = work / (out_pdf.stem + ".html")
    source.write_text(html_text, encoding="utf-8")
    subprocess.run(
        ["soffice", "--headless", "--infilter=HTML (StarWriter)", "--convert-to", "pdf:writer_pdf_Export", "--outdir", str(work), str(source)],
        check=True, capture_output=True, timeout=300,
    )
    shutil.move(work / (out_pdf.stem + ".pdf"), out_pdf)


def chrome(html_text: str, out_pdf: Path, work: Path):
    source = work / (out_pdf.stem + ".html")
    source.write_text(html_text, encoding="utf-8")
    subprocess.run(
        ["google-chrome", "--headless", "--no-sandbox", "--disable-gpu", "--no-pdf-header-footer",
         f"--print-to-pdf={out_pdf}", source.as_uri()],
        check=True, capture_output=True, timeout=300,
    )


# ---------------------------------------------------------------- LaTeX

LATEX_SPECIAL = {
    "\\": r"\textbackslash{}", "{": r"\{", "}": r"\}", "$": r"\$", "&": r"\&", "#": r"\#",
    "_": r"\_", "%": r"\%", "~": r"\textasciitilde{}", "^": r"\textasciicircum{}",
}


def tex_escape(text: str) -> str:
    return "".join(LATEX_SPECIAL.get(c, c) for c in text)


POLYGLOSSIA = {"pt-BR": "portuguese", "es": "spanish", "fr": "french", "de": "german",
               "it": "italian", "la": "latin", "en": "english", "el": "greek", "ru": "russian"}
BABEL = {"pt-BR": "brazil", "es": "spanish", "fr": "french", "de": "ngerman", "it": "italian",
         "en": "english", "la": "latin"}


def latex(pages: list[list[str]], out_pdf: Path, work: Path, engine: str, lang: str,
          two_columns: bool = False, ot1: bool = False):
    body = "\n\\newpage\n".join("\n\n".join(tex_escape(p) for p in page) for page in pages)
    columns = "\\twocolumn" if two_columns else ""
    if engine == "xelatex":
        extra = "variant=polytonic" if lang == "el" else ""
        preamble = (
            "\\usepackage{fontspec}\\usepackage{polyglossia}"
            f"\\setdefaultlanguage[{extra}]{{{POLYGLOSSIA[lang]}}}"
            "\\setmainfont{FreeSerif}[Ligatures=TeX]"
        )
    else:
        encoding = "" if ot1 else "\\usepackage[T1]{fontenc}\\usepackage{lmodern}"
        # Babel language files are not installed for pdfLaTeX here: default (English)
        # hyphenation, which still breaks words across lines as real documents do.
        preamble = f"\\usepackage[utf8]{{inputenc}}{encoding}"
    source = (
        f"\\documentclass[11pt,a4paper]{{article}}{preamble}\\usepackage[margin=2cm]{{geometry}}"
        f"\\pagestyle{{empty}}\\begin{{document}}{columns}\n{body}\n\\end{{document}}\n"
    )
    tex = work / (out_pdf.stem + ".tex")
    tex.write_text(source, encoding="utf-8")
    result = subprocess.run([engine, "-interaction=nonstopmode", "-halt-on-error", tex.name],
                            cwd=work, capture_output=True, timeout=600)
    if result.returncode != 0:
        raise RuntimeError(result.stdout.decode(errors="replace")[-1500:])
    shutil.move(work / (out_pdf.stem + ".pdf"), out_pdf)


# ---------------------------------------------------------------- ReportLab

def reportlab(pages: list[list[str]], out_pdf: Path, font: str, kind: str):
    from reportlab.lib.pagesizes import A4
    from reportlab.lib.styles import ParagraphStyle
    from reportlab.pdfbase import pdfmetrics
    from reportlab.pdfbase.cidfonts import UnicodeCIDFont
    from reportlab.platypus import PageBreak, Paragraph, SimpleDocTemplate

    if font not in ("Helvetica", "Times-Roman"):
        pdfmetrics.registerFont(UnicodeCIDFont(font))
    style = ParagraphStyle("body", fontName=font, fontSize=11, leading=15,
                           wordWrap="CJK" if kind in UNSPACED else None)
    story = []
    for i, page in enumerate(pages):
        if i:
            story.append(PageBreak())
        story += [Paragraph(html.escape(p), style) for p in page]
    SimpleDocTemplate(str(out_pdf), pagesize=A4).build(story)


# ---------------------------------------------------------------- main

def main():
    downloads, out = Path(sys.argv[1]), Path(sys.argv[2])
    out.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp())
    made = []

    def make(name, pages, producer, lang, build):
        pdf = out / f"{name}.pdf"
        try:
            build(pdf)
        except Exception as error:  # a producer that fails is reported, not fatal
            print(f"SKIPPED {name}: {str(error)[-300:]}", file=sys.stderr)
            return
        write_truth(out, name, pages, producer, lang)
        made.append(name)

    for short, source, lang, kind in LANGUAGES:
        paras = paragraphs(downloads / "udhr2" / f"{source}.txt")
        pages = chunk(paras, kind, 220)
        doc = page_html(pages, lang, kind)
        make(f"libreoffice-{short}", pages, "LibreOffice", lang, lambda p: libreoffice(doc, p, work))
        make(f"chrome-{short}", pages, "Chrome", lang, lambda p: chrome(doc, p, work))
        if lang in POLYGLOSSIA:
            make(f"xelatex-{short}", pages, "XeLaTeX", lang,
                 lambda p: latex(pages, p, work, "xelatex", lang))
        if lang in BABEL:
            make(f"pdflatex-{short}", pages, "pdfLaTeX T1", lang,
                 lambda p: latex(pages, p, work, "pdflatex", lang))

    # Harder layouts and older conventions.
    for short, source, lang in [("por", "por_BR", "pt-BR"), ("fra", "fra", "fr"), ("deu", "deu", "de")]:
        paras = paragraphs(downloads / "udhr2" / f"{source}.txt")
        pages = chunk(paras, "latin", 220)
        make(f"pdflatex-ot1-{short}", pages, "pdfLaTeX OT1", lang,
             lambda p: latex(pages, p, work, "pdflatex", lang, ot1=True))
        wide = chunk(paras, "latin", 520)
        make(f"xelatex-2col-{short}", wide, "XeLaTeX two columns", lang,
             lambda p: latex(wide, p, work, "xelatex", lang, two_columns=True))
    por = chunk(paragraphs(downloads / "udhr2" / "por_BR.txt"), "latin", 220)
    make("reportlab-por", por, "ReportLab", "pt-BR", lambda p: reportlab(por, p, "Helvetica", "latin"))
    for short, source, lang, font in [("zhs", "cmn_hans", "zh-Hans", "STSong-Light"),
                                      ("jpn", "jpn", "ja", "HeiseiMin-W3"),
                                      ("kor", "kor", "ko", "HYSMyeongJo-Medium")]:
        pages = chunk(paragraphs(downloads / "udhr2" / f"{source}.txt"), "cjk", 220)
        make(f"reportlab-{short}", pages, "ReportLab, CJK font not embedded", lang,
             lambda p: reportlab(pages, p, font, "cjk"))

    # Speed test: a whole novel, one paragraph per line kept, pages of ~400 words.
    novel = [p for p in (downloads / "gutenberg" / "melville-moby_dick.txt").read_text().split("\n\n")]
    novel = [re.sub(r"\s+", " ", p).strip() for p in novel if p.strip()]
    pages = chunk(novel, "latin", 400)
    doc = page_html(pages, "en", "latin")
    make("speed-libreoffice-moby-dick", pages, "LibreOffice", "en", lambda p: libreoffice(doc, p, work))

    print(f"{len(made)} PDFs in {out}")


if __name__ == "__main__":
    main()
