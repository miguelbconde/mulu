//! PDF, with pdf_oxide (decision D24): one section per page, with the page's printed
//! label when the PDF defines one (a book whose page 1 is the 15th page of the file,
//! or whose preface is numbered i, ii, iii).
//!
//! Password-protected PDFs are reported as protected. PDFs that are "encrypted" only to
//! restrict printing or copying open with an empty password, as every PDF reader does,
//! and are read normally. A page that cannot be read is left empty; the other pages are
//! still indexed. PDFs whose pages are only images (scans) have no text: they are
//! reported as such, for OCR in version 0.5.
//!
//! Metadata (title, author, year) is read in version 0.6, with the catalog.

use crate::cleanup::tidy;
use crate::{Document, Place, Problem, Section};
use pdf_oxide::PdfDocument;
use pdf_oxide::extractors::page_labels::PageLabelExtractor;
use std::panic::{AssertUnwindSafe, catch_unwind};

pub fn read(bytes: Vec<u8>) -> Result<Document, Problem> {
    // pdf_oxide has never crashed in our tests (D24); this is a second line of defense
    // behind the separate reading process.
    catch_unwind(AssertUnwindSafe(|| read_inner(bytes)))
        .unwrap_or_else(|_| Err(Problem::Corrupted("the PDF reader stopped".into())))
}

fn read_inner(bytes: Vec<u8>) -> Result<Document, Problem> {
    // The header must appear in the first kilobyte (PDF specification).
    if !bytes[..bytes.len().min(1024)]
        .windows(5)
        .any(|w| w == b"%PDF-")
    {
        return Err(Problem::Corrupted("not a PDF file".into()));
    }
    let document = PdfDocument::from_bytes(bytes).map_err(|e| Problem::Corrupted(e.to_string()))?;
    if document.is_encrypted() && !matches!(document.authenticate(b""), Ok(true)) {
        return Err(Problem::Protected);
    }
    let count = document
        .page_count()
        .map_err(|e| Problem::Corrupted(e.to_string()))?;
    let labels = PageLabelExtractor::extract(&document).unwrap_or_default();
    let mut sections = Vec::with_capacity(count);
    for index in 0..count {
        let text = document.extract_text(index).unwrap_or_default();
        let number = u32::try_from(index + 1).unwrap_or(u32::MAX);
        let label = PageLabelExtractor::get_label(&labels, index);
        sections.push(Section {
            place: Place::Page {
                number,
                label: (label != number.to_string() && !label.is_empty()).then_some(label),
            },
            text: tidy(&text),
        });
    }
    Ok(Document {
        sections,
        ..Document::default()
    })
}
