//! Mulu decision D24: comparing PDF text extraction engines.
//!
//! `extract ENGINE FILE.pdf` prints the text of each page as JSON, `{"pages": [...]}`.
//! `score TRUTH.json OUTPUT.json` compares an extraction with the known text, with the
//! same word analysis as Mulu's search (`mulu-text`), and prints the counts as JSON.
//!
//! The driver (`run.py`) runs every extraction in its own process, with a time limit,
//! and measures time and memory from outside.

mod hayro_text;

use mulu_text::index_terms;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("extract") if args.len() == 4 => match extract(&args[2], &args[3]) {
            Ok(pages) => {
                println!("{}", json!({ "pages": pages }));
                report_peak_memory();
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(2)
            }
        },
        Some("score") if args.len() == 4 => {
            let truth = read_pages(&args[2]);
            let output = read_pages(&args[3]);
            println!("{}", score(&truth, &output));
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("usage: pdf-extraction extract ENGINE FILE | score TRUTH OUTPUT");
            ExitCode::from(1)
        }
    }
}

fn extract(engine: &str, path: &str) -> Result<Vec<String>, String> {
    match engine {
        "pdf_oxide" => {
            let document = pdf_oxide::PdfDocument::open(path).map_err(|e| e.to_string())?;
            let count = document.page_count().map_err(|e| e.to_string())?;
            (0..count)
                .map(|i| document.extract_text(i).map_err(|e| e.to_string()))
                .collect()
        }
        "pdf-extract" => pdf_extract::extract_text_by_pages(path).map_err(|e| e.to_string()),
        "hayro" => hayro_text::extract(path),
        // PDFium through pdfium-render, the way Mulu would use it (re-measured on
        // 2026-11-20 after the first test on real books, see the README).
        "pdfium-rs" => {
            let library = std::env::var("PDFIUM_LIBRARY").map_err(|_| "set PDFIUM_LIBRARY".to_owned())?;
            let bindings = pdfium_render::prelude::Pdfium::bind_to_library(library).map_err(|e| e.to_string())?;
            let pdfium = pdfium_render::prelude::Pdfium::new(bindings);
            let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
            let document = pdfium.load_pdf_from_byte_slice(&bytes, None).map_err(|e| e.to_string())?;
            let mut pages = Vec::new();
            for page in document.pages().iter() {
                // As Mulu does (crates/mulu-extract/src/pdf.rs): CR LF to LF, and the
                // mark of a word hyphenated at a line end removed.
                let text = page.text().map(|t| t.all()).unwrap_or_default();
                pages.push(text.replace("\r\n", "\n").replace(['\u{2}', '\u{FFFE}'], ""));
            }
            Ok(pages)
        }
        _ => Err(format!("unknown engine {engine}")),
    }
}

/// Prints the peak memory of this process (Linux: VmHWM, which starts afresh when the
/// program starts, unlike the counter seen by the parent process).
fn report_peak_memory() {
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        if let Some(line) = status.lines().find(|l| l.starts_with("VmHWM:")) {
            let kb = line.split_whitespace().nth(1).unwrap_or("0");
            eprintln!("MAXRSS_KB={kb}");
        }
    }
}

fn read_pages(path: &str) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let value: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    value["pages"]
        .as_array()
        .map(|pages| pages.iter().map(|p| p.as_str().unwrap_or("").to_owned()).collect())
        .unwrap_or_default()
}

/// The clean-up any PDF reader in Mulu would do, applied equally to every engine:
/// words split by a hyphen at the end of a line are joined, and a line break between
/// two characters of a script without spaces (Chinese, Japanese, Thai) is removed.
pub fn tidy(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if (c == '-' || c == '\u{00AD}') && i > 0 && chars[i - 1].is_alphabetic() {
            // Hyphen, optional spaces, a line break, optional spaces, a lowercase letter.
            let mut j = i + 1;
            while j < chars.len() && (chars[j] == ' ' || chars[j] == '\r') {
                j += 1;
            }
            if j < chars.len() && chars[j] == '\n' {
                let mut k = j + 1;
                while k < chars.len() && chars[k] == ' ' {
                    k += 1;
                }
                if k < chars.len() && chars[k].is_lowercase() {
                    i = k;
                    continue;
                }
            }
        }
        // A spacing accent right before a letter, as old TeX fonts (OT1 encoding) draw
        // them: "Na¸c˜oes" becomes "Nações".
        if let Some(combining) = combining_accent(c)
            && let Some(&next) = chars.get(i + 1)
            && next.is_alphabetic()
        {
            let composed: String = [next, combining].iter().collect::<String>();
            out.extend(unicode_normalization::UnicodeNormalization::nfc(composed.chars()));
            i += 2;
            continue;
        }
        if c == '\n' || c == '\r' {
            let previous = out.chars().last();
            let mut k = i + 1;
            while k < chars.len() && matches!(chars[k], '\n' | '\r' | ' ') {
                k += 1;
            }
            let next = chars.get(k).copied();
            if previous.is_some_and(mulu_text::is_unspaced) && next.is_some_and(mulu_text::is_unspaced) {
                i = k;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

fn combining_accent(c: char) -> Option<char> {
    Some(match c {
        '\u{00B4}' => '\u{0301}', // acute
        '\u{02DC}' => '\u{0303}', // tilde
        '\u{02C6}' => '\u{0302}', // circumflex
        '\u{00A8}' => '\u{0308}', // diaeresis
        '\u{00B8}' => '\u{0327}', // cedilla
        '\u{02C7}' => '\u{030C}', // caron
        '\u{02D8}' => '\u{0306}', // breve
        '\u{02DA}' => '\u{030A}', // ring
        '\u{02D9}' => '\u{0307}', // dot above
        '\u{00AF}' => '\u{0304}', // macron
        '\u{02DD}' => '\u{030B}', // double acute
        _ => return None,
    })
}

fn terms(text: &str) -> Vec<String> {
    index_terms(text).into_iter().map(|t| t.text).collect()
}

fn multiset(terms: &[String]) -> HashMap<&str, usize> {
    let mut counts = HashMap::new();
    for t in terms {
        *counts.entry(t.as_str()).or_insert(0) += 1;
    }
    counts
}

fn overlap(a: &HashMap<&str, usize>, b: &HashMap<&str, usize>) -> usize {
    a.iter().map(|(k, n)| (*n).min(*b.get(k).unwrap_or(&0))).sum()
}

fn windows(terms: &[String], n: usize) -> Vec<String> {
    terms.windows(n).map(|w| w.join("\u{1}")).collect()
}

/// Counts for one way of reading the output (raw, or after `tidy`).
fn counts(truth: &[String], output: &[String]) -> Value {
    let mut truth_terms = 0;
    let mut output_terms = 0;
    let mut found = 0;
    let mut phrases = 0;
    let mut phrases_found = 0;
    let mut all_truth = Vec::new();
    let mut all_output = Vec::new();
    for (i, page) in truth.iter().enumerate() {
        let t = terms(page);
        let o = terms(output.get(i).map(String::as_str).unwrap_or(""));
        truth_terms += t.len();
        output_terms += o.len();
        found += overlap(&multiset(&t), &multiset(&o));
        let out_windows: HashSet<String> = windows(&o, 3).into_iter().collect();
        for w in windows(&t, 3) {
            phrases += 1;
            if out_windows.contains(&w) {
                phrases_found += 1;
            }
        }
        all_truth.extend(t);
        all_output.extend(o);
    }
    for page in output.iter().skip(truth.len()) {
        let o = terms(page);
        output_terms += o.len();
        all_output.extend(o);
    }
    let found_anywhere = overlap(&multiset(&all_truth), &multiset(&all_output));
    json!({
        "truth_terms": truth_terms,
        "output_terms": output_terms,
        "found_on_right_page": found,
        "found_anywhere": found_anywhere,
        "phrases": phrases,
        "phrases_found": phrases_found,
    })
}

fn score(truth: &[String], output: &[String]) -> Value {
    let tidied: Vec<String> = output.iter().map(|p| tidy(p)).collect();
    json!({
        "pages_truth": truth.len(),
        "pages_output": output.len(),
        "raw": counts(truth, output),
        "tidy": counts(truth, &tidied),
    })
}

#[cfg(test)]
mod tests {
    use super::tidy;

    #[test]
    fn tidy_joins_hyphenated_words_and_unspaced_lines() {
        assert_eq!(tidy("a liber-\ndade e a Ex-\nPresidente"), "a liberdade e a Ex-\nPresidente");
        assert_eq!(tidy("人人生而\n自由"), "人人生而自由");
        assert_eq!(tidy("fim\nde linha"), "fim\nde linha");
        assert_eq!(tidy("Na\u{b8}c\u{2dc}oes, f\u{b4}e"), "Nações, fé");
    }
}
