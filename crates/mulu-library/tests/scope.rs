//! The scope of a search: folders of a library left out with their checkboxes (D16).

use mulu_library::{Folder, Library};
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

#[test]
fn leaving_folders_out_of_a_search() {
    let folder = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    let root = folder.path();
    write(
        root,
        "Grécia/Homero - Ilíada.txt",
        "Canta, ó deusa, a cólera de Aquiles. Aquiles.",
    );
    write(
        root,
        "Grécia/Tragédia/Sófocles - Édipo.txt",
        "Édipo e Aquiles",
    );
    write(
        root,
        "Grécia antiga/Hesíodo.txt",
        "Aquiles não está aqui; as Musas sim.",
    );
    write(
        root,
        "Roma/Virgílio - Eneida.txt",
        "Canto as armas e o varão. Aquiles",
    );
    write(root, "Aquiles na raiz.txt", "Aquiles");
    let library = Library::open(root, store.path()).unwrap();
    library.refresh().unwrap();
    library.index_pending(vec![reader()], |_| true).unwrap();

    let folders = library.folders().unwrap();
    let f = |path: &str, files: u64| Folder {
        path: path.to_owned(),
        files,
    };
    assert_eq!(
        folders,
        [
            f("Grécia", 2),
            f("Grécia antiga", 1),
            f("Grécia/Tragédia", 1),
            f("Roma", 1)
        ]
    );

    let all = library.search("aquiles").unwrap();
    assert_eq!(all.files.len(), 5);
    assert_eq!(all.total, 6);

    // Each folder counts for its own files only: leaving out "Grécia" and its
    // subfolder leaves "Grécia antiga" in, whose name only begins the same way.
    let out =
        |folders: &[&str]| -> Vec<String> { folders.iter().map(|f| (*f).to_owned()).collect() };
    let some = library
        .search_in("aquiles", &out(&["Grécia", "Grécia/Tragédia"]))
        .unwrap();
    let mut names: Vec<&str> = some.files.iter().map(|f| f.name.as_str()).collect();
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "Aquiles na raiz.txt",
            "Hesíodo.txt",
            "Virgílio - Eneida.txt"
        ]
    );
    assert_eq!(some.total, 3);
    // Files found by their name follow the same scope.
    assert!(some.by_name.iter().all(|f| !f.path.starts_with("Grécia/")));

    // A folder left out while its subfolder stays in.
    let some = library.search_in("aquiles", &out(&["Grécia"])).unwrap();
    assert_eq!(some.files.len(), 4);
    assert!(some.files.iter().any(|f| f.name == "Sófocles - Édipo.txt"));
    assert!(some.files.iter().all(|f| f.name != "Homero - Ilíada.txt"));

    // The top folder: files directly in the library folder.
    let some = library.search_in("aquiles", &out(&[""])).unwrap();
    assert!(some.files.iter().all(|f| f.name != "Aquiles na raiz.txt"));
    assert_eq!(some.files.len(), 4);
}
