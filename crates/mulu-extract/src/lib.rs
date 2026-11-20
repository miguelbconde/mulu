//! # mulu-extract
//!
//! Mulu's file readers. Each reader turns one file into a [`Document`]: its metadata
//! (title, authors, language, date, when the file declares them) and its text, split
//! into [`Section`]s: pages where the format has fixed pages, chapters or parts where
//! it does not. Each section becomes one entry in the index (decision D20).
//!
//! Every file is treated as untrusted (project principles): nothing in it is ever
//! executed, sizes are checked before anything is unpacked, and a damaged file gives
//! a [`Problem`] instead of a crash. These readers run inside the separate reading
//! process, which adds time and memory limits on top of the ones here.
//!
//! Formats in version 0.1: PDF, EPUB, plain text, Markdown and HTML.

mod cleanup;
mod decode;
mod epub;
mod html;
mod pdf;
mod plain;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// The file formats Mulu can read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Format {
    Pdf,
    Text,
    Markdown,
    Html,
    Epub,
}

impl Format {
    /// The format of a file, from its extension; `None` if Mulu does not read it.
    pub fn from_path(path: &Path) -> Option<Format> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "txt" | "text" => Some(Format::Text),
            "md" | "markdown" => Some(Format::Markdown),
            "html" | "htm" | "xhtml" => Some(Format::Html),
            "epub" => Some(Format::Epub),
            "pdf" => Some(Format::Pdf),
            _ => None,
        }
    }
}

/// Where a section sits in its file, so a passage can say where it was found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Place {
    /// The whole file, for formats with no pages or chapters.
    Whole,
    /// A page, counted from 1 in the file, with the number printed on it when the file
    /// says it differs (a book's page "1" may be the 15th page of the PDF, a preface
    /// may be numbered "iv").
    Page { number: u32, label: Option<String> },
    /// One part of a book without fixed pages (an EPUB chapter, for instance), counted
    /// from 1 in reading order, with its heading when the file has one.
    Part { number: u32, title: Option<String> },
}

/// One page, chapter or part of a file, with its text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Section {
    pub place: Place,
    pub text: String,
}

/// What a reader found in a file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    pub title: Option<String>,
    pub authors: Vec<String>,
    /// Language as declared by the file (for example "pt-BR"); not detected.
    pub language: Option<String>,
    /// Date as declared by the file, unparsed (for example "1999" or "1999-04-01").
    pub date: Option<String>,
    pub sections: Vec<Section>,
}

/// Why a file could not be read. The interface turns each kind into a plain-language
/// message; the file can still be found by its name (decision D21).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Problem {
    /// Not a format Mulu reads.
    Unsupported,
    /// Protected by DRM. Mulu lists such files and never tries to remove the protection.
    Protected,
    /// Damaged or not what its extension says; the detail is for the log, not the user.
    Corrupted(String),
    /// Larger than the limits allow.
    TooLarge,
    /// Readable, but with no text inside (only images, for example).
    NoText,
    /// The file could not be opened (missing, no permission, still in the cloud…).
    Unreadable(String),
    /// Stored in the cloud and not downloaded to this computer: not read, so that Mulu
    /// never makes the cloud service download a whole library.
    InCloud,
    /// Reading took longer than the time limit; the reading process was stopped.
    TimedOut,
    /// The reading process stopped unexpectedly while reading this file.
    ReaderStopped,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Problem::Unsupported => write!(f, "format not supported"),
            Problem::Protected => write!(f, "protected by DRM"),
            Problem::Corrupted(detail) => write!(f, "damaged file: {detail}"),
            Problem::TooLarge => write!(f, "file too large"),
            Problem::NoText => write!(f, "no text found"),
            Problem::Unreadable(detail) => write!(f, "could not open the file: {detail}"),
            Problem::InCloud => write!(f, "in the cloud, not downloaded"),
            Problem::TimedOut => write!(f, "reading took too long"),
            Problem::ReaderStopped => write!(f, "the reader stopped on this file"),
        }
    }
}

impl std::error::Error for Problem {}

/// Size limits that protect the computer from huge or malicious files.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Largest plain-text, Markdown or HTML file read, in bytes.
    pub max_text_file: u64,
    /// Largest EPUB file opened, in bytes.
    pub max_archive_file: u64,
    /// Largest PDF file read, in bytes. The PDF reader holds the whole file in memory
    /// (twice, briefly), so very large PDFs, usually scans without text, are skipped
    /// until version 0.5 (OCR) reads them page by page.
    pub max_pdf_file: u64,
    /// Most bytes unpacked from one archive, all entries together (guards against
    /// "zip bombs", small files that expand to fill the memory).
    pub max_unpacked: u64,
    /// Most bytes unpacked from a single archive entry.
    pub max_entry: u64,
    /// Most entries in one archive.
    pub max_entries: usize,
}

impl Default for Limits {
    fn default() -> Self {
        const MB: u64 = 1024 * 1024;
        Limits {
            max_text_file: 256 * MB,
            max_archive_file: 2048 * MB,
            max_pdf_file: 512 * MB,
            max_unpacked: 1024 * MB,
            max_entry: 128 * MB,
            max_entries: 50_000,
        }
    }
}

/// Reads a file, choosing the reader by its extension.
pub fn extract(path: &Path, limits: &Limits) -> Result<Document, Problem> {
    let format = Format::from_path(path).ok_or(Problem::Unsupported)?;
    let file = File::open(path).map_err(|e| Problem::Unreadable(e.to_string()))?;
    let size = file
        .metadata()
        .map_err(|e| Problem::Unreadable(e.to_string()))?
        .len();
    match format {
        Format::Epub => {
            if size > limits.max_archive_file {
                return Err(Problem::TooLarge);
            }
            finish(epub::read(file, limits)?)
        }
        Format::Pdf | Format::Text | Format::Markdown | Format::Html => {
            let max = if format == Format::Pdf {
                limits.max_pdf_file
            } else {
                limits.max_text_file
            };
            if size > max {
                return Err(Problem::TooLarge);
            }
            let mut bytes = Vec::new();
            file.take(max + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| match e.kind() {
                    // The reading process's memory allowance was reached (D25).
                    std::io::ErrorKind::OutOfMemory => Problem::TooLarge,
                    _ => Problem::Unreadable(e.to_string()),
                })?;
            extract_bytes(format, &bytes, limits)
        }
    }
}

/// Reads a file already loaded in memory.
pub fn extract_bytes(format: Format, bytes: &[u8], limits: &Limits) -> Result<Document, Problem> {
    let document = match format {
        Format::Pdf => {
            if bytes.len() as u64 > limits.max_pdf_file {
                return Err(Problem::TooLarge);
            }
            pdf::read(bytes.to_vec())?
        }
        Format::Text | Format::Markdown => {
            if bytes.len() as u64 > limits.max_text_file {
                return Err(Problem::TooLarge);
            }
            plain::read(bytes)
        }
        Format::Html => {
            if bytes.len() as u64 > limits.max_text_file {
                return Err(Problem::TooLarge);
            }
            html::read_file(bytes)
        }
        Format::Epub => epub::read(std::io::Cursor::new(bytes), limits)?,
    };
    finish(document)
}

/// Drops empty sections; a file with no text at all is reported as such.
fn finish(mut document: Document) -> Result<Document, Problem> {
    document
        .sections
        .retain(|section| section.text.chars().any(|c| !c.is_whitespace()));
    if document.sections.is_empty() {
        return Err(Problem::NoText);
    }
    Ok(document)
}
