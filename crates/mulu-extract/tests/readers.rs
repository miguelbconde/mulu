//! Tests of the file readers on whole files: well-formed books, damaged and hostile
//! files, file names in several scripts.

use mulu_extract::{Document, Format, Limits, Place, Problem, extract, extract_bytes};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

const CONTAINER: &str = r#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>"#;

const PACKAGE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Memórias Póstumas de Brás Cubas</dc:title>
    <dc:creator>Machado de Assis</dc:creator>
    <dc:creator>Editora &amp; Cia.</dc:creator>
    <dc:language>pt-BR</dc:language>
    <dc:date>1881</dc:date>
  </metadata>
  <manifest>
    <item id="cover" href="Images/cover.jpg" media-type="image/jpeg"/>
    <item id="c0" href="Text/capa.xhtml" media-type="application/xhtml+xml"/>
    <item id="c1" href="Text/cap%201.xhtml" media-type="application/xhtml+xml"/>
    <item id="c2" href="Text/cap2.xhtml#start" media-type="application/xhtml+xml"/>
    <item id="css" href="Styles/a.css" media-type="text/css"/>
  </manifest>
  <spine><itemref idref="c0"/><itemref idref="c1"/><itemref idref="missing"/><itemref idref="c2"/></spine>
</package>"#;

fn chapter(heading: &str, body: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><html xmlns=\"http://www.w3.org/1999/xhtml\">\
         <head><title>Livro</title></head><body><h2>{heading}</h2><p>{body}</p></body></html>"
    )
}

fn zip_of(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        let method = if *name == "mimetype" {
            CompressionMethod::Stored
        } else {
            CompressionMethod::Deflated
        };
        zip.start_file(
            *name,
            SimpleFileOptions::default().compression_method(method),
        )
        .unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

fn sample_epub(extra: &[(&str, &[u8])]) -> Vec<u8> {
    let cover = "<html><body><img src=\"../Images/cover.jpg\"/></body></html>".to_owned();
    let one = chapter(
        "Óbito do autor",
        "Algum tempo hesitei se devia abrir estas memórias pelo princípio ou pelo fim.",
    );
    let two = chapter(
        "O emplasto",
        "Com efeito, um dia de manhã, estando a passear na chácara…",
    );
    let mut entries: Vec<(&str, &[u8])> = vec![
        ("mimetype", b"application/epub+zip"),
        ("META-INF/container.xml", CONTAINER.as_bytes()),
        ("OEBPS/content.opf", PACKAGE.as_bytes()),
        ("OEBPS/Text/capa.xhtml", cover.as_bytes()),
        ("OEBPS/Text/cap 1.xhtml", one.as_bytes()),
        ("OEBPS/Text/cap2.xhtml", two.as_bytes()),
    ];
    entries.extend_from_slice(extra);
    zip_of(&entries)
}

fn read_epub(bytes: &[u8]) -> Result<Document, Problem> {
    extract_bytes(Format::Epub, bytes, &Limits::default())
}

#[test]
fn epub_metadata_and_chapters_in_reading_order() {
    let document = read_epub(&sample_epub(&[])).unwrap();
    assert_eq!(
        document.title.as_deref(),
        Some("Memórias Póstumas de Brás Cubas")
    );
    assert_eq!(document.authors, ["Machado de Assis", "Editora & Cia."]);
    assert_eq!(document.language.as_deref(), Some("pt-BR"));
    assert_eq!(document.date.as_deref(), Some("1881"));
    // The cover page has no text and is dropped; the missing spine entry is skipped.
    assert_eq!(document.sections.len(), 2);
    assert_eq!(
        document.sections[0].place,
        Place::Part {
            number: 2,
            title: Some("Óbito do autor".into())
        }
    );
    assert!(
        document.sections[0]
            .text
            .contains("pelo princípio ou pelo fim")
    );
    assert_eq!(
        document.sections[1].place,
        Place::Part {
            number: 3,
            title: Some("O emplasto".into())
        }
    );
}

#[test]
fn epub_with_drm_is_reported_as_protected() {
    let adobe = sample_epub(&[("META-INF/rights.xml", b"<rights/>")]);
    assert_eq!(read_epub(&adobe), Err(Problem::Protected));
    let encrypted = br#"<encryption><EncryptedData><EncryptionMethod
        Algorithm="http://www.w3.org/2001/04/xmlenc#aes256-cbc"/></EncryptedData></encryption>"#;
    let lcp = sample_epub(&[("META-INF/encryption.xml", encrypted)]);
    assert_eq!(read_epub(&lcp), Err(Problem::Protected));
}

#[test]
fn epub_with_obfuscated_fonts_is_readable() {
    let fonts = br#"<encryption><EncryptedData><EncryptionMethod
        Algorithm="http://www.idpf.org/2008/embedding"/></EncryptedData></encryption>"#;
    let book = sample_epub(&[("META-INF/encryption.xml", fonts)]);
    assert_eq!(read_epub(&book).unwrap().sections.len(), 2);
}

#[test]
fn damaged_epubs_give_a_problem_not_a_crash() {
    assert!(matches!(
        read_epub(b"not a zip at all"),
        Err(Problem::Corrupted(_))
    ));
    assert!(matches!(read_epub(b""), Err(Problem::Corrupted(_))));
    let no_container = zip_of(&[("mimetype", b"application/epub+zip")]);
    assert!(matches!(
        read_epub(&no_container),
        Err(Problem::Corrupted(_))
    ));
    let bad_package = zip_of(&[
        ("META-INF/container.xml", CONTAINER.as_bytes()),
        (
            "OEBPS/content.opf",
            b"<package><metadata><dc:title>x</wrong>",
        ),
    ]);
    assert!(matches!(
        read_epub(&bad_package),
        Err(Problem::Corrupted(_))
    ));
    // Every truncation of a valid book is handled.
    let book = sample_epub(&[]);
    for length in (0..book.len()).step_by(7) {
        let _ = read_epub(&book[..length]);
    }
}

#[test]
fn epub_limits_stop_zip_bombs() {
    // 50 MB of zeros compress to about 50 KB.
    let zeros = vec![0u8; 50 * 1024 * 1024];
    let bomb = sample_epub(&[("OEBPS/Text/bomb.xhtml", &zeros)]);
    let package = PACKAGE.replace(
        "<itemref idref=\"c2\"/>",
        "<itemref idref=\"c2\"/><itemref idref=\"b\"/>",
    )
    .replace(
        "<item id=\"css\"",
        "<item id=\"b\" href=\"Text/bomb.xhtml\" media-type=\"application/xhtml+xml\"/><item id=\"css\"",
    );
    let bomb = {
        let mut archive = zip::ZipArchive::new(Cursor::new(bomb)).unwrap();
        let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).unwrap();
            let name = entry.name().unwrap().into_owned();
            let mut bytes = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut bytes).unwrap();
            let bytes = if name == "OEBPS/content.opf" {
                package.clone().into_bytes()
            } else {
                bytes
            };
            entries.push((name, bytes));
        }
        let borrowed: Vec<(&str, &[u8])> = entries
            .iter()
            .map(|(n, b)| (n.as_str(), b.as_slice()))
            .collect();
        zip_of(&borrowed)
    };
    assert!(bomb.len() < 1024 * 1024);
    let tight = Limits {
        max_entry: 10 * 1024 * 1024,
        ..Limits::default()
    };
    assert_eq!(
        extract_bytes(Format::Epub, &bomb, &tight),
        Err(Problem::TooLarge)
    );
    let tight = Limits {
        max_unpacked: 20 * 1024 * 1024,
        ..Limits::default()
    };
    assert_eq!(
        extract_bytes(Format::Epub, &bomb, &tight),
        Err(Problem::TooLarge)
    );
}

#[test]
fn text_files_in_several_scripts_and_encodings() {
    let limits = Limits::default();
    let document = extract_bytes(
        Format::Text,
        "Ἄνδρα μοι ἔννεπε, Μοῦσα\nМой дядя самых честных правил\n目录".as_bytes(),
        &limits,
    )
    .unwrap();
    assert!(document.sections[0].text.contains("Μοῦσα"));
    let (latin1, _, _) =
        encoding_rs::WINDOWS_1252.encode("Você já leu \"Iracema\"? Não, nem \"O Guarani\".");
    let document = extract_bytes(Format::Markdown, &latin1, &limits).unwrap();
    assert_eq!(
        document.sections[0].text,
        "Você já leu \"Iracema\"? Não, nem \"O Guarani\"."
    );
    assert_eq!(
        extract_bytes(Format::Text, b"  \n\t ", &limits),
        Err(Problem::NoText)
    );
}

#[test]
fn html_files_declare_their_encoding() {
    let (page, _, _) = encoding_rs::WINDOWS_1252
        .encode("<html><head><meta charset=\"windows-1252\"><title>Poemas</title></head><body><p>Canção do exílio</p></body></html>");
    let document = extract_bytes(Format::Html, &page, &Limits::default()).unwrap();
    assert_eq!(document.title.as_deref(), Some("Poemas"));
    assert_eq!(document.sections[0].text, "Canção do exílio");
}

#[test]
fn files_on_disk_with_names_in_several_scripts() {
    let folder = tempfile::tempdir().unwrap();
    let names = [
        "Assis, Machado (1899) - Dom Casmurro.txt",
        "目录 研究.md",
        "Ἰλιάς.HTML",
        "Пушкин — Евгений Онегин.htm",
        "livro.epub",
    ];
    for name in &names[..4] {
        std::fs::write(folder.path().join(name), "<p>texto de teste</p>").unwrap();
    }
    std::fs::write(folder.path().join(names[4]), sample_epub(&[])).unwrap();
    for name in names {
        let document = extract(&folder.path().join(name), &Limits::default()).unwrap();
        assert!(!document.sections.is_empty(), "{name}");
    }
    assert!(matches!(
        extract(&folder.path().join("ausente.txt"), &Limits::default()),
        Err(Problem::Unreadable(_))
    ));
    assert_eq!(
        extract(&folder.path().join("a.djvu"), &Limits::default()),
        Err(Problem::Unsupported)
    );
    let big = Limits {
        max_text_file: 4,
        ..Limits::default()
    };
    assert_eq!(
        extract(&folder.path().join(names[0]), &big),
        Err(Problem::TooLarge)
    );
}

/// Random bytes, in every format: the readers must never crash or hang.
#[test]
fn random_input_never_crashes() {
    let mut state: u64 = 0x2545_F491_4F6C_DD1D;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let book = sample_epub(&[]);
    for round in 0..2_000 {
        let length = (next() % 2_000) as usize;
        let mut bytes: Vec<u8> = (0..length).map(|_| next() as u8).collect();
        if round % 2 == 0 {
            // Damage a valid book in a few places.
            bytes = book.clone();
            for _ in 0..(1 + next() % 8) {
                let at = (next() as usize) % bytes.len();
                bytes[at] = next() as u8;
            }
        }
        for format in [Format::Pdf, Format::Text, Format::Markdown, Format::Html, Format::Epub] {
            let _ = extract_bytes(format, &bytes, &Limits::default());
        }
    }
}
