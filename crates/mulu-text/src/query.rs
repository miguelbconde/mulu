//! Reading what the user typed into a search, with the signs of decision D33:
//!
//! | Sign            | Meaning                                              |
//! |-----------------|------------------------------------------------------|
//! | `a b`           | files that contain both words, anywhere              |
//! | `"a b"`         | that exact phrase; inside quotes everything is text  |
//! | `-a`            | files without this word                              |
//! | `a \| b`        | either word                                          |
//! | `"a b"~10`      | the words within 10 words of each other, any order   |
//! | `melancol*`     | every word that begins this way                      |
//! | `( )`           | grouping: `machado (capitu \| bentinho)`             |
//! | `name:a`        | in the file name only                                |
//!
//! `|` binds tighter than the space, so `machado capitu | bentinho` means
//! `machado (capitu | bentinho)`, as in web search engines. A `-` counts only at the
//! start of a word (after a space, a `(` or the start), so hyphenated words such as
//! "bem-te-vi" stay words. Words in capitals (AND, OR, NOT, E, OU, NÃO) are ordinary
//! words. Nothing typed is ever an error: an unclosed quote or bracket runs to the end,
//! and a stray `)` or `|` is ignored.

use crate::{Search, search, search_text};

/// A search, read into a tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
    /// A word, an exact phrase, a word beginning or a single character.
    Term(Search),
    /// Several terms, each within `within` words of the others, in any order.
    Near(Vec<Search>, u32),
    /// Files that contain every part.
    All(Vec<Query>),
    /// Files that contain at least one part.
    Any(Vec<Query>),
    /// Files that do not contain this.
    Not(Box<Query>),
    /// This, looked for in the file name only.
    Name(Box<Query>),
}

impl Query {
    /// Whether nothing searchable was typed.
    pub fn is_empty(&self) -> bool {
        match self {
            Query::Term(s) => *s == Search::Nothing,
            Query::Near(terms, _) => terms.is_empty(),
            Query::All(parts) | Query::Any(parts) => parts.is_empty(),
            Query::Not(q) | Query::Name(q) => q.is_empty(),
        }
    }

    /// Whether the search asks for something in the text (not only in names, and not
    /// only for what is absent).
    pub fn wants_text(&self) -> bool {
        match self {
            Query::Term(_) | Query::Near(..) => true,
            Query::All(parts) | Query::Any(parts) => parts.iter().any(Query::wants_text),
            Query::Not(_) | Query::Name(_) => false,
        }
    }
}

/// The largest distance accepted after `~`, in words.
pub const MAX_WITHIN: u32 = 1000;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    Quoted(String, Option<u32>),
    Open,
    Close,
    Either,
    /// A `-` at the start of a word.
    Without,
    /// `name:` at the start of a word.
    InName,
}

const QUOTES: [char; 3] = ['"', '“', '”'];

fn tokens(query: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let chars: Vec<(usize, char)> = query.char_indices().collect();
    let mut i = 0;
    // Whether the next character starts a word (so `-` and `name:` are signs there).
    let mut at_start = true;
    while i < chars.len() {
        let (byte, c) = chars[i];
        if c.is_whitespace() {
            at_start = true;
            i += 1;
        } else if QUOTES.contains(&c) {
            let start = i + 1;
            let mut end = start;
            while end < chars.len() && !QUOTES.contains(&chars[end].1) {
                end += 1;
            }
            let text = slice(query, &chars, start, end);
            i = (end + 1).min(chars.len());
            // `~N` right after the closing quote.
            let mut within = None;
            if end < chars.len() && i < chars.len() && chars[i].1 == '~' {
                let mut j = i + 1;
                while j < chars.len() && chars[j].1.is_ascii_digit() {
                    j += 1;
                }
                if j > i + 1 {
                    within = slice(query, &chars, i + 1, j)
                        .parse::<u32>()
                        .ok()
                        .map(|n| n.clamp(1, MAX_WITHIN));
                    i = j;
                }
            }
            out.push(Token::Quoted(text.to_owned(), within));
            at_start = false;
        } else if c == '(' {
            out.push(Token::Open);
            at_start = true;
            i += 1;
        } else if c == ')' {
            out.push(Token::Close);
            at_start = false;
            i += 1;
        } else if c == '|' {
            out.push(Token::Either);
            at_start = true;
            i += 1;
        } else if c == '-'
            && at_start
            && chars
                .get(i + 1)
                .is_some_and(|n| !n.1.is_whitespace() && n.1 != '-')
        {
            out.push(Token::Without);
            i += 1;
        } else if at_start
            && query
                .get(byte..byte + 5)
                .is_some_and(|w| w.eq_ignore_ascii_case("name:"))
        {
            out.push(Token::InName);
            i += 5;
        } else {
            let start = i;
            while i < chars.len() {
                let c = chars[i].1;
                if c.is_whitespace() || QUOTES.contains(&c) || matches!(c, '(' | ')' | '|') {
                    break;
                }
                i += 1;
            }
            out.push(Token::Word(slice(query, &chars, start, i).to_owned()));
            at_start = false;
        }
    }
    out
}

fn slice<'a>(query: &'a str, chars: &[(usize, char)], start: usize, end: usize) -> &'a str {
    let from = chars.get(start).map_or(query.len(), |c| c.0);
    let to = chars.get(end).map_or(query.len(), |c| c.0);
    &query[from..to]
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    /// Parts separated by spaces, until a `)` or the end.
    fn all(&mut self, depth: usize) -> Query {
        let mut parts = Vec::new();
        while let Some(token) = self.peek() {
            match token {
                Token::Close if depth > 0 => break,
                Token::Close | Token::Either => self.at += 1,
                _ => {
                    let part = self.either(depth);
                    if !part.is_empty() {
                        parts.push(part);
                    }
                }
            }
        }
        if parts.len() == 1 {
            parts.pop().unwrap_or(Query::All(Vec::new()))
        } else {
            Query::All(parts)
        }
    }

    /// One part, or several joined by `|`.
    fn either(&mut self, depth: usize) -> Query {
        let mut options = vec![self.unary(depth)];
        while self.peek() == Some(&Token::Either) {
            self.at += 1;
            match self.peek() {
                None | Some(Token::Close | Token::Either) => break,
                _ => options.push(self.unary(depth)),
            }
        }
        options.retain(|q| !q.is_empty());
        if options.len() == 1 {
            options.pop().unwrap_or(Query::All(Vec::new()))
        } else {
            Query::Any(options)
        }
    }

    fn unary(&mut self, depth: usize) -> Query {
        match self.peek() {
            Some(Token::Without) => {
                self.at += 1;
                match self.peek() {
                    None | Some(Token::Close | Token::Either) => Query::All(Vec::new()),
                    _ => {
                        let q = self.unary(depth);
                        if q.is_empty() {
                            q
                        } else {
                            Query::Not(Box::new(q))
                        }
                    }
                }
            }
            Some(Token::InName) => {
                self.at += 1;
                match self.peek() {
                    None | Some(Token::Close | Token::Either) => Query::All(Vec::new()),
                    _ => {
                        let q = self.unary(depth);
                        if q.is_empty() {
                            q
                        } else {
                            Query::Name(Box::new(q))
                        }
                    }
                }
            }
            _ => self.atom(depth),
        }
    }

    fn atom(&mut self, depth: usize) -> Query {
        let Some(token) = self.tokens.get(self.at).cloned() else {
            return Query::All(Vec::new());
        };
        self.at += 1;
        match token {
            Token::Word(word) => Query::Term(search(&word)),
            Token::Quoted(text, None) => Query::Term(search_text(&text)),
            Token::Quoted(text, Some(within)) => near(&text, within),
            Token::Open => {
                let inner = self.all(depth + 1);
                if self.peek() == Some(&Token::Close) {
                    self.at += 1;
                }
                inner
            }
            // Signs where a word was expected stand for nothing.
            Token::Close | Token::Either | Token::Without | Token::InName => Query::All(Vec::new()),
        }
    }
}

/// `"a b c"~N`: each space-separated piece is a term. Single characters of unspaced
/// scripts have no positions in the index, so they cannot take part and are left out.
fn near(text: &str, within: u32) -> Query {
    let terms: Vec<Search> = text
        .split_whitespace()
        .map(search_text)
        .filter(|s| matches!(s, Search::Phrase(_)))
        .collect();
    match terms.len() {
        0 => Query::All(Vec::new()),
        1 => Query::Term(terms.into_iter().next().unwrap_or(Search::Nothing)),
        _ => Query::Near(terms, within),
    }
}

/// Reads a search typed by the user. Never fails.
pub fn parse(query: &str) -> Query {
    let mut parser = Parser {
        tokens: tokens(query),
        at: 0,
    };
    parser.all(0)
}

#[cfg(test)]
mod tests {
    use super::{Query, parse};
    use crate::Search;

    fn word(w: &str) -> Query {
        Query::Term(Search::Phrase(vec![(0, w.to_owned())]))
    }
    fn phrase(words: &[&str]) -> Search {
        Search::Phrase(
            words
                .iter()
                .enumerate()
                .map(|(i, w)| (i as u32, (*w).to_owned()))
                .collect(),
        )
    }

    #[test]
    fn plain_words_are_all_required() {
        assert_eq!(
            parse("capitu bentinho"),
            Query::All(vec![word("capitu"), word("bentinho")])
        );
        assert_eq!(parse("capitu"), word("capitu"));
        assert!(parse("   ").is_empty());
        assert!(parse("!!! ...").is_empty());
    }

    #[test]
    fn capital_words_are_words() {
        assert_eq!(
            parse("capitu NÃO bentinho"),
            Query::All(vec![word("capitu"), word("nao"), word("bentinho")])
        );
        assert_eq!(
            parse("a OR b"),
            Query::All(vec![word("a"), word("or"), word("b")])
        );
    }

    #[test]
    fn phrases_and_their_signs() {
        assert_eq!(
            parse("\"olhos de ressaca\""),
            Query::Term(phrase(&["olhos", "de", "ressaca"]))
        );
        assert_eq!(
            parse("“olhos de ressaca”"),
            Query::Term(phrase(&["olhos", "de", "ressaca"]))
        );
        // Inside quotes everything is text.
        assert_eq!(
            parse("\"-capitu | bentinho\""),
            Query::Term(phrase(&["capitu", "bentinho"]))
        );
        assert_eq!(parse("\"melancol*\""), word("melancol"));
        // An unclosed quote runs to the end.
        assert_eq!(parse("\"sem fim"), Query::Term(phrase(&["sem", "fim"])));
    }

    #[test]
    fn without() {
        assert_eq!(
            parse("capitu -bentinho"),
            Query::All(vec![word("capitu"), Query::Not(Box::new(word("bentinho")))])
        );
        // Hyphenated words stay words.
        assert_eq!(
            parse("bem-te-vi"),
            Query::Term(phrase(&["bem", "te", "vi"]))
        );
        assert_eq!(
            parse("-\"olhos de ressaca\" capitu"),
            Query::All(vec![
                Query::Not(Box::new(Query::Term(phrase(&["olhos", "de", "ressaca"])))),
                word("capitu")
            ])
        );
        // A lone dash is nothing.
        assert_eq!(
            parse("capitu - bentinho"),
            Query::All(vec![word("capitu"), word("bentinho")])
        );
    }

    #[test]
    fn either_binds_tighter_than_space() {
        assert_eq!(
            parse("capitu | bentinho"),
            Query::Any(vec![word("capitu"), word("bentinho")])
        );
        assert_eq!(
            parse("capitu|bentinho"),
            Query::Any(vec![word("capitu"), word("bentinho")])
        );
        assert_eq!(
            parse("machado capitu | bentinho"),
            Query::All(vec![
                word("machado"),
                Query::Any(vec![word("capitu"), word("bentinho")])
            ])
        );
        // Stray signs are ignored.
        assert_eq!(parse("| capitu |"), word("capitu"));
    }

    #[test]
    fn grouping() {
        assert_eq!(
            parse("(capitu | bentinho) ciúme"),
            Query::All(vec![
                Query::Any(vec![word("capitu"), word("bentinho")]),
                word("ciume")
            ])
        );
        // An unclosed bracket runs to the end; a stray one is ignored.
        assert_eq!(
            parse("(capitu | bentinho"),
            Query::Any(vec![word("capitu"), word("bentinho")])
        );
        assert_eq!(parse("capitu)"), word("capitu"));
        assert!(parse("()").is_empty());
    }

    #[test]
    fn near() {
        assert_eq!(
            parse("\"capitu olhos\"~10"),
            Query::Near(vec![phrase(&["capitu"]), phrase(&["olhos"])], 10)
        );
        // `~` not right after a closing quote is text.
        assert_eq!(
            parse("\"capitu olhos\" ~10"),
            Query::All(vec![Query::Term(phrase(&["capitu", "olhos"])), word("10")])
        );
        // One word with a distance is just the word.
        assert_eq!(parse("\"capitu\"~5"), word("capitu"));
        assert_eq!(
            parse("\"a b\"~0"),
            Query::Near(vec![phrase(&["a"]), phrase(&["b"])], 1)
        );
    }

    #[test]
    fn prefix() {
        assert_eq!(
            parse("melancol*"),
            Query::Term(Search::Prefix("melancol".into()))
        );
        // Multi-byte text where `name:` would be checked.
        assert!(!parse("长江 名字").is_empty());
    }

    #[test]
    fn in_name() {
        assert_eq!(parse("name:assis"), Query::Name(Box::new(word("assis"))));
        assert_eq!(parse("Name:assis"), Query::Name(Box::new(word("assis"))));
        assert_eq!(
            parse("name:\"dom casmurro\" capitu"),
            Query::All(vec![
                Query::Name(Box::new(Query::Term(phrase(&["dom", "casmurro"])))),
                word("capitu")
            ])
        );
        // Not at the start of a word, it is text.
        assert_eq!(parse("rename:x"), word("rename:x"));
    }
}
