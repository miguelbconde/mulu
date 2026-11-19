# How should Mulu split Chinese text for search? (decision D9 → D13)

*Experiment run on 2026-11-19. Result: Mulu indexes Chinese as overlapping character pairs (bigrams), with a separate single-character field. See [DECISIONS.md](../../DECISIONS.md), D13.*

## The question

Chinese is written without spaces between words, so a search index has to decide what its "words" are. The choice is made once: changing it later means every user rebuilds their index. Two families of approaches are in use today:

- **Dictionary segmentation.** A dictionary and a statistical model guess where words begin and end. Jieba is the most widely used; Lindera offers the same Jieba dictionary or CC-CEDICT.
- **Character n-grams.** Every character, or every pair of neighboring characters, becomes an index term. No dictionary is involved.

Mulu's main use is a researcher gathering everything a library holds about a term, so the question is mainly about **recall**: when someone searches for a string of characters, are all the documents that contain it found?

## Strategies compared

All four use [Tantivy](https://github.com/quickwit-oss/tantivy) 0.27, the search engine Mulu uses.

| Strategy | Index terms | Query |
|---|---|---|
| `unigram` | each Han character | phrase of characters |
| `bigram` | each pair of neighboring Han characters, plus a single-character field without positions | phrase of pairs; one-character queries use the single-character field |
| `jieba` | [jieba-rs](https://github.com/messense/jieba-rs) 0.11, precise mode | phrase of words, segmented the same way |
| `jieba-search` | jieba-rs search mode (overlapping words) | all words of the precise segmentation required |

In every strategy, non-Han text is split into ordinary words, and a position gap is left at punctuation so that a phrase never matches across a sentence break.

## Corpora and queries

[`prepare_corpus.py`](prepare_corpus.py) downloads and prepares the texts. They are not redistributed here.

| Corpus | Source | Documents | Characters |
|---|---|---|---|
| `modern_hans` | Qian Zhongshu, *Fortress Besieged* (围城), the jieba-rs benchmark text, in passages of about 150 characters | 1,532 | 246,946 |
| `modern_hant` | the same text converted to Traditional Chinese with OpenCC (`s2twp`) | 1,532 | 246,956 |
| `classical_hans` | *Analects*, *Mencius*, *Great Learning* ([chinese-poetry](https://github.com/chinese-poetry/chinese-poetry)) | 1,175 | 69,387 |
| `classical_hant` | Tang poems ([chinese-poetry](https://github.com/chinese-poetry/chinese-poetry)) | 1,999 | 120,142 |

Each corpus has 700 queries, sampled with a fixed seed:

- **char:** 100 single characters found in the corpus;
- **word:** 300 words from the Jieba dictionary that appear in the corpus (converted to Traditional characters for the Traditional corpora);
- **fragment:** 300 random strings of 2 to 6 characters taken from the text, which may cross word boundaries, like a half-remembered phrase.

**Ground truth** is plain substring containment, as with Ctrl+F: a document is relevant if its text contains the query.

## Results

### Did the search find the right documents?

Totals over the four corpora. *Recall* is the share of relevant documents found; *precision* is the share of documents found that are relevant. Per-corpus figures are in [`results/quality.tsv`](results/quality.tsv).

| Strategy | Single characters (recall / precision) | Dictionary words | Fragments |
|---|---|---|---|
| `unigram` | 100% / 100% | 100% / 100% | 100% / 100% |
| **`bigram`** | **100% / 100%** | **100% / 100%** | **100% / 100%** |
| `jieba` | 24.4% / 100% | 75.9% / 100% | 58.3% / 100% |
| `jieba-search` | 24.4% / 100% | 81.6% / 78.6% | 65.8% / 39.8% |

Dictionary segmentation loses about a quarter of the documents even for real dictionary words, and three quarters for single characters. The loss is worst on Tang poetry, where the dictionary fits least (63.5% recall for words). Search mode recovers a little recall at a large cost in precision.

### Speed and size

The four corpora repeated 40 times: 249,520 documents, 80.8 MB of text. One indexing thread on a cloud virtual machine (Intel Xeon, 2.3 GHz, 2 cores). Query times are over the 700 `modern_hans` queries, run 3 times each. Raw figures are in [`results/perf.tsv`](results/perf.tsv).

| Strategy | Indexing time | Index size | Index ÷ text | Median query | 95th percentile | Slowest query |
|---|---|---|---|---|---|---|
| `unigram` | 5.6 s | 48.8 MB | 0.60 | 0.090 ms | 1.10 ms | 7.4 ms |
| **`bigram`** | **13.9 s** | **87.9 MB** | **1.09** | **0.001 ms** | **0.035 ms** | **0.57 ms** |
| `jieba` | 8.6 s | 38.4 MB | 0.47 | 0.001 ms | 0.140 ms | 2.5 ms |
| `jieba-search` | 8.4 s | 40.2 MB | 0.50 | 0.001 ms | 0.050 ms | 0.35 ms |

`unigram` finds everything but is the slowest to query: a phrase made of common characters has to walk very long lists. Its slowest query is 13 times slower than `bigram`'s, and that gap widens as a library grows.

## Decision

**Bigrams with a single-character field.** It is the only strategy that is both exact and fast:

- it finds every document that contains the query, and only those, in Simplified, Traditional, modern and classical Chinese alike;
- it needs no dictionary, so there is no dictionary to ship (jieba-rs embeds about 5 MB) and no dictionary update that would force users to rebuild their index;
- it treats all Han text the same way, which also serves Japanese kanji and Korean hanja.

The costs: the index is about twice the size of a Jieba index, and indexing is slower. A search for 市长 ("mayor") also finds 武汉市长江大桥 ("Wuhan Yangtze River Bridge"), because the two characters appear there side by side. That is the expected behavior of an exact-text search, and the same thing Ctrl+F does.

## Limits of this experiment

- The corpus is small and was repeated to measure speed. Real libraries are larger and more varied; speed will be measured again on large libraries before each release.
- Only whether a document is found was measured, not the order of the results.
- The Traditional modern corpus is a machine conversion of a Simplified text.
- Japanese kana and Korean hangul were not tested. They will be handled when those languages are measured (version 0.8).

## Reproducing

```sh
pip install opencc==1.4.2
cargo build --release
python3 prepare_corpus.py corpus ~/.cargo/registry/src/*/jieba-rs-0.11.0/src/data/dict.txt
./target/release/cjk-tokenization corpus quality > quality.tsv
./target/release/cjk-tokenization corpus perf 40 > perf.tsv
```

`cargo build` downloads jieba-rs 0.11.0 first, which provides the dictionary file used to sample word queries.
