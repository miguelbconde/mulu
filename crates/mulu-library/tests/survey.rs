//! The survey before the first reading, and what a library leaves out (instructions,
//! section 1).

use mulu_library::{Library, Skip, survey};
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

fn sample(root: &Path) {
    write(root, "Livros/Homero - Ilíada.txt", "Aquiles");
    write(
        root,
        "Livros/Grécia/Hesíodo.html",
        "<p>Aquiles e as Musas</p>",
    );
    write(root, "Livros/Grécia/Píndaro.md", "Aquiles nos jogos");
    write(
        root,
        "Downloads/manual da impressora.txt",
        "Aquiles não, impressora",
    );
    write(root, "Downloads/recibo.html", "<p>Aquiles?</p>");
    write(root, "Downloads/foto.jpg", "not a book");
}

#[test]
fn counting_before_reading() {
    let folder = tempfile::tempdir().unwrap();
    sample(folder.path());
    let s = survey(folder.path(), &Skip::default());
    assert_eq!(s.files, 5);
    assert_eq!(s.in_cloud, 0);
    assert_eq!(
        s.formats,
        [
            ("html".to_owned(), 2),
            ("txt".to_owned(), 2),
            ("md".to_owned(), 1)
        ]
    );
    let folders: Vec<(&str, u64)> = s
        .folders
        .iter()
        .map(|f| (f.path.as_str(), f.files))
        .collect();
    assert_eq!(
        folders,
        [("Downloads", 2), ("Livros", 3), ("Livros/Grécia", 2)]
    );

    // A folder left out is not counted.
    let skip = Skip {
        formats: Vec::new(),
        folders: vec!["Downloads".to_owned()],
    };
    assert_eq!(survey(folder.path(), &skip).files, 3);
}

#[test]
fn a_library_reads_only_what_was_kept_in_the_survey() {
    let folder = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    sample(folder.path());
    let library = Library::open(folder.path(), store.path()).unwrap();
    library
        .set_skip(&Skip {
            formats: vec!["md".to_owned()],
            folders: vec!["Downloads".to_owned()],
        })
        .unwrap();
    library.refresh().unwrap();
    library.index_pending(vec![reader()], |_| true).unwrap();
    let mut names: Vec<String> = library
        .search("aquiles")
        .unwrap()
        .files
        .into_iter()
        .map(|f| f.name)
        .collect();
    names.sort();
    assert_eq!(names, ["Hesíodo.html", "Homero - Ilíada.txt"]);

    // Letting a folder in again: the next check reads it; leaving one out takes its
    // files out of the index.
    library
        .set_skip(&Skip {
            formats: vec!["md".to_owned()],
            folders: vec!["Livros/Grécia".to_owned()],
        })
        .unwrap();
    let changes = library.refresh().unwrap();
    assert_eq!((changes.added, changes.removed), (2, 1));
    library.index_pending(vec![reader()], |_| true).unwrap();
    let mut names: Vec<String> = library
        .search("aquiles")
        .unwrap()
        .files
        .into_iter()
        .map(|f| f.name)
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "Homero - Ilíada.txt",
            "manual da impressora.txt",
            "recibo.html"
        ]
    );
}

#[test]
fn skip_rules_survive_as_text() {
    let skip = Skip {
        formats: vec!["pdf".to_owned()],
        folders: vec!["Cursos/2019".to_owned(), "Downloads".to_owned()],
    };
    assert_eq!(Skip::from_text(&skip.to_text()), skip);
    assert_eq!(Skip::from_text(""), Skip::default());
}
