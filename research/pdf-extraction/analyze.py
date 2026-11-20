"""Summarizes results/runs.jsonl into results/summary.md.
Usage: python3 analyze.py RESULTS_DIR"""

import json
import statistics
import sys
from collections import defaultdict
from pathlib import Path

ENGINES = ["pdfium-rs", "pdf_oxide", "pdfium", "hayro", "pdf-extract", "poppler"]
SCRIPT = {"por": "Latin", "spa": "Latin", "fra": "Latin", "deu": "Latin", "ita": "Latin", "lat": "Latin",
          "eng": "Latin", "ell": "Greek (polytonic)", "rus": "Cyrillic", "heb": "Hebrew", "arb": "Arabic",
          "hin": "Devanagari", "tha": "Thai", "zhs": "Chinese", "zht": "Chinese", "jpn": "Japanese", "kor": "Korean"}


def pct(a, b):
    return f"{100 * a / b:.1f}%" if b else "–"


def main():
    results = Path(sys.argv[1])
    rows = [json.loads(line) for line in open(results / "runs.jsonl")]
    corpus = [r for r in rows if r["set"] == "corpus"]
    real = [r for r in rows if r["set"] == "real"]
    out = []

    # ---- Text quality on the corpus with known text
    def table(title, key_of, keys):
        out.append(f"\n### {title}\n")
        out.append("| | " + " | ".join(ENGINES) + " |")
        out.append("|---|" + "---|" * len(ENGINES))
        for key in keys:
            cells = []
            for engine in ENGINES:
                found = total = 0
                for r in corpus:
                    if r["engine"] != engine or key_of(r) != key or r["file"].startswith("speed-"):
                        continue
                    truth = None
                    s = r.get("score")
                    # a failed file counts as nothing found
                    t = Path(results.parent / "corpus" / r["file"]).with_suffix(".truth.json")
                    if s:
                        found += s["tidy"]["found_on_right_page"]
                        total += s["tidy"]["truth_terms"]
                    else:
                        truth = truth_terms.get(r["file"], 0)
                        total += truth
                cells.append(pct(found, total))
            out.append(f"| {key} | " + " | ".join(cells) + " |")

    # truth term counts, for files an engine failed on, taken from any successful run
    truth_terms = {}
    for r in corpus:
        if r.get("score"):
            truth_terms[r["file"]] = r["score"]["tidy"]["truth_terms"]

    out.append("## Text quality: words a search would find on the right page\n")
    out.append("Share of the words of each page found on that same page, after the clean-up every "
               "Mulu reader does (hyphenated words joined, line breaks inside Chinese, Japanese and Thai "
               "removed). Words are compared as Mulu's search sees them (`mulu-text`).")
    producer = lambda r: r["file"].split("-")[0] + ("-ot1" if "-ot1-" in r["file"] else "") + ("-2col" if "-2col-" in r["file"] else "")
    table("By producer", producer, sorted({producer(r) for r in corpus if not r["file"].startswith("speed-")}))
    script = lambda r: SCRIPT.get(r["file"].rsplit("-", 1)[1].removesuffix(".pdf"), "?")
    table("By script", script, sorted(set(SCRIPT.values())))
    table("All files", lambda r: "all", ["all"])

    out.append("\n### Other measures, all files\n")
    out.append("| | " + " | ".join(ENGINES) + " |")
    out.append("|---|" + "---|" * len(ENGINES))
    lines = defaultdict(list)
    for engine in ENGINES:
        agg = defaultdict(int)
        fails = 0
        for r in corpus:
            if r["engine"] != engine or r["file"].startswith("speed-"):
                continue
            if not r.get("score"):
                fails += 1
                continue
            for mode in ("raw", "tidy"):
                for k, v in r["score"][mode].items():
                    agg[f"{mode}.{k}"] += v
        lines["Words found, anywhere in the file"].append(pct(agg["tidy.found_anywhere"], agg["tidy.truth_terms"]))
        lines["Words found, raw text (no clean-up)"].append(pct(agg["raw.found_on_right_page"], agg["raw.truth_terms"]))
        lines["Precision (extracted words that are real)"].append(pct(agg["tidy.found_anywhere"], agg["tidy.output_terms"]))
        lines["Three-word phrases found (exact-phrase search)"].append(pct(agg["tidy.phrases_found"], agg["tidy.phrases"]))
        lines["Files it could not read"].append(str(fails))
    for k, v in lines.items():
        out.append(f"| {k} | " + " | ".join(v) + " |")

    # ---- Speed on the long novel
    out.append("\n## Speed and memory on a long book\n")
    out.append("Moby-Dick, 645 pages made with LibreOffice. Time includes starting the process "
               "(for PDFium, also starting Python); memory is the peak of the whole process, as reported "
               "by the engine itself (for PDFium, it includes Python; Poppler's was not measured). "
               "See the README for the cost of Python alone.\n")
    out.append("| | " + " | ".join(ENGINES) + " |")
    out.append("|---|" + "---|" * len(ENGINES))
    speed = {r["engine"]: r for r in corpus if r["file"].startswith("speed-")}
    out.append("| Time | " + " | ".join(f'{speed[e]["seconds"]:.2f} s' if e in speed else "–" for e in ENGINES) + " |")
    out.append("| Peak memory | " + " | ".join(f'{speed[e]["max_rss_kb"] / 1024:.0f} MB' if e in speed and speed[e]["max_rss_kb"] else "not measured" for e in ENGINES) + " |")
    out.append("| Words found | " + " | ".join(
        pct(speed[e]["score"]["tidy"]["found_on_right_page"], speed[e]["score"]["tidy"]["truth_terms"]) if e in speed and speed[e].get("score") else "–"
        for e in ENGINES) + " |")

    # ---- Robustness on real-world files
    files = sorted({r["file"] for r in real})
    out.append(f"\n## Robustness on {len(files)} real-world PDFs (pdf.js test suite)\n")
    out.append("Many of these files are deliberately damaged or unusual: they come from bug reports. "
               "\"Error\" means the engine said it could not read the file (normal for a damaged or "
               "password-protected file); \"crash\" means the process died (panic, segmentation fault); "
               f"\"time limit\" means it ran longer than 60 s.\n")
    out.append("| | " + " | ".join(ENGINES) + " |")
    out.append("|---|" + "---|" * len(ENGINES))
    by = defaultdict(lambda: defaultdict(list))
    for r in real:
        by[r["engine"]][r["status"]].append(r)
    for status, label in [("ok", "Read"), ("error", "Error"), ("crash", "Crash"), ("timeout", "Time limit")]:
        out.append(f"| {label} | " + " | ".join(str(len(by[e][status])) for e in ENGINES) + " |")
    ok_text = defaultdict(dict)
    for r in real:
        ok_text[r["file"]][r["engine"]] = r["status"]
    for label, fn in [
        ("Median time per file", lambda rs: f'{statistics.median([r["seconds"] for r in rs]) * 1000:.0f} ms'),
        ("95th percentile time", lambda rs: f'{statistics.quantiles([r["seconds"] for r in rs], n=20)[-1] * 1000:.0f} ms'),
        ("Median peak memory", lambda rs: f'{statistics.median(m) / 1024:.0f} MB' if (m := [r["max_rss_kb"] for r in rs if r["max_rss_kb"]]) else "not measured"),
        ("Largest peak memory", lambda rs: f'{max(m) / 1024:.0f} MB' if (m := [r["max_rss_kb"] for r in rs if r["max_rss_kb"]]) else "not measured"),
    ]:
        out.append(f"| {label} | " + " | ".join(fn(by[e]["ok"]) if by[e]["ok"] else "–" for e in ENGINES) + " |")
    out.append("\nCrashes and time limits, by file:\n")
    for e in ENGINES:
        bad = sorted(r["file"] for r in by[e]["crash"] + by[e]["timeout"])
        out.append(f"- **{e}** ({len(bad)}): " + (", ".join(bad[:40]) + (" …" if len(bad) > 40 else "") if bad else "none"))
    only_others = defaultdict(int)
    for f, st in ok_text.items():
        for e in ENGINES:
            if st.get(e) != "ok" and sum(1 for x in ENGINES if st.get(x) == "ok") >= 3:
                only_others[e] += 1
    out.append("\nFiles an engine could not read although at least three others could: "
               + ", ".join(f"{e} {only_others[e]}" for e in ENGINES) + ".")

    (results / "summary.md").write_text("\n".join(out) + "\n")
    print("\n".join(out))


if __name__ == "__main__":
    main()
