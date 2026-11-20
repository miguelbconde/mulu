//! Plain text and Markdown.
//!
//! Markdown is read as plain text in version 0.1: its marks (`#`, `*`, `_`) are
//! punctuation, which the search already ignores, so every word is found; passages
//! show the marks as written.
//!
//! A text file has no pages, unless it was exported with page-break characters
//! (form feed, U+000C), as some converters and printers do: then each part is a page.

use crate::decode::decode;
use crate::{Document, Place, Section};

pub fn read(bytes: &[u8]) -> Document {
    let text = decode(bytes, None);
    let text = normalize_line_ends(&text);
    let sections = if text.contains('\u{000C}') {
        text.split('\u{000C}')
            .enumerate()
            .map(|(i, page)| Section {
                place: Place::Page {
                    number: u32::try_from(i + 1).unwrap_or(u32::MAX),
                    label: None,
                },
                text: page.to_owned(),
            })
            .collect()
    } else {
        vec![Section {
            place: Place::Whole,
            text,
        }]
    };
    Document {
        sections,
        ..Document::default()
    }
}

fn normalize_line_ends(text: &str) -> String {
    if text.contains('\r') {
        text.replace("\r\n", "\n").replace('\r', "\n")
    } else {
        text.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_text_without_page_breaks_is_one_section() {
        let document = read("Linha um\r\nLinha dois".as_bytes());
        assert_eq!(document.sections.len(), 1);
        assert_eq!(document.sections[0].place, Place::Whole);
        assert_eq!(document.sections[0].text, "Linha um\nLinha dois");
    }

    #[test]
    fn page_breaks_make_pages() {
        let document = read("primeira\u{000C}segunda\u{000C}terceira".as_bytes());
        let places: Vec<_> = document.sections.iter().map(|s| s.place.clone()).collect();
        let page = |number| Place::Page {
            number,
            label: None,
        };
        assert_eq!(places, [page(1), page(2), page(3)]);
        assert_eq!(document.sections[1].text, "segunda");
    }
}
