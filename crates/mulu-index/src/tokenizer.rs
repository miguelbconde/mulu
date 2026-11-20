//! Connects mulu-text to Tantivy: the index stores exactly the terms that mulu-text
//! produces, so indexing and searching always agree.

use tantivy::tokenizer::{Token, TokenStream, Tokenizer};

/// Name under which the main tokenizer is registered in the index.
pub const MAIN: &str = "mulu";
/// Name under which the single-character tokenizer is registered.
pub const SINGLE_CHARACTERS: &str = "mulu-chars";

#[derive(Clone, Copy)]
pub struct MuluTokenizer {
    single_characters: bool,
}

impl MuluTokenizer {
    pub fn main() -> Self {
        Self {
            single_characters: false,
        }
    }

    pub fn single_characters() -> Self {
        Self {
            single_characters: true,
        }
    }
}

pub struct Terms {
    tokens: Vec<Token>,
    next: usize,
}

impl TokenStream for Terms {
    fn advance(&mut self) -> bool {
        if self.next < self.tokens.len() {
            self.next += 1;
            true
        } else {
            false
        }
    }

    fn token(&self) -> &Token {
        &self.tokens[self.next - 1]
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.tokens[self.next - 1]
    }
}

impl Tokenizer for MuluTokenizer {
    type TokenStream<'a> = Terms;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Terms {
        let terms = if self.single_characters {
            mulu_text::single_character_terms(text)
        } else {
            mulu_text::index_terms(text)
        };
        let tokens = terms
            .into_iter()
            .map(|t| Token {
                offset_from: t.start,
                offset_to: t.end,
                position: t.position as usize,
                text: t.text,
                position_length: 1,
            })
            .collect();
        Terms { tokens, next: 0 }
    }
}
