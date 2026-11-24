//! Finding files by their name (decision D21), with the same word rules and signs as
//! the search in the text (decision D33): "machado casm*" finds
//! "ASSIS, Machado (1899) - Dom Casmurro.pdf".

use mulu_text::{Query, Search, index_terms, single_character_terms};

/// Whether the file name matches the search. Parts near each other (`"a b"~N`) only
/// need to be in the name; `name:` changes nothing here, since this is the name.
pub fn matches(name: &str, query: &Query) -> bool {
    // Without the extension: by the Unicode word rules "Casmurro.pdf" is a single word.
    let name = std::path::Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(name);
    let terms = index_terms(name);
    // Only what is absent ("-quincas") finds nothing, as in the text.
    asks_for_something(query) && eval(name, &terms, query)
}

fn asks_for_something(query: &Query) -> bool {
    match query {
        Query::Term(_) | Query::Near(..) | Query::Name(_) => true,
        Query::All(parts) | Query::Any(parts) => parts.iter().any(asks_for_something),
        Query::Not(_) => false,
    }
}

fn eval(name: &str, terms: &[mulu_text::Term], query: &Query) -> bool {
    match query {
        Query::Term(part) => has(name, terms, part),
        Query::Near(parts, _) => parts.iter().all(|p| has(name, terms, p)),
        Query::All(parts) => parts.iter().all(|p| eval(name, terms, p)),
        Query::Any(parts) => parts
            .iter()
            .filter(|p| !matches!(p, Query::Not(_)))
            .any(|p| eval(name, terms, p)),
        Query::Not(inner) => !eval(name, terms, inner),
        Query::Name(inner) => eval(name, terms, inner),
    }
}

fn has(name: &str, terms: &[mulu_text::Term], part: &Search) -> bool {
    match part {
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
    }
}

#[cfg(test)]
mod tests {
    use super::matches;
    use mulu_text::parse;

    fn m(name: &str, query: &str) -> bool {
        matches(name, &parse(query))
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

    #[test]
    fn signs() {
        let name = "ASSIS, Machado (1899) - Dom Casmurro.pdf";
        assert!(m(name, "quincas | casmurro"));
        assert!(m(name, "machado -quincas"));
        assert!(!m(name, "machado -casmurro"));
        assert!(!m(name, "-quincas"));
        assert!(m(name, "name:assis"));
        assert!(m(name, "\"assis casmurro\"~3"));
    }
}
