//! EPUB books (versions 2 and 3).
//!
//! An EPUB is a ZIP archive of XHTML chapters plus a "package" file that lists the
//! metadata and the reading order (the "spine"). Each chapter in the spine becomes one
//! section. Protected books (Adobe, Apple, Barnes & Noble, Readium LCP) are reported as
//! protected; Mulu never tries to remove the protection. Font obfuscation, which some
//! publishers apply only to embedded fonts, is not a protection of the text.
//!
//! Safety: the archive is read in memory, never unpacked to disk; every entry is size
//! checked while it is read, not only by the size it declares.

use crate::decode::decode;
use crate::html::page_text;
use crate::{Document, Limits, Place, Problem, Section};
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::io::{Read, Seek};
use zip::ZipArchive;
use zip::result::ZipError;

/// Algorithms that only obfuscate embedded fonts (IDPF and Adobe).
const FONT_OBFUSCATION: &[&str] = &[
    "http://www.idpf.org/2008/embedding",
    "http://ns.adobe.com/pdf/enc#RC",
];

/// Files that only protected books contain (Adobe ADEPT, Apple FairPlay, Readium LCP).
const DRM_MARKERS: &[&str] = &[
    "META-INF/rights.xml",
    "META-INF/sinf.xml",
    "META-INF/license.lcpl",
];

struct Archive<R> {
    zip: ZipArchive<R>,
    limits: Limits,
    unpacked: u64,
}

impl<R: Read + Seek> Archive<R> {
    fn has(&self, name: &str) -> bool {
        self.zip.index_for_name(name).is_some()
    }

    /// Reads one entry, enforcing the per-entry and total limits on the bytes actually
    /// unpacked.
    fn read(&mut self, name: &str) -> Result<Option<Vec<u8>>, Problem> {
        let entry = match self.zip.by_name(name) {
            Ok(entry) => entry,
            Err(ZipError::FileNotFound) => return Ok(None),
            Err(ZipError::UnsupportedArchive(detail)) if detail.contains("ncrypt") => {
                return Err(Problem::Protected);
            }
            Err(e) => return Err(Problem::Corrupted(e.to_string())),
        };
        if entry.encrypted() {
            return Err(Problem::Protected);
        }
        let allowed = self
            .limits
            .max_entry
            .min(self.limits.max_unpacked.saturating_sub(self.unpacked));
        if entry.size() > allowed {
            return Err(Problem::TooLarge);
        }
        let mut bytes = Vec::new();
        entry
            .take(allowed + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| Problem::Corrupted(e.to_string()))?;
        if bytes.len() as u64 > allowed {
            return Err(Problem::TooLarge);
        }
        self.unpacked += bytes.len() as u64;
        Ok(Some(bytes))
    }
}

pub fn read<R: Read + Seek>(reader: R, limits: &Limits) -> Result<Document, Problem> {
    let zip = ZipArchive::new(reader).map_err(|e| Problem::Corrupted(e.to_string()))?;
    if zip.len() > limits.max_entries {
        return Err(Problem::TooLarge);
    }
    let mut archive = Archive {
        zip,
        limits: *limits,
        unpacked: 0,
    };

    if DRM_MARKERS.iter().any(|name| archive.has(name)) {
        return Err(Problem::Protected);
    }
    if let Some(encryption) = archive.read("META-INF/encryption.xml")?
        && encrypts_more_than_fonts(&encryption)
    {
        return Err(Problem::Protected);
    }

    let container = archive
        .read("META-INF/container.xml")?
        .ok_or_else(|| Problem::Corrupted("no META-INF/container.xml".into()))?;
    let package_path = package_path(&container)
        .ok_or_else(|| Problem::Corrupted("no package file in container.xml".into()))?;
    let package_bytes = archive
        .read(&package_path)?
        .ok_or_else(|| Problem::Corrupted(format!("package file {package_path} missing")))?;
    let package = parse_package(&package_bytes)?;
    let base = match package_path.rfind('/') {
        Some(slash) => &package_path[..=slash],
        None => "",
    };

    let mut sections = Vec::new();
    let mut number = 0u32;
    for idref in &package.spine {
        let Some(item) = package.manifest.get(idref) else {
            continue;
        };
        if !is_html(&item.media_type) {
            continue;
        }
        let path = resolve(base, &item.href);
        let Some(bytes) = archive.read(&path)? else {
            continue;
        };
        let page = page_text(&decode(&bytes, None));
        number += 1;
        sections.push(Section {
            place: Place::Part {
                number,
                title: page.heading.or(page.title),
            },
            text: page.text,
        });
    }

    Ok(Document {
        title: package.title,
        authors: package.authors,
        language: package.language,
        date: package.date,
        sections,
    })
}

fn is_html(media_type: &str) -> bool {
    matches!(
        media_type.trim().to_ascii_lowercase().as_str(),
        "application/xhtml+xml" | "text/html" | "application/x-dtbook+xml"
    )
}

/// Whether `encryption.xml` encrypts anything with an algorithm other than font
/// obfuscation.
fn encrypts_more_than_fonts(xml: &[u8]) -> bool {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(e) | Event::Empty(e))
                if e.local_name().as_ref() == "EncryptionMethod" =>
            {
                match attribute(&e, "Algorithm") {
                    Some(algorithm) if FONT_OBFUSCATION.contains(&algorithm.trim()) => {}
                    _ => return true,
                }
            }
            Ok(Event::Eof) => return false,
            // An unreadable encryption file is treated as protection, to be safe.
            Err(_) => return true,
            _ => {}
        }
        buffer.clear();
    }
}

fn package_path(container: &[u8]) -> Option<String> {
    let mut reader = Reader::from_reader(container);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(e) | Event::Empty(e)) if e.local_name().as_ref() == "rootfile" => {
                if let Some(path) = attribute(&e, "full-path") {
                    return Some(path.trim_start_matches('/').to_owned());
                }
            }
            Ok(Event::Eof) | Err(_) => return None,
            _ => {}
        }
        buffer.clear();
    }
}

struct Item {
    href: String,
    media_type: String,
}

#[derive(Default)]
struct Package {
    title: Option<String>,
    authors: Vec<String>,
    language: Option<String>,
    date: Option<String>,
    manifest: HashMap<String, Item>,
    spine: Vec<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum Field {
    Title,
    Creator,
    Language,
    Date,
}

fn parse_package(xml: &[u8]) -> Result<Package, Problem> {
    let xml = decode(xml, None);
    let mut reader = Reader::from_str(&xml);
    let mut package = Package::default();
    let mut field: Option<Field> = None;
    let mut value = String::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                field = match e.local_name().as_ref() {
                    "title" => Some(Field::Title),
                    "creator" => Some(Field::Creator),
                    "language" => Some(Field::Language),
                    "date" => Some(Field::Date),
                    _ => None,
                };
                value.clear();
                package_element(&e, &mut package);
            }
            Ok(Event::Empty(e)) => package_element(&e, &mut package),
            Ok(Event::Text(text)) if field.is_some() => value.push_str(&text.into_inner()),
            Ok(Event::CData(text)) if field.is_some() => value.push_str(&text.into_inner()),
            Ok(Event::GeneralRef(reference)) if field.is_some() => {
                if let Ok(Some(c)) = reference.resolve_char_ref() {
                    value.push(c);
                } else {
                    value.push_str(match reference.into_inner().as_ref() {
                        "amp" => "&",
                        "lt" => "<",
                        "gt" => ">",
                        "quot" => "\"",
                        "apos" => "'",
                        _ => "",
                    });
                }
            }
            Ok(Event::End(_)) => {
                if let Some(done) = field.take() {
                    let cleaned = value.split_whitespace().collect::<Vec<_>>().join(" ");
                    if !cleaned.is_empty() {
                        match done {
                            Field::Title if package.title.is_none() => {
                                package.title = Some(cleaned)
                            }
                            Field::Creator => package.authors.push(cleaned),
                            Field::Language if package.language.is_none() => {
                                package.language = Some(cleaned);
                            }
                            Field::Date if package.date.is_none() => package.date = Some(cleaned),
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Problem::Corrupted(format!("package file: {e}"))),
            _ => {}
        }
    }
    Ok(package)
}

fn package_element(e: &BytesStart<'_>, package: &mut Package) {
    match e.local_name().as_ref() {
        "item" => {
            if let (Some(id), Some(href)) = (attribute(e, "id"), attribute(e, "href")) {
                let media_type = attribute(e, "media-type").unwrap_or_default();
                package.manifest.insert(id, Item { href, media_type });
            }
        }
        "itemref" => {
            if let Some(idref) = attribute(e, "idref") {
                package.spine.push(idref);
            }
        }
        _ => {}
    }
}

fn attribute(e: &BytesStart<'_>, name: &str) -> Option<String> {
    let attribute = e.try_get_attribute(name).ok()??;
    Some(
        attribute
            .normalized_value(quick_xml::XmlVersion::Implicit1_0)
            .ok()?
            .into_owned(),
    )
}

/// The path of a manifest entry inside the archive: relative to the package file,
/// without a `#fragment`, with `%20`-style escapes decoded and `..` resolved.
fn resolve(base: &str, href: &str) -> String {
    let href = href.split('#').next().unwrap_or_default();
    let href = percent_decode(href);
    let joined = if let Some(absolute) = href.strip_prefix('/') {
        absolute.to_owned()
    } else {
        format!("{base}{href}")
    };
    let mut parts: Vec<&str> = Vec::new();
    for part in joined.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = &bytes[i + 1..i + 3];
            if let (Some(high), Some(low)) = (hex_value(hex[0]), hex_value(hex[1])) {
                let byte = high * 16 + low;
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    (byte as char)
        .to_digit(16)
        .and_then(|d| u8::try_from(d).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_are_resolved() {
        assert_eq!(
            resolve("OEBPS/", "Text/cap%201.xhtml#p3"),
            "OEBPS/Text/cap 1.xhtml"
        );
        assert_eq!(
            resolve("OEBPS/Text/", "../Text/a.xhtml"),
            "OEBPS/Text/a.xhtml"
        );
        assert_eq!(resolve("", "a.html"), "a.html");
        assert_eq!(percent_decode("a%C3%A7%"), "aç%");
        assert_eq!(percent_decode("%zz%4"), "%zz%4");
    }

    #[test]
    fn font_obfuscation_is_not_protection() {
        let fonts = br#"<encryption xmlns="urn:oasis:names:tc:opendocument:xmlns:container"
            xmlns:enc="http://www.w3.org/2001/04/xmlenc#"><enc:EncryptedData>
            <enc:EncryptionMethod Algorithm="http://www.idpf.org/2008/embedding"/>
            </enc:EncryptedData></encryption>"#;
        assert!(!encrypts_more_than_fonts(fonts));
        let drm = br#"<encryption><EncryptedData>
            <EncryptionMethod Algorithm="http://www.w3.org/2001/04/xmlenc#aes128-cbc"/>
            </EncryptedData></encryption>"#;
        assert!(encrypts_more_than_fonts(drm));
    }
}
