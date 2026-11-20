//! The text of an HTML or XHTML page (a web page saved to disk, or a chapter of an EPUB).
//!
//! Uses the HTML tokenizer of html5ever, the parser from the Servo browser engine
//! (already part of Mulu through Tauri). Only the tokenizer is used, not the full
//! document tree, so memory stays proportional to the text. Scripts, styles and other
//! non-text content are skipped and never run; line breaks are kept between blocks
//! (paragraphs, headings, list items) so that passages read naturally.

use crate::decode::{decode, html_declared};
use crate::{Document, Place, Section};
use html5ever::tendril::StrTendril;
use html5ever::tokenizer::states::RawKind;
use html5ever::tokenizer::{
    BufferQueue, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts,
};
use std::cell::RefCell;

/// The text of one page, with its `<title>` and its first heading.
#[derive(Debug, Default)]
pub struct PageText {
    pub title: Option<String>,
    pub heading: Option<String>,
    pub text: String,
}

/// Reads a standalone HTML file.
pub fn read_file(bytes: &[u8]) -> Document {
    let text = decode(bytes, html_declared(bytes));
    let page = page_text(&text);
    Document {
        title: page.title,
        sections: vec![Section {
            place: Place::Whole,
            text: page.text,
        }],
        ..Document::default()
    }
}

/// Extracts the text of an HTML page.
pub fn page_text(html: &str) -> PageText {
    let tokenizer = Tokenizer::new(Sink::default(), TokenizerOpts::default());
    let queue = BufferQueue::default();
    queue.push_back(StrTendril::from_slice(html));
    let _ = tokenizer.feed(&queue);
    tokenizer.end();
    let state = tokenizer.sink.state.into_inner();
    PageText {
        title: clean(&state.title),
        heading: state.heading,
        text: state.text,
    }
}

#[derive(Default)]
struct Sink {
    state: RefCell<State>,
}

#[derive(Default)]
struct State {
    text: String,
    title: String,
    heading: Option<String>,
    /// Text of the heading being read, until the first non-empty one is kept.
    heading_buffer: Option<String>,
    in_title: bool,
    /// Depth inside elements whose content is not text (`<script>`, `<style>`…).
    skip_depth: u32,
    pending_space: bool,
    pending_break: bool,
}

/// Elements that start a new line in the extracted text.
const BLOCKS: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "body",
    "br",
    "caption",
    "dd",
    "div",
    "dl",
    "dt",
    "figcaption",
    "figure",
    "footer",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "li",
    "main",
    "nav",
    "ol",
    "p",
    "pre",
    "section",
    "table",
    "td",
    "th",
    "tr",
    "ul",
];

/// Elements whose content is never text for the reader.
const SKIPPED: &[&str] = &[
    "script", "style", "template", "noembed", "noframes", "iframe",
];

impl State {
    fn push_text(&mut self, chunk: &str) {
        if self.in_title {
            self.title.push_str(chunk);
            return;
        }
        if self.skip_depth > 0 {
            return;
        }
        if let Some(buffer) = &mut self.heading_buffer {
            buffer.push_str(chunk);
        }
        for c in chunk.chars() {
            if c.is_whitespace() {
                self.pending_space = true;
                continue;
            }
            if !self.text.is_empty() {
                if self.pending_break {
                    self.text.push('\n');
                } else if self.pending_space {
                    self.text.push(' ');
                }
            }
            self.pending_break = false;
            self.pending_space = false;
            self.text.push(c);
        }
    }

    fn tag(&mut self, kind: TagKind, name: &str, self_closing: bool) -> TokenSinkResult<()> {
        let start = kind == TagKind::StartTag;
        if BLOCKS.contains(&name) {
            self.pending_break = true;
        }
        if matches!(name, "h1" | "h2" | "h3") && self.heading.is_none() {
            if start {
                self.heading_buffer = Some(String::new());
            } else if let Some(buffer) = self.heading_buffer.take() {
                self.heading = clean(&buffer);
            }
        }
        if name == "title" {
            self.in_title = start && !self_closing;
            return if self.in_title {
                TokenSinkResult::RawData(RawKind::Rcdata)
            } else {
                TokenSinkResult::Continue
            };
        }
        if SKIPPED.contains(&name) || name == "head" {
            if start && !self_closing {
                self.skip_depth += 1;
            } else if !start {
                self.skip_depth = self.skip_depth.saturating_sub(1);
            }
            if start && !self_closing {
                return match name {
                    "script" => TokenSinkResult::RawData(RawKind::ScriptData),
                    "head" => TokenSinkResult::Continue,
                    _ => TokenSinkResult::RawData(RawKind::Rawtext),
                };
            }
            return TokenSinkResult::Continue;
        }
        if name == "textarea" && start {
            return TokenSinkResult::RawData(RawKind::Rcdata);
        }
        if name == "plaintext" && start {
            return TokenSinkResult::Plaintext;
        }
        // `<body>` ends a `<head>` left open.
        if name == "body" && start {
            self.skip_depth = 0;
        }
        TokenSinkResult::Continue
    }
}

impl TokenSink for Sink {
    type Handle = ();

    fn process_token(&self, token: Token, _line_number: u64) -> TokenSinkResult<()> {
        let mut state = self.state.borrow_mut();
        match token {
            Token::CharacterTokens(text) => {
                state.push_text(&text);
                TokenSinkResult::Continue
            }
            Token::TagToken(tag) => state.tag(tag.kind, &tag.name, tag.self_closing),
            _ => TokenSinkResult::Continue,
        }
    }
}

/// Collapses whitespace; `None` for empty text.
fn clean(text: &str) -> Option<String> {
    let cleaned = text.split_whitespace().collect::<Vec<_>>().join(" ");
    (!cleaned.is_empty()).then_some(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_title_and_heading() {
        let page = page_text(
            "<!DOCTYPE html><html><head><title> Dom  Casmurro </title>\
             <style>p { color: red }</style><script>if (a < b) { x = '</p>' }</script></head>\
             <body><h1>Capítulo I</h1><p>Do título.</p><p>Uma noite destas,\n   vindo da cidade…</p></body></html>",
        );
        assert_eq!(page.title.as_deref(), Some("Dom Casmurro"));
        assert_eq!(page.heading.as_deref(), Some("Capítulo I"));
        assert_eq!(
            page.text,
            "Capítulo I\nDo título.\nUma noite destas, vindo da cidade…"
        );
    }

    #[test]
    fn entities_are_decoded() {
        let page = page_text("<p>Fran&ccedil;ois &amp; Marie &#x76EE;&#24405; &nbsp;fim</p>");
        assert_eq!(page.text, "François & Marie 目录 fim");
    }

    #[test]
    fn inline_elements_do_not_split_words() {
        let page = page_text("<p>melan<i>co</i>lia e <b>saudade</b></p>");
        assert_eq!(page.text, "melancolia e saudade");
    }

    #[test]
    fn xhtml_self_closing_tags() {
        let page = page_text(
            "<?xml version=\"1.0\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title/>\
             <link rel=\"stylesheet\" href=\"a.css\"/></head><body><p>um<br/>dois</p></body></html>",
        );
        assert_eq!(page.title, None);
        assert_eq!(page.text, "um\ndois");
    }

    #[test]
    fn broken_html_never_fails() {
        for input in [
            "<p",
            "<<<>>>",
            "</head></head><p>texto",
            "<script>nunca fecha",
            "&#xFFFFFFFF;",
        ] {
            let _ = page_text(input);
        }
        assert_eq!(page_text("</head></head><p>texto").text, "texto");
    }
}
