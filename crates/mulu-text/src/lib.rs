//! # mulu-text
//!
//! Turns text, in any script, into the terms that Mulu stores in its search index,
//! and turns a search into the same kind of terms, so that both always agree.
//!
//! - Scripts that separate words with spaces (Latin, Greek, Cyrillic, Arabic, Hebrew,
//!   Devanagari, ...) are split into words by the Unicode word boundary rules
//!   (UAX #29). Apostrophes also split words, so "homme" is found in "l'homme".
//! - Scripts written without spaces (Chinese, Japanese, Korean, Thai, Lao, Khmer,
//!   Myanmar) are indexed as overlapping pairs of characters, plus a separate list of
//!   single characters. A search then finds exactly the documents that contain the
//!   characters typed, as Ctrl+F would (decisions D13 and D15).
//! - Every term is folded (see [`fold`]): case, compatibility forms and optional
//!   accents are ignored.
//!
//! Positions leave a gap after each run of an unspaced script, so that a search can
//! never match across a punctuation mark between two such runs.

mod normalize;
mod script;

pub use normalize::fold;
pub use script::is_unspaced;

use unicode_segmentation::UnicodeSegmentation;

/// One search term, with where it came from in the original text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    /// The folded text that goes into the index.
    pub text: String,
    /// Position in the sequence of terms, used for exact phrases.
    pub position: u32,
    /// Byte range in the original text, used to highlight the passage.
    pub start: usize,
    pub end: usize,
}

/// What a search should look for, once the typed text has been analyzed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Search {
    /// Nothing searchable was typed (only spaces, punctuation or symbols).
    Nothing,
    /// A single character of an unspaced script: looked up in the single-character list.
    SingleCharacter(String),
    /// A sequence of terms that must appear in this order, at these relative positions.
    Phrase(Vec<(u32, String)>),
    /// Every word that starts with this (folded) text, typed as `melancol*` (decision
    /// D21). Only one word, of at least two letters, in a script written with spaces.
    Prefix(String),
}

struct Builder {
    terms: Vec<Term>,
    position: u32,
    /// Pending characters of an unspaced script: (start, end, folded text).
    run: Vec<(usize, usize, String)>,
}

impl Builder {
    fn push(&mut self, text: String, start: usize, end: usize) {
        if text.is_empty() {
            return;
        }
        self.terms.push(Term {
            text,
            position: self.position,
            start,
            end,
        });
        self.position += 1;
    }

    fn flush_run(&mut self) {
        if self.run.is_empty() {
            return;
        }
        let run = std::mem::take(&mut self.run);
        if run.len() == 1 {
            let (start, end, text) = run.into_iter().next().unwrap_or_default();
            self.push(text, start, end);
        } else {
            for pair in run.windows(2) {
                let (start, _, first) = &pair[0];
                let (_, end, second) = &pair[1];
                self.push(format!("{first}{second}"), *start, *end);
            }
        }
        // Leave a gap so that a phrase never continues across the end of this run.
        self.position += 1;
    }

    fn word(&mut self, word: &str, start: usize) {
        self.flush_run();
        let mut part_start = 0;
        let ends = word
            .char_indices()
            .filter(|(_, c)| matches!(c, '\'' | '’'))
            .map(|(i, c)| (i, i + c.len_utf8()))
            .chain(std::iter::once((word.len(), word.len())));
        for (part_end, next_start) in ends {
            let part = &word[part_start..part_end];
            if part.chars().any(char::is_alphanumeric) {
                self.push(fold(part), start + part_start, start + part_end);
            }
            part_start = next_start;
        }
    }
}

/// Splits `text` into the terms stored in the main index field.
pub fn index_terms(text: &str) -> Vec<Term> {
    let mut b = Builder {
        terms: Vec::new(),
        position: 0,
        run: Vec::new(),
    };
    for (start, segment) in text.split_word_bound_indices() {
        if segment.chars().any(is_unspaced) {
            for (offset, grapheme) in segment.grapheme_indices(true) {
                let s = start + offset;
                if grapheme.chars().next().is_some_and(is_unspaced) {
                    b.run.push((s, s + grapheme.len(), fold(grapheme)));
                } else if grapheme.chars().any(char::is_alphanumeric) {
                    b.word(grapheme, s);
                } else {
                    b.flush_run();
                }
            }
        } else if segment.chars().any(char::is_alphanumeric) {
            b.word(segment, start);
        } else {
            b.flush_run();
        }
    }
    b.flush_run();
    b.terms
}

/// Lists every single character of an unspaced script, for one-character searches.
/// Positions are not needed for this list and are simply consecutive.
pub fn single_character_terms(text: &str) -> Vec<Term> {
    text.grapheme_indices(true)
        .filter(|(_, g)| g.chars().next().is_some_and(is_unspaced))
        .enumerate()
        .map(|(i, (start, g))| Term {
            text: fold(g),
            position: u32::try_from(i).unwrap_or(u32::MAX),
            start,
            end: start + g.len(),
        })
        .collect()
}

/// Analyzes what the user typed, with exactly the same rules as the indexed text.
pub fn search(query: &str) -> Search {
    let trimmed = query.trim();
    let mut graphemes = trimmed.graphemes(true);
    if let (Some(g), None) = (graphemes.next(), graphemes.next())
        && g.chars().next().is_some_and(is_unspaced)
    {
        return Search::SingleCharacter(fold(g));
    }
    if let Some(stem) = trimmed.strip_suffix('*') {
        let terms = index_terms(stem);
        if let [term] = terms.as_slice()
            && term.text.chars().count() >= 2
            && !term.text.chars().any(is_unspaced)
            && term.end == stem.len()
        {
            return Search::Prefix(term.text.clone());
        }
    }
    let terms = index_terms(trimmed);
    let Some(first) = terms.first().map(|t| t.position) else {
        return Search::Nothing;
    };
    Search::Phrase(
        terms
            .into_iter()
            .map(|t| (t.position - first, t.text))
            .collect(),
    )
}
