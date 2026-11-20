//! PDF, with PDFium (decision D28, which replaces D24): one section per page, with the
//! page's printed label when the PDF defines one (a book whose page 1 is the 15th page
//! of the file, or whose preface is numbered i, ii, iii).
//!
//! PDFium is Google's PDF engine, the one in Chrome. Mulu loads it from a library file
//! shipped with the program (`pdfium.dll`, `libpdfium.dylib`), built without JavaScript
//! and without XFA forms; it only ever runs inside the separate reading process (D25).
//!
//! Password-protected PDFs are reported as protected. PDFs that are "encrypted" only to
//! restrict printing or copying open as in any PDF reader. A page whose text cannot be
//! read is left empty; the other pages are still indexed. PDFs whose pages are only
//! images (scans) have no text: they are reported as such, for OCR in version 0.5.
//!
//! Metadata (title, author, year) is read in version 0.6, with the catalog.

use crate::cleanup::tidy;
use crate::{Document, Place, Problem, Section};
use pdfium_render::prelude::{Pdfium, PdfiumError, PdfiumInternalError};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static LIBRARY: OnceLock<PathBuf> = OnceLock::new();
static PDFIUM: OnceLock<Result<Pdfium, String>> = OnceLock::new();

/// Tells the reader where the PDFium library is. Without it, the reader looks next to
/// the program, and on macOS in the application's resources.
pub fn set_pdfium_library(path: &Path) {
    let _ = LIBRARY.set(path.to_owned());
}

/// The usual places of the library, for a program started without an explicit path.
fn default_library() -> Option<PathBuf> {
    let program = std::env::current_exe().ok()?;
    let folder = program.parent()?;
    let name = if cfg!(windows) {
        "pdfium.dll"
    } else if cfg!(target_os = "macos") {
        "libpdfium.dylib"
    } else {
        "libpdfium.so"
    };
    [
        folder.join("pdfium").join(name),
        folder.join(name),
        folder.join("../Resources/pdfium").join(name),
    ]
    .into_iter()
    .find(|p| p.is_file())
}

fn pdfium() -> Result<&'static Pdfium, Problem> {
    PDFIUM
        .get_or_init(|| {
            let path = LIBRARY
                .get()
                .cloned()
                .or_else(default_library)
                .ok_or_else(|| "the PDF reader library was not found".to_owned())?;
            Pdfium::bind_to_library(&path)
                .map(Pdfium::new)
                .map_err(|e| format!("the PDF reader library could not be loaded: {e}"))
        })
        .as_ref()
        .map_err(|e| Problem::Unreadable(e.clone()))
}

pub fn read(bytes: &[u8]) -> Result<Document, Problem> {
    // Panics in the Rust part are caught; anything worse ends only the reading process.
    catch_unwind(AssertUnwindSafe(|| read_inner(bytes)))
        .unwrap_or_else(|_| Err(Problem::Corrupted("the PDF reader stopped".into())))
}

fn read_inner(bytes: &[u8]) -> Result<Document, Problem> {
    // The header must appear in the first kilobyte (PDF specification).
    if !bytes[..bytes.len().min(1024)]
        .windows(5)
        .any(|w| w == b"%PDF-")
    {
        return Err(Problem::Corrupted("not a PDF file".into()));
    }
    let pdfium = pdfium()?;
    let document = pdfium
        .load_pdf_from_byte_slice(bytes, None)
        .map_err(|error| match error {
            PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::PasswordError | PdfiumInternalError::SecurityError,
            ) => Problem::Protected,
            other => Problem::Corrupted(other.to_string()),
        })?;
    let mut sections = Vec::new();
    for (index, page) in document.pages().iter().enumerate() {
        let text = page.text().map(|t| t.all()).unwrap_or_default();
        let number = u32::try_from(index + 1).unwrap_or(u32::MAX);
        let label = page
            .label()
            .map(str::trim)
            .filter(|l| !l.is_empty() && *l != number.to_string())
            .map(str::to_owned);
        sections.push(Section {
            place: Place::Page { number, label },
            text: tidy(&normalize(&text)),
        });
    }
    Ok(Document {
        sections,
        ..Document::default()
    })
}

/// PDFium ends lines with CR LF, and marks a word broken by a hyphen at the end of a
/// line with U+0002, joining the two lines itself: the mark is removed, so that the
/// word is whole ("liber" U+0002 "dade" becomes "liberdade").
fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace(['\u{2}', '\u{FFFE}'], "")
}
