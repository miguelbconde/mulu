//! The reading process keeps Mulu safe from files that hang, use too much memory or
//! crash the reader.

use mulu_extract::Problem;
use mulu_reader::{Reader, Settings};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn worker() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mulu-read-worker"))
}

fn write(folder: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = folder.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn files_are_read_in_the_reading_process() {
    let folder = tempfile::tempdir().unwrap();
    let mut reader = Reader::new(worker(), Settings::default());
    let a = write(
        folder.path(),
        "Machado (1899) - Dom Casmurro.txt",
        "Uma noite destas, vindo da cidade".as_bytes(),
    );
    let b = write(folder.path(), "目录.html", "<p>人人生而自由</p>".as_bytes());
    assert!(
        reader.read(&a).unwrap().sections[0]
            .text
            .contains("vindo da cidade")
    );
    assert!(
        reader.read(&b).unwrap().sections[0]
            .text
            .contains("人人生而自由")
    );
    assert_eq!(
        reader.read(&folder.path().join("x.djvu")),
        Err(Problem::Unsupported)
    );
    assert_eq!(
        reader.read(&write(folder.path(), "vazio.txt", b"  ")),
        Err(Problem::NoText)
    );
}

#[test]
fn a_file_over_the_memory_allowance_is_too_large_and_the_next_file_is_read() {
    let folder = tempfile::tempdir().unwrap();
    // 40 MB of text against an allowance of 16 MB.
    let big = write(
        folder.path(),
        "grande.txt",
        &b"palavra ".repeat(5 * 1024 * 1024),
    );
    let small = write(folder.path(), "pequeno.txt", b"melancolia");
    let settings = Settings {
        memory_limit: 16 * 1024 * 1024,
        ..Settings::default()
    };
    let mut reader = Reader::new(worker(), settings);
    assert_eq!(reader.read(&big), Err(Problem::TooLarge));
    assert!(
        reader.read(&small).unwrap().sections[0]
            .text
            .contains("melancolia")
    );
}

#[test]
fn a_reader_that_runs_out_of_memory_while_working_is_stopped() {
    let folder = tempfile::tempdir().unwrap();
    // The file fits in the allowance, but turning it into text needs more.
    let page = write(
        folder.path(),
        "pagina.html",
        &b"<p>palavra</p>".repeat(900 * 1024),
    );
    let settings = Settings {
        memory_limit: 16 * 1024 * 1024,
        ..Settings::default()
    };
    let mut reader = Reader::new(worker(), settings);
    assert_eq!(reader.read(&page), Err(Problem::TooLarge));
}

#[test]
fn a_file_over_the_time_limit_is_stopped_and_the_next_file_is_read() {
    let folder = tempfile::tempdir().unwrap();
    let big = write(
        folder.path(),
        "lento.html",
        &b"<p>uma frase qualquer</p>".repeat(2 * 1024 * 1024),
    );
    let small = write(folder.path(), "pequeno.txt", b"saudade");
    let mut reader = Reader::new(
        worker(),
        Settings {
            time_limit: Duration::from_millis(1),
            ..Settings::default()
        },
    );
    let start = Instant::now();
    assert_eq!(reader.read(&big), Err(Problem::TimedOut));
    assert!(start.elapsed() < Duration::from_secs(5));
    let mut reader = Reader::new(worker(), Settings::default());
    assert!(
        reader.read(&small).unwrap().sections[0]
            .text
            .contains("saudade")
    );
}

#[test]
fn a_reading_process_that_disappears_is_replaced() {
    let folder = tempfile::tempdir().unwrap();
    let file = write(folder.path(), "a.txt", b"primeiro");
    let mut reader = Reader::new(worker(), Settings::default());
    assert!(reader.read(&file).is_ok());
    kill(reader.process_id().unwrap());
    std::thread::sleep(Duration::from_millis(200));
    assert!(
        reader.read(&file).unwrap().sections[0]
            .text
            .contains("primeiro")
    );
}

#[cfg(unix)]
fn kill(pid: u32) {
    let _ = std::process::Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .status();
}

#[cfg(windows)]
fn kill(pid: u32) {
    let _ = std::process::Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .status();
}

#[test]
fn pdfs_are_read_in_the_reading_process() {
    let pdfium = std::env::var_os("MULU_PDFIUM")
        .expect("set MULU_PDFIUM to the PDFium library (see scripts/fetch-pdfium.sh)");
    let settings = Settings {
        pdfium: Some(PathBuf::from(pdfium)),
        ..Settings::default()
    };
    let mut reader = Reader::new(worker(), settings);
    let book =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../mulu-extract/tests/fixtures/book.pdf");
    let document = reader.read(&book).unwrap();
    assert_eq!(document.sections.len(), 4);
    assert!(document.sections[2].text.contains("liberdade"));
}

#[test]
fn settings_round_trip_through_the_command_line() {
    let settings = Settings::from_args([
        "--read-worker".into(),
        "--memory=1234".into(),
        "--seconds=2.5".into(),
        "--pdfium=/a b/libpdfium.so".into(),
    ]);
    assert_eq!(settings.pdfium, Some(PathBuf::from("/a b/libpdfium.so")));
    assert_eq!(settings.memory_limit, 1234);
    assert_eq!(settings.time_limit, Duration::from_millis(2500));
}
