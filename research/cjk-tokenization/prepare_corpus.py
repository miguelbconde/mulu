#!/usr/bin/env python3
"""Prepare the corpora and queries for Mulu decision D9 (Chinese tokenization).

Usage:
    pip install opencc==1.4.2
    python3 prepare_corpus.py OUT_DIR JIEBA_DICT_TXT

OUT_DIR receives four corpora (one document per line) and one query file per corpus.
JIEBA_DICT_TXT is the dictionary shipped with jieba-rs 0.11.0
(src/data/dict.txt in the crate source), used only to sample real words as queries.

Sources (downloaded by this script, not redistributed by Mulu):
- Qian Zhongshu, "Fortress Besieged" (围城), the benchmark text of the jieba-rs project:
  https://github.com/messense/jieba-rs (examples/weicheng/src/weicheng.txt)
- Tang poems and the Analects, Mencius and Great Learning from the
  chinese-poetry project (MIT license, public-domain texts):
  https://github.com/chinese-poetry/chinese-poetry
"""
import json
import os
import random
import re
import sys
import urllib.parse
import urllib.request

import opencc

WEICHENG = "https://raw.githubusercontent.com/messense/jieba-rs/main/examples/weicheng/src/weicheng.txt"
POETRY = "https://raw.githubusercontent.com/chinese-poetry/chinese-poetry/master/"
POETRY_FILES = {
    "tang": ["全唐诗/poet.tang.0.json", "全唐诗/poet.tang.1000.json"],
    "classics": ["论语/lunyu.json", "四书五经/mengzi.json", "四书五经/daxue.json"],
}
HAN_RUN = re.compile(r"[㐀-䶿一-鿿豈-﫿]+")


def fetch(url):
    with urllib.request.urlopen(url) as r:
        return r.read().decode("utf-8")


def poetry(path):
    return json.loads(fetch(POETRY + urllib.parse.quote(path)))


def chunk_prose(paragraphs, size=150):
    """Group sentences into documents of about `size` characters."""
    out = []
    for p in paragraphs:
        cur = ""
        for s in re.split(r"(?<=[。！？])", p):
            cur += s
            if len(cur) >= size:
                out.append(cur)
                cur = ""
        if len(cur) >= 20:
            out.append(cur)
    return out


def main(out_dir, dict_path):
    os.makedirs(out_dir, exist_ok=True)
    to_trad = opencc.OpenCC("s2twp")

    prose = [l.strip().replace("　", "") for l in fetch(WEICHENG).splitlines()]
    prose = chunk_prose([p for p in prose if len(p) >= 20])
    tang = []
    for f in POETRY_FILES["tang"]:
        for poem in poetry(f):
            s = "".join(poem["paragraphs"]).strip()
            if len(s) >= 10:
                tang.append(s)
    classics = []
    for f in POETRY_FILES["classics"]:
        data = poetry(f)
        for chapter in data if isinstance(data, list) else [data]:
            classics += [p.strip() for p in chapter.get("paragraphs", []) if len(p) >= 10]

    corpora = {
        "modern_hans": prose,
        "modern_hant": [to_trad.convert(d) for d in prose],
        "classical_hans": classics,
        "classical_hant": tang,
    }

    words = [l.split()[0] for l in open(dict_path, encoding="utf-8") if l.strip()]
    words = [w for w in words if 2 <= len(w) <= 4 and all("一" <= c <= "鿿" for c in w)]

    for name, docs in corpora.items():
        with open(os.path.join(out_dir, name + ".txt"), "w", encoding="utf-8") as f:
            f.write("\n".join(docs) + "\n")
        text = "\n".join(docs)
        rnd = random.Random(42)
        vocab = words if name.endswith("hans") else sorted({to_trad.convert(w) for w in words})
        present = [w for w in vocab if w in text]
        word_queries = rnd.sample(present, min(300, len(present)))
        runs = [r for r in HAN_RUN.findall(text) if len(r) >= 6]
        fragments = []
        while len(fragments) < 300:
            r = rnd.choice(runs)
            n = rnd.randint(2, 6)
            i = rnd.randint(0, len(r) - n)
            fragments.append(r[i:i + n])
        chars = rnd.sample(sorted({c for c in text if HAN_RUN.match(c)}), 100)
        with open(os.path.join(out_dir, name + ".queries.tsv"), "w", encoding="utf-8") as f:
            for kind, qs in (("char", chars), ("word", word_queries), ("fragment", fragments)):
                for q in qs:
                    f.write(f"{kind}\t{q}\n")
        print(f"{name}: {len(docs)} documents, {len(text)} characters")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
