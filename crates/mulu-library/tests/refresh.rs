//! Checking a library for changes (D16, D31): new, changed, renamed, moved and deleted
//! files, without reading again what did not change.

use mulu_library::{Changes, Library};
use mulu_reader::{Reader, Settings};
use std::path::{Path, PathBuf};

fn reader() -> Reader {
    let pdfium = std::env::var_os("MULU_PDFIUM")
        .expect("set MULU_PDFIUM to the PDFium library (see scripts/fetch-pdfium.sh)");
    let settings = Settings {
        pdfium: Some(PathBuf::from(pdfium)),
        ..Settings::default()
    };
    Reader::new(env!("CARGO_BIN_EXE_mulu-library-worker"), settings)
}

fn write(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// Reads what is pending and returns how many files were read.
fn read(library: &Library) -> u64 {
    let mut reads = 0;
    library
        .index_pending(vec![reader()], |p| {
            if p.current.is_some() {
                reads += 1;
            }
            true
        })
        .unwrap();
    reads
}

fn names(library: &Library, query: &str) -> Vec<String> {
    let mut found: Vec<String> = library
        .search(query)
        .unwrap()
        .files
        .into_iter()
        .map(|f| f.path.replace('\\', "/"))
        .collect();
    found.sort();
    found
}

#[test]
fn only_what_changed_is_read_again() {
    let folder = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    let root = folder.path();
    // Long enough for both ends of the fingerprint (64 KB each) to be read.
    let long = "palavra comum ".repeat(12_000);
    write(
        root,
        "A/Camões - Os Lusíadas.txt",
        &format!("As armas e os barões {long}"),
    );
    write(
        root,
        "A/Pessoa - Mensagem.txt",
        "Mar português, ó mar salgado",
    );
    write(
        root,
        "B/Bandeira - Libertinagem.txt",
        "Vou-me embora pra Pasárgada",
    );
    write(
        root,
        "B/Drummond - Alguma poesia.txt",
        "No meio do caminho tinha uma pedra",
    );

    let library = Library::open(root, store.path()).unwrap();
    assert_eq!(library.checked().unwrap(), None);
    let first = library.refresh().unwrap();
    assert_eq!(
        first,
        Changes {
            added: 4,
            ..Changes::default()
        }
    );
    library.index_pending(vec![reader()], |_| true).unwrap();
    assert!(library.checked().unwrap().is_some());

    // Nothing changed: nothing to read.
    assert_eq!(library.refresh().unwrap(), Changes::default());
    assert_eq!(read(&library), 0);

    // A renamed file, a file moved to another folder, a new file, a changed file
    // (different size) and a deleted file.
    std::fs::rename(
        root.join("A/Camões - Os Lusíadas.txt"),
        root.join("A/CAMÕES (1572) - Os Lusíadas.txt"),
    )
    .unwrap();
    std::fs::create_dir_all(root.join("C/Modernismo")).unwrap();
    std::fs::rename(
        root.join("B/Bandeira - Libertinagem.txt"),
        root.join("C/Modernismo/Bandeira - Libertinagem.txt"),
    )
    .unwrap();
    write(
        root,
        "C/Cecília - Viagem.txt",
        "Eu canto porque o instante existe",
    );
    write(
        root,
        "A/Pessoa - Mensagem.txt",
        "Mar português, ó mar salgado, quanto do teu sal",
    );
    std::fs::remove_file(root.join("B/Drummond - Alguma poesia.txt")).unwrap();

    let changes = library.refresh().unwrap();
    assert_eq!(
        changes,
        Changes {
            added: 1,
            changed: 1,
            moved: 2,
            removed: 1,
        }
    );
    // Only the new and the changed file are read.
    assert_eq!(read(&library), 2);

    // Renamed and moved books are found at their new place, without being read again.
    assert_eq!(
        names(&library, "barões"),
        ["A/CAMÕES (1572) - Os Lusíadas.txt"]
    );
    assert_eq!(
        names(&library, "pasárgada"),
        ["C/Modernismo/Bandeira - Libertinagem.txt"]
    );
    // The changed file has its new text; the deleted one left the index.
    assert_eq!(names(&library, "sal"), ["A/Pessoa - Mensagem.txt"]);
    assert!(names(&library, "pedra").is_empty());
    assert_eq!(names(&library, "instante"), ["C/Cecília - Viagem.txt"]);
    assert_eq!(library.progress().unwrap().total, 4);
}

#[test]
fn a_moved_copy_and_its_original_stay_two_files() {
    // Two identical files: when one moves, the other is still there, so nothing is
    // taken for a move that is not one.
    let folder = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    let root = folder.path();
    write(root, "A/livro.txt", "um texto repetido");
    write(root, "B/livro.txt", "um texto repetido");
    let library = Library::open(root, store.path()).unwrap();
    library.refresh().unwrap();
    library.index_pending(vec![reader()], |_| true).unwrap();

    std::fs::create_dir_all(root.join("C")).unwrap();
    std::fs::rename(root.join("B/livro.txt"), root.join("C/livro.txt")).unwrap();
    let changes = library.refresh().unwrap();
    assert_eq!(
        changes,
        Changes {
            moved: 1,
            ..Changes::default()
        }
    );
    assert_eq!(names(&library, "repetido"), ["A/livro.txt", "C/livro.txt"]);
}

#[test]
fn a_library_from_version_0_1_is_checked_without_reading_everything_again() {
    // Catalogs written by 0.1 have no fingerprints: the files that did not change are
    // not read again; a moved file is read again once, at its new place.
    let folder = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    let root = folder.path();
    write(root, "A/um.txt", "primeiro livro");
    write(root, "A/dois.txt", "segundo livro");
    {
        let library = Library::open(root, store.path()).unwrap();
        library.refresh().unwrap();
        library.index_pending(vec![reader()], |_| true).unwrap();
    }
    let db = rusqlite::Connection::open(store.path().join("catalog.sqlite")).unwrap();
    db.execute_batch("UPDATE files SET fingerprint = NULL; DELETE FROM meta;")
        .unwrap();
    drop(db);

    std::fs::rename(root.join("A/dois.txt"), root.join("A/2.txt")).unwrap();
    let library = Library::open(root, store.path()).unwrap();
    let changes = library.refresh().unwrap();
    assert_eq!(
        changes,
        Changes {
            added: 1,
            removed: 1,
            ..Changes::default()
        }
    );
    assert_eq!(read(&library), 1);
    assert_eq!(names(&library, "livro"), ["A/2.txt", "A/um.txt"]);
}

#[test]
fn a_missing_library_folder_keeps_its_index() {
    // An external disk that is not plugged in: checking must not empty the index.
    let parent = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    let root = parent.path().join("Disco externo");
    write(&root, "A/livro.txt", "um livro no disco externo");
    let library = Library::open(&root, store.path()).unwrap();
    library.refresh().unwrap();
    library.index_pending(vec![reader()], |_| true).unwrap();

    std::fs::rename(&root, parent.path().join("desligado")).unwrap();
    assert!(library.refresh().is_err());
    assert_eq!(library.progress().unwrap().total, 1);
    assert_eq!(names(&library, "externo"), ["A/livro.txt"]);
}
