//! D9 experiment: how should Mulu split Chinese text into index terms?
//!
//! Compares four strategies on the same corpora and queries:
//!   unigram       one term per Han character; multi-character queries are phrase queries
//!   bigram        overlapping character pairs, plus a single-character field without
//!                 positions for one-character queries; multi-character queries are phrase queries
//!   jieba         dictionary segmentation (jieba-rs, precise mode); phrase queries
//!   jieba-search  jieba "search" mode for indexing, precise mode for queries, all terms required
//!
//! Ground truth for every query is plain substring containment ("Ctrl+F"):
//! a document is relevant if its text contains the query string.

use jieba_rs::Jieba;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use tantivy::collector::DocSetCollector;
use tantivy::query::{BooleanQuery, Occur, PhraseQuery, Query, TermQuery};
use tantivy::schema::*;
use tantivy::tokenizer::{Token, TokenStream, Tokenizer};
use tantivy::{doc, Index, TantivyDocument};

fn is_han(c: char) -> bool {
    matches!(c as u32,
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0x20000..=0x2FA1F)
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Strategy { Unigram, Bigram, Jieba, JiebaSearch }

/// Splits text into runs: (is_han, byte_start, &str). Non-Han runs are alphanumeric words.
fn runs(text: &str) -> Vec<(bool, usize, &str)> {
    let mut out = Vec::new();
    let mut cur: Option<(bool, usize)> = None;
    for (i, c) in text.char_indices() {
        let kind = if is_han(c) { Some(true) } else if c.is_alphanumeric() { Some(false) } else { None };
        match (cur, kind) {
            (Some((k, _)), Some(k2)) if k == k2 => {}
            (Some((k, s)), _) => { out.push((k, s, &text[s..i])); cur = kind.map(|k| (k, i)); }
            (None, Some(k)) => cur = Some((k, i)),
            (None, None) => {}
        }
    }
    if let Some((k, s)) = cur { out.push((k, s, &text[s..])); }
    out
}

fn analyze(text: &str, st: Strategy, jieba: &Jieba, query_mode: bool) -> Vec<Token> {
    let mut toks = Vec::new();
    let mut pos = 0usize;
    let push = |toks: &mut Vec<Token>, s: &str, from: usize, to: usize, pos: &mut usize| {
        toks.push(Token { offset_from: from, offset_to: to, position: *pos, text: s.to_lowercase(), position_length: 1 });
        *pos += 1;
    };
    for (han, start, run) in runs(text) {
        if !han { push(&mut toks, run, start, start + run.len(), &mut pos); continue; }
        match st {
            Strategy::Unigram => {
                for (i, c) in run.char_indices() {
                    push(&mut toks, &run[i..i + c.len_utf8()], start + i, start + i + c.len_utf8(), &mut pos);
                }
            }
            Strategy::Bigram => {
                let idx: Vec<usize> = run.char_indices().map(|(i, _)| i).chain(std::iter::once(run.len())).collect();
                let n = idx.len() - 1;
                if n == 1 { push(&mut toks, run, start, start + run.len(), &mut pos); }
                for k in 0..n.saturating_sub(1) {
                    push(&mut toks, &run[idx[k]..idx[k + 2]], start + idx[k], start + idx[k + 2], &mut pos);
                }
            }
            Strategy::Jieba => {
                for t in jieba.cut(run, true) {
                    push(&mut toks, t.word, start + t.byte_start, start + t.byte_end, &mut pos);
                }
            }
            Strategy::JiebaSearch => {
                let words = if query_mode { jieba.cut(run, true) } else { jieba.cut_for_search(run, true) };
                for t in words { push(&mut toks, t.word, start + t.byte_start, start + t.byte_end, &mut pos); }
            }
        }
        // Leave a gap between runs so that phrase queries never match across punctuation.
        pos += 1;
    }
    toks
}

#[derive(Clone)]
struct MuluTokenizer { st: Strategy, jieba: Arc<Jieba> }
struct VecStream { toks: Vec<Token>, i: usize }
impl TokenStream for VecStream {
    fn advance(&mut self) -> bool { if self.i < self.toks.len() { self.i += 1; true } else { false } }
    fn token(&self) -> &Token { &self.toks[self.i - 1] }
    fn token_mut(&mut self) -> &mut Token { &mut self.toks[self.i - 1] }
}
impl Tokenizer for MuluTokenizer {
    type TokenStream<'a> = VecStream;
    fn token_stream<'a>(&'a mut self, text: &'a str) -> VecStream {
        VecStream { toks: analyze(text, self.st, &self.jieba, false), i: 0 }
    }
}

struct Built { index: Index, main: Field, uni: Option<Field>, id: Field, secs: f64, bytes: u64 }

fn dir_size(p: &std::path::Path) -> u64 {
    std::fs::read_dir(p).unwrap().map(|e| e.unwrap().metadata().unwrap().len()).sum()
}

fn build(docs: &[String], st: Strategy, jieba: &Arc<Jieba>, dir: &std::path::Path) -> Built {
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).unwrap();
    let mut sb = Schema::builder();
    let opts = |name: &str| TextOptions::default().set_indexing_options(
        TextFieldIndexing::default().set_tokenizer(name).set_index_option(IndexRecordOption::WithFreqsAndPositions));
    let main = sb.add_text_field("text", opts("main"));
    let uni = if st == Strategy::Bigram { Some(sb.add_text_field("uni", TextOptions::default().set_indexing_options(
        TextFieldIndexing::default().set_tokenizer("uni").set_index_option(IndexRecordOption::WithFreqs)))) } else { None };
    let id = sb.add_u64_field("id", STORED | FAST);
    let index = Index::create_in_dir(dir, sb.build()).unwrap();
    index.tokenizers().register("main", MuluTokenizer { st, jieba: jieba.clone() });
    index.tokenizers().register("uni", MuluTokenizer { st: Strategy::Unigram, jieba: jieba.clone() });
    let t0 = Instant::now();
    let mut w = index.writer_with_num_threads(1, 200_000_000).unwrap();
    for (i, d) in docs.iter().enumerate() {
        let mut td = doc!(main => d.as_str(), id => i as u64);
        if let Some(u) = uni { td.add_text(u, d); }
        w.add_document(td).unwrap();
    }
    w.commit().unwrap();
    w.wait_merging_threads().unwrap();
    let secs = t0.elapsed().as_secs_f64();
    let index = Index::open_in_dir(dir).unwrap();
    index.tokenizers().register("main", MuluTokenizer { st, jieba: jieba.clone() });
    index.tokenizers().register("uni", MuluTokenizer { st: Strategy::Unigram, jieba: jieba.clone() });
    Built { bytes: dir_size(dir), index, main, uni, id, secs }
}

fn make_query(b: &Built, st: Strategy, q: &str, jieba: &Jieba) -> Box<dyn Query> {
    let (field, toks) = if st == Strategy::Bigram && q.chars().count() == 1 {
        (b.uni.unwrap(), analyze(q, Strategy::Unigram, jieba, true))
    } else {
        (b.main, analyze(q, st, jieba, true))
    };
    let terms: Vec<Term> = toks.iter().map(|t| Term::from_field_text(field, &t.text)).collect();
    if terms.len() == 1 {
        return Box::new(TermQuery::new(terms[0].clone(), IndexRecordOption::WithFreqs));
    }
    if st == Strategy::JiebaSearch {
        let subs = terms.into_iter().map(|t| (Occur::Must, Box::new(TermQuery::new(t, IndexRecordOption::Basic)) as Box<dyn Query>)).collect();
        return Box::new(BooleanQuery::new(subs));
    }
    Box::new(PhraseQuery::new_with_offset(toks.iter().zip(terms).map(|(t, term)| (t.position, term)).collect()))
}

fn search_ids(b: &Built, query: &dyn Query) -> HashSet<u64> {
    let searcher = b.index.reader().unwrap().searcher();
    let set = searcher.search(query, &DocSetCollector).unwrap();
    set.into_iter().map(|a| {
        let d: TantivyDocument = searcher.doc(a).unwrap();
        d.get_first(b.id).unwrap().as_u64().unwrap()
    }).collect()
}

const STRATS: [(Strategy, &str); 4] = [
    (Strategy::Unigram, "unigram"), (Strategy::Bigram, "bigram"),
    (Strategy::Jieba, "jieba"), (Strategy::JiebaSearch, "jieba-search")];

fn main() {
    let corpus_dir = std::env::args().nth(1).expect("corpus dir");
    let mode = std::env::args().nth(2).unwrap_or_else(|| "quality".into());
    let t0 = Instant::now();
    let jieba = Arc::new(Jieba::new());
    eprintln!("jieba dictionary load: {:.2} s", t0.elapsed().as_secs_f64());
    let corpora = ["modern_hans", "modern_hant", "classical_hans", "classical_hant"];
    let tmp = std::path::PathBuf::from("/tmp/cjk-idx");

    if mode == "quality" {
        println!("corpus\tstrategy\tquery_type\tqueries\trelevant\tfound\tfalse_hits\trecall\tprecision\tqueries_fully_answered");
        for c in corpora {
            let docs: Vec<String> = std::fs::read_to_string(format!("{corpus_dir}/{c}.txt")).unwrap()
                .lines().map(String::from).collect();
            let queries: Vec<(String, String)> = std::fs::read_to_string(format!("{corpus_dir}/{c}.queries.tsv")).unwrap()
                .lines().map(|l| { let (a, b) = l.split_once('\t').unwrap(); (a.into(), b.into()) }).collect();
            for (st, name) in STRATS {
                let b = build(&docs, st, &jieba, &tmp.join(name));
                for qt in ["char", "word", "fragment"] {
                    let (mut nq, mut rel, mut found, mut fp, mut full) = (0, 0, 0, 0, 0);
                    for (_, q) in queries.iter().filter(|(t, _)| t == qt) {
                        let truth: HashSet<u64> = docs.iter().enumerate().filter(|(_, d)| d.contains(q.as_str())).map(|(i, _)| i as u64).collect();
                        let got = search_ids(&b, make_query(&b, st, q, &jieba).as_ref());
                        let tp = got.intersection(&truth).count();
                        nq += 1; rel += truth.len(); found += tp; fp += got.len() - tp;
                        if tp == truth.len() && got.len() == tp { full += 1; }
                    }
                    println!("{c}\t{name}\t{qt}\t{nq}\t{rel}\t{found}\t{fp}\t{:.4}\t{:.4}\t{:.4}",
                        found as f64 / rel as f64, found as f64 / (found + fp).max(1) as f64, full as f64 / nq as f64);
                }
            }
        }
    } else {
        // performance: all corpora concatenated and repeated `reps` times
        let reps: usize = std::env::args().nth(3).map(|s| s.parse().unwrap()).unwrap_or(20);
        let mut base = Vec::new();
        for c in corpora { base.extend(std::fs::read_to_string(format!("{corpus_dir}/{c}.txt")).unwrap().lines().map(String::from)); }
        let docs: Vec<String> = (0..reps).flat_map(|_| base.iter().cloned()).collect();
        let text_bytes: usize = docs.iter().map(|d| d.len()).sum();
        let queries: Vec<String> = std::fs::read_to_string(format!("{corpus_dir}/modern_hans.queries.tsv")).unwrap()
            .lines().map(|l| l.split_once('\t').unwrap().1.to_string()).collect();
        println!("strategy\tdocs\ttext_MB\tindex_s\tindex_MB\tindex_to_text\tquery_median_ms\tquery_p95_ms\tquery_max_ms");
        for (st, name) in STRATS {
            let b = build(&docs, st, &jieba, &tmp.join(name));
            let searcher = b.index.reader().unwrap().searcher();
            let mut times = Vec::new();
            for _ in 0..3 { for q in &queries {
                let query = make_query(&b, st, q, &jieba);
                let t = Instant::now();
                let _n = searcher.search(query.as_ref(), &tantivy::collector::Count).unwrap();
                times.push(t.elapsed().as_secs_f64() * 1000.0);
            } }
            times.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let p = |f: f64| times[((times.len() as f64 - 1.0) * f) as usize];
            println!("{name}\t{}\t{:.1}\t{:.2}\t{:.1}\t{:.2}\t{:.3}\t{:.3}\t{:.3}", docs.len(), text_bytes as f64 / 1e6, b.secs,
                b.bytes as f64 / 1e6, b.bytes as f64 / text_bytes as f64, p(0.5), p(0.95), times[times.len() - 1]);
        }
    }
}
