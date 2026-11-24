//! Finding each occurrence in a page and cutting the passage around it.

use mulu_text::{Search, index_terms, is_unspaced, single_character_terms};
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

/// Byte ranges of every occurrence of `search` in `text`, in order.
pub fn occurrences(text: &str, search: &Search) -> Vec<(usize, usize)> {
    match search {
        Search::Nothing => Vec::new(),
        Search::SingleCharacter(c) => single_character_terms(text)
            .into_iter()
            .filter(|t| &t.text == c)
            .map(|t| (t.start, t.end))
            .collect(),
        Search::Prefix(prefix) => index_terms(text)
            .into_iter()
            .filter(|t| t.text.starts_with(prefix.as_str()))
            .map(|t| (t.start, t.end))
            .collect(),
        Search::Phrase(phrase) => located(&index_terms(text), phrase)
            .into_iter()
            .map(|(_, start, end)| (start, end))
            .collect(),
    }
}

/// Word positions and byte ranges of every occurrence of a word or phrase.
fn located(terms: &[mulu_text::Term], phrase: &[(u32, String)]) -> Vec<(u32, usize, usize)> {
    let by_position: HashMap<u32, usize> = terms
        .iter()
        .enumerate()
        .map(|(i, t)| (t.position, i))
        .collect();
    let Some((_, first)) = phrase.first() else {
        return Vec::new();
    };
    let last_offset = phrase.last().map_or(0, |(offset, _)| *offset);
    terms
        .iter()
        .filter(|anchor| &anchor.text == first)
        .filter_map(|anchor| {
            let all_present = phrase.iter().all(|(offset, text)| {
                by_position
                    .get(&(anchor.position + offset))
                    .is_some_and(|&i| &terms[i].text == text)
            });
            let end = by_position.get(&(anchor.position + last_offset))?;
            all_present.then(|| (anchor.position, anchor.start, terms[*end].end))
        })
        .collect()
}

/// Byte ranges of the occurrences of terms near each other (`"a b"~N`): an occurrence
/// counts when every other term has one within `within` words of it, the same rule
/// the index uses to count them.
pub fn near_occurrences(text: &str, near: &[Search], within: u32) -> Vec<(usize, usize)> {
    let terms = index_terms(text);
    let found: Vec<Vec<(u32, usize, usize)>> = near
        .iter()
        .map(|s| match s {
            Search::Phrase(phrase) => located(&terms, phrase),
            _ => Vec::new(),
        })
        .collect();
    let mut out = Vec::new();
    for (i, list) in found.iter().enumerate() {
        for &(p, start, end) in list {
            let close = found
                .iter()
                .enumerate()
                .all(|(j, other)| j == i || other.iter().any(|&(q, _, _)| q.abs_diff(p) <= within));
            if close {
                out.push((start, end));
            }
        }
    }
    out.sort_unstable();
    out
}

/// A passage around one occurrence, ready to display with the match highlighted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Passage {
    /// Text before the match; starts with "…" when the page continues before it.
    pub before: String,
    /// The matched text, exactly as written in the page.
    pub matched: String,
    /// Text after the match; ends with "…" when the page continues after it.
    pub after: String,
}

/// How much one segment of text counts towards the passage length. A word counts as
/// two units; in scripts written without spaces each character counts as one, so that
/// 25 words of context become about 50 characters there (project rule).
fn units(segment: &str) -> usize {
    if segment.chars().any(is_unspaced) {
        segment.graphemes(true).count()
    } else if segment.chars().any(char::is_alphanumeric) {
        2
    } else {
        0
    }
}

/// Cuts a passage with about `words` words before and after the byte range.
pub fn passage(text: &str, start: usize, end: usize, words: usize) -> Passage {
    let budget = words * 2;

    let mut begin = start;
    let mut spent = 0;
    for (i, segment) in text[..start].split_word_bound_indices().rev() {
        let cost = units(segment);
        if spent + cost > budget && cost > 0 {
            break;
        }
        spent += cost;
        begin = i;
    }

    let mut finish = end;
    let mut spent = 0;
    for (i, segment) in text[end..].split_word_bound_indices() {
        let cost = units(segment);
        if spent + cost > budget && cost > 0 {
            break;
        }
        spent += cost;
        finish = end + i + segment.len();
    }

    let tidy = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut before = tidy(&text[begin..start]);
    if !before.is_empty() && text[begin..start].ends_with(char::is_whitespace) {
        before.push(' ');
    }
    if text[..begin].chars().any(|c| !c.is_whitespace()) {
        before.insert_str(0, "… ");
    }
    let mut after = tidy(&text[end..finish]);
    if !after.is_empty() && text[end..finish].starts_with(char::is_whitespace) {
        after.insert(0, ' ');
    }
    if text[finish..].chars().any(|c| !c.is_whitespace()) {
        after.push_str(" …");
    }
    Passage {
        before,
        matched: tidy(&text[start..end]),
        after,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_words_and_phrases_regardless_of_accents() {
        let text = "A ação do romance. Ação e reação.";
        assert_eq!(occurrences(text, &mulu_text::search("acao")).len(), 2);
        let found = occurrences(text, &mulu_text::search("do romance"));
        let start = text.find("do romance").unwrap();
        assert_eq!(found, vec![(start, start + "do romance".len())]);
        assert_eq!(&text[found[0].0..found[0].1], "do romance");
    }

    #[test]
    fn finds_chinese_substrings() {
        let text = "武汉市长江大桥。长江很长。";
        assert_eq!(occurrences(text, &mulu_text::search("长江")).len(), 2);
        assert_eq!(occurrences(text, &mulu_text::search("长")).len(), 3);
    }

    #[test]
    fn passage_has_25_words_each_side() {
        let words: Vec<String> = (1..=80).map(|i| format!("w{i}")).collect();
        let text = words.join(" ");
        let start = text.find("w40").unwrap();
        let p = passage(&text, start, start + 3, 25);
        assert_eq!(p.matched, "w40");
        assert!(p.before.starts_with("… w15 "), "{}", p.before);
        assert!(p.before.ends_with("w39 "));
        assert!(p.after.starts_with(" w41"));
        assert!(p.after.ends_with("w65 …"), "{}", p.after);
    }

    #[test]
    fn passage_at_the_edges_has_no_ellipsis() {
        let text = "Capitu tinha olhos de ressaca.";
        let p = passage(text, 0, 6, 25);
        assert_eq!((p.before.as_str(), p.matched.as_str()), ("", "Capitu"));
        assert_eq!(p.after, " tinha olhos de ressaca.");
    }

    #[test]
    fn chinese_passages_count_characters() {
        let text = "一二三四五六七八九十".repeat(10);
        let start = text.char_indices().nth(50).unwrap().0;
        let end = text.char_indices().nth(51).unwrap().0;
        let p = passage(&text, start, end, 25);
        assert_eq!(p.before.trim_start_matches("… ").chars().count(), 50);
        assert_eq!(p.after.trim_end_matches(" …").chars().count(), 49);
    }
}
