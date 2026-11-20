//! Tests of the PDF reader on small files made by `fixtures/make_fixtures.sh`.

use mulu_extract::{Format, Limits, Place, Problem, extract, extract_bytes};
use std::path::PathBuf;

/// The PDFium library: `scripts/fetch-pdfium.sh PLATFORM`, then the path of the library
/// in MULU_PDFIUM (the automated checks do both).
pub fn use_pdfium() {
    let path = std::env::var_os("MULU_PDFIUM")
        .expect("set MULU_PDFIUM to the PDFium library (see scripts/fetch-pdfium.sh)");
    mulu_extract::set_pdfium_library(std::path::Path::new(&path));
}

fn fixture(name: &str) -> PathBuf {
    use_pdfium();
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn pages_keep_their_printed_numbers() {
    let document = extract(&fixture("book.pdf"), &Limits::default()).unwrap();
    let places: Vec<Place> = document.sections.iter().map(|s| s.place.clone()).collect();
    let page = |number, label: &str| Place::Page {
        number,
        label: Some(label.to_owned()),
    };
    assert_eq!(
        places,
        [page(1, "i"), page(2, "ii"), page(3, "1"), page(4, "2")]
    );
    assert!(document.sections[0].text.contains("melancolia dos livros"));
}

#[test]
fn hyphenated_words_and_old_accents_are_searchable() {
    let document = extract(&fixture("book.pdf"), &Limits::default()).unwrap();
    let text = &document.sections[2].text;
    assert!(text.contains("liberdade"), "{text}");
    assert!(text.contains("família"), "{text}");
    assert!(text.contains("justiça"), "{text}");
}

#[test]
fn chinese_lines_are_joined() {
    let document = extract(&fixture("chinese.pdf"), &Limits::default()).unwrap();
    let text: String = document.sections.iter().map(|s| s.text.as_str()).collect();
    assert!(text.contains("人人生而自由"), "{text}");
    assert!(text.contains("兄弟关系的精神"), "{text}");
}

#[test]
fn password_protection_and_copy_restrictions() {
    assert_eq!(
        extract(&fixture("password.pdf"), &Limits::default()),
        Err(Problem::Protected)
    );
    // Copy and print restrictions do not stop reading, as in any PDF reader.
    let document = extract(&fixture("restricted.pdf"), &Limits::default()).unwrap();
    assert!(document.sections[0].text.contains("melancolia"));
}

#[test]
fn a_scan_has_no_text() {
    assert_eq!(
        extract(&fixture("scan.pdf"), &Limits::default()),
        Err(Problem::NoText)
    );
}

#[test]
fn damaged_pdfs_give_a_problem_not_a_crash() {
    use_pdfium();
    let limits = Limits::default();
    assert!(matches!(
        extract_bytes(Format::Pdf, b"", &limits),
        Err(Problem::Corrupted(_))
    ));
    assert!(matches!(
        extract_bytes(Format::Pdf, b"PK\x03\x04 a zip file named .pdf", &limits),
        Err(Problem::Corrupted(_))
    ));
    let book = std::fs::read(fixture("book.pdf")).unwrap();
    for length in (0..book.len()).step_by(97) {
        let _ = extract_bytes(Format::Pdf, &book[..length], &limits);
    }
    let tight = Limits {
        max_pdf_file: 1000,
        ..Limits::default()
    };
    assert_eq!(
        extract(&fixture("book.pdf"), &tight),
        Err(Problem::TooLarge)
    );
}

/// Random damage in a valid PDF: the reader must never crash.
#[test]
fn damaged_bytes_never_crash() {
    let book = std::fs::read(fixture("book.pdf")).unwrap();
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for _ in 0..300 {
        let mut bytes = book.clone();
        for _ in 0..(1 + next() % 16) {
            let at = (next() as usize) % bytes.len();
            bytes[at] = next() as u8;
        }
        let _ = extract_bytes(Format::Pdf, &bytes, &Limits::default());
    }
}
