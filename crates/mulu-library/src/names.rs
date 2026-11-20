//! Finding files by their name (decision D21), with the same word rules as the search
//! in the text: "machado casm*" finds "ASSIS, Machado (1899) - Dom Casmurro.pdf".

use mulu_text::{Search, index_terms, single_character_terms};

/// Whether the file name contains every part of the search.
pub fn matches(name: &str, parts: &[Search]) -> bool {
    // Without the extension: by the Unicode word rules "Casmurro.pdf" is a single word.
    let name = std::path::Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(name);
    let terms = index_terms(name);
    parts.iter().all(|part| match part {
        Search::Nothing => true,
        Search::SingleCharacter(c) => single_character_terms(name).iter().any(|t| &t.text == c),
        Search::Prefix(prefix) => terms.iter().any(|t| t.text.starts_with(prefix.as_str())),
        Search::Phrase(phrase) => terms.iter().any(|anchor| {
            phrase.iter().all(|(offset, text)| {
                terms
                    .iter()
                    .any(|t| t.position == anchor.position + offset && &t.text == text)
            })
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::matches;
    use mulu_text::search;

    fn m(name: &str, query: &str) -> bool {
        let parts: Vec<_> = mulu_index::split_query(query)
            .iter()
            .map(|p| search(p))
            .collect();
        matches(name, &parts)
    }

    #[test]
    fn names() {
        let name = "ASSIS, Machado (1899) - Dom Casmurro.pdf";
        assert!(m(name, "machado casm*"));
        assert!(m(name, "\"dom casmurro\""));
        assert!(m(name, "1899"));
        assert!(!m(name, "\"casmurro dom\""));
        assert!(!m(name, "quincas"));
        assert!(m("鲁迅 - 呐喊.epub", "呐喊"));
        assert!(m("鲁迅 - 呐喊.epub", "迅"));
    }
}
