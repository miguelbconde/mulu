//! A whole library, from choosing a folder to searching it.

use mulu_extract::Problem;
use mulu_library::Library;
use mulu_reader::{Reader, Settings};
use std::path::{Path, PathBuf};

fn reader() -> Reader {
    Reader::new(
        env!("CARGO_BIN_EXE_mulu-library-worker"),
        Settings::default(),
    )
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../mulu-extract/tests/fixtures")
        .join(name)
}

/// A small library: books in subfolders, names in several scripts, a protected PDF, an
/// iCloud placeholder, a hidden file and files of other kinds.
fn books(root: &Path) {
    let write = |relative: &str, bytes: &[u8]| {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    };
    write(
        "A/ASSIS, Machado (1899) - Dom Casmurro.txt",
        "Capitu tinha olhos de ressaca. A liberdade.".as_bytes(),
    );
    write("A/Notas.md", b"# Notas\n\nA melancolia dos livros.");
    write("Z/鲁迅 - 呐喊.html", "<p>人人生而自由</p>".as_bytes());
    write(
        "Z/sub/Declaração.pdf",
        &std::fs::read(fixture("book.pdf")).unwrap(),
    );
    write(
        "Z/sub/Com senha.pdf",
        &std::fs::read(fixture("password.pdf")).unwrap(),
    );
    write("Z/.Na nuvem.epub.icloud", b"placeholder");
    write("Z/.escondido.txt", b"liberdade escondida");
    write("Z/foto.jpg", b"\xff\xd8\xff");
}

#[test]
fn choose_index_and_search_a_library() {
    let folder = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    books(folder.path());

    let library = Library::open(folder.path(), store.path()).unwrap();
    assert_eq!(library.scan().unwrap(), 6);

    let mut reports = Vec::new();
    library
        .index_pending(vec![reader(), reader()], |p| {
            reports.push((p.done, p.total));
            true
        })
        .unwrap();
    assert_eq!(reports.last(), Some(&(6, 6)));

    // Text search: most occurrences first; the PDF has "liberdade" on two pages.
    let answer = library.search("liberdade").unwrap();
    let names: Vec<&str> = answer.files.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(
        names,
        ["Declaração.pdf", "ASSIS, Machado (1899) - Dom Casmurro.txt"]
    );
    assert_eq!(
        answer.files[0].folder,
        Path::new("Z").join("sub").to_string_lossy()
    );
    let passages = library
        .passages("liberdade", answer.files[0].id, 0, 10)
        .unwrap();
    assert_eq!(passages.len(), 2);
    assert_eq!(passages[0].place, "p:1");
    assert_eq!(passages[0].matched, "liberdade");

    // Prefix and Chinese.
    // Both have one occurrence ("melancolia"); equal counts come in no fixed order.
    let mut names: Vec<String> = library
        .search("melancol*")
        .unwrap()
        .files
        .into_iter()
        .map(|f| f.name)
        .collect();
    names.sort();
    assert_eq!(names, ["Declaração.pdf", "Notas.md"]);
    assert_eq!(
        library.search("自由").unwrap().files[0].name,
        "鲁迅 - 呐喊.html"
    );

    // By name, including files whose text could not be read.
    let answer = library.search("senha").unwrap();
    assert!(answer.files.is_empty());
    assert_eq!(answer.by_name[0].name, "Com senha.pdf");
    assert_eq!(answer.by_name[0].problem, Some(Problem::Protected));
    assert_eq!(
        library.search("nuvem").unwrap().by_name[0].name,
        "Na nuvem.epub"
    );

    // The list of files that could not be read.
    let mut problems: Vec<(String, Option<Problem>)> = library
        .problems()
        .unwrap()
        .into_iter()
        .map(|f| (f.name, f.problem))
        .collect();
    problems.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        problems,
        [
            ("Com senha.pdf".to_owned(), Some(Problem::Protected)),
            ("Na nuvem.epub".to_owned(), Some(Problem::InCloud)),
        ]
    );

    // The reading report has a time for every file read.
    let report = library.report(3).unwrap();
    assert_eq!((report.files, report.indexed), (6, 4));
    assert_eq!(report.slowest.len(), 3);

    // Nothing in the library folder was changed: still the same eight files.
    let mut count = 0;
    for entry in walk(folder.path()) {
        assert!(entry.is_file());
        count += 1;
    }
    assert_eq!(count, 8);
}

#[test]
fn indexing_can_stop_and_continue_later() {
    let folder = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    books(folder.path());
    {
        let library = Library::open(folder.path(), store.path()).unwrap();
        library.scan().unwrap();
        let mut seen = 0;
        library
            .index_pending(vec![reader()], |_| {
                seen += 1;
                seen < 2
            })
            .unwrap();
        // Stopped after the second file; the file already being read is kept too.
        let done = library.progress().unwrap().done;
        assert!((2..6).contains(&done), "{done}");
    }
    // Mulu was closed and opened again.
    let library = Library::open(folder.path(), store.path()).unwrap();
    assert_eq!(library.scan().unwrap(), 6);
    library
        .index_pending(vec![reader(), reader()], |_| true)
        .unwrap();
    let progress = library.progress().unwrap();
    assert_eq!((progress.done, progress.total), (6, 6));
    assert_eq!(library.search("liberdade").unwrap().files.len(), 2);
    assert_eq!(library.search("capitu").unwrap().files.len(), 1);
}

fn walk(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(root).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else {
            out.push(path);
        }
    }
    out
}
