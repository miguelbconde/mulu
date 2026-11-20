"""Runs every engine on every PDF, each in its own process with a time limit, and
records the outcome, time, peak memory and (for the test corpus) the text scores.

Usage: python3 run.py CORPUS_DIR REAL_WORLD_DIR RESULTS_DIR
CORPUS_DIR holds the PDFs made by make_corpus.py, with their .truth.json files.
REAL_WORLD_DIR holds real PDFs without known text (we used the 1,103 PDFs of the
pdf.js test suite, https://github.com/mozilla/pdf.js/tree/master/test/pdfs): only
crashes, time limits, time and memory are measured there.
"""

import json
import os
import signal
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent
HARNESS = HERE / "target" / "release" / "pdf-extraction"
TIMEOUT = 60  # seconds per file

ENGINES = {
    "pdf_oxide": lambda pdf: [str(HARNESS), "extract", "pdf_oxide", pdf],
    "pdf-extract": lambda pdf: [str(HARNESS), "extract", "pdf-extract", pdf],
    "hayro": lambda pdf: [str(HARNESS), "extract", "hayro", pdf],
    "pdfium": lambda pdf: [sys.executable, str(HERE / "pdfium_extract.py"), pdf],
    "poppler": None,  # reference only: pdftotext, pages split at form feeds
}


def run_one(engine: str, pdf: str) -> dict:
    with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as err:
        command = ENGINES[engine](pdf) if ENGINES[engine] else ["pdftotext", "-enc", "UTF-8", pdf, "-"]
        start = time.monotonic()
        process = subprocess.Popen(command, stdout=out, stderr=err, start_new_session=True)
        status = "ok"
        deadline = start + TIMEOUT
        while True:
            pid, code, usage = os.wait4(process.pid, os.WNOHANG)
            if pid:
                break
            if time.monotonic() > deadline:
                os.killpg(process.pid, signal.SIGKILL)
                pid, code, usage = os.wait4(process.pid, 0)
                status = "timeout"
                break
            time.sleep(0.005)
        seconds = time.monotonic() - start
        if status == "ok":
            if os.WIFSIGNALED(code):
                status = "crash"  # killed by a signal: segmentation fault, abort...
            elif os.WEXITSTATUS(code) == 101:
                status = "crash"  # Rust panic
            elif os.WEXITSTATUS(code) != 0:
                status = "error"  # the engine reported that it could not read the file
        out.seek(0)
        data = out.read().decode("utf-8", errors="replace")
        if status == "ok" and engine == "poppler":
            pages = data.split("\f")
            if pages and pages[-1].strip() == "":
                pages.pop()
            data = json.dumps({"pages": pages}, ensure_ascii=False)
        err.seek(0)
        errors = err.read().decode("utf-8", errors="replace")
        # Engines report their own peak memory; the counter seen from here would include
        # the memory of this Python process at the time it started the engine.
        peak = [line for line in errors.splitlines() if line.startswith("MAXRSS_KB=")]
        return {
            "engine": engine,
            "file": Path(pdf).name,
            "status": status,
            "seconds": round(seconds, 4),
            "max_rss_kb": int(peak[-1].split("=")[1]) if peak else None,
            "stderr": errors[-300:] if status != "ok" else "",
            "output": data if status == "ok" else "",
        }


def score(truth: Path, output: str) -> dict:
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f:
        f.write(output)
        name = f.name
    try:
        result = subprocess.run([str(HARNESS), "score", str(truth), name], capture_output=True, timeout=120)
        return json.loads(result.stdout)
    finally:
        os.unlink(name)


def main():
    corpus, real_world, results = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
    results.mkdir(parents=True, exist_ok=True)
    jobs = []
    for pdf in sorted(corpus.glob("*.pdf")):
        jobs += [("corpus", engine, str(pdf)) for engine in ENGINES]
    for pdf in sorted(real_world.glob("*.pdf")):
        jobs += [("real", engine, str(pdf)) for engine in ENGINES]

    def work(job):
        kind, engine, pdf = job
        row = run_one(engine, pdf)
        row["set"] = kind
        if kind == "corpus":
            row["score"] = score(Path(pdf).with_suffix(".truth.json"), row["output"]) if row["status"] == "ok" else None
            if row["status"] == "ok":
                (results / "outputs").mkdir(exist_ok=True)
                (results / "outputs" / f"{Path(pdf).stem}.{engine}.json").write_text(row["output"])
        del row["output"]
        return row

    with ThreadPoolExecutor(max_workers=2) as pool, open(results / "runs.jsonl", "w") as log:
        for i, row in enumerate(pool.map(work, jobs)):
            log.write(json.dumps(row, ensure_ascii=False) + "\n")
            if i % 200 == 0:
                print(f"{i}/{len(jobs)}", flush=True)


if __name__ == "__main__":
    main()
