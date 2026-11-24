//! Mulu (目录): the desktop application.
//!
//! The window (in `ui/`) asks this program for everything through a few commands:
//! choose the library folder, follow the indexing, search, read passages, list the
//! files that could not be read, open a file. The library itself lives in
//! `mulu-library`; files are read in the separate reading process (`mulu-reader`),
//! which is this same program started with `--read-worker` (D25).

// On Windows, do not open a console window next to the application.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use mulu_library::{
    Answer, Changes, Folder, FoundFile, Library, Passage, Progress, Report, Skip, Survey,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;

/// The file name of the PDF engine's library on this system (D28).
const PDFIUM_FILE: &str = if cfg!(windows) {
    "pdfium.dll"
} else if cfg!(target_os = "macos") {
    "libpdfium.dylib"
} else {
    "libpdfium.so"
};

// Counts the memory used by the reading process, which runs this same program (D25).
// In the window process the allowance is unlimited.
#[global_allocator]
static ALLOCATOR: mulu_reader::MemoryCap = mulu_reader::memory_cap();

/// What Mulu remembers between sessions, in its own data folder.
#[derive(Default, Serialize, Deserialize)]
struct Config {
    /// The libraries ("bases"), in the order they are shown.
    #[serde(default)]
    libraries: Vec<PathBuf>,
    /// Mulu 0.1 and 0.2.0 kept a single library here. Read once, then moved to
    /// `libraries`.
    #[serde(default, skip_serializing)]
    library: Option<PathBuf>,
    /// The reading was paused by the user, and stays paused after a restart (0.2.2).
    #[serde(default)]
    paused: bool,
}

impl Config {
    fn roots(&self) -> Vec<PathBuf> {
        let mut roots = self.libraries.clone();
        if let Some(old) = &self.library
            && !roots.contains(old)
        {
            roots.insert(0, old.clone());
        }
        roots
    }
}

/// The library being read now, and how far.
#[derive(Clone, Serialize)]
struct Reading {
    base: String,
    #[serde(flatten)]
    progress: Progress,
}

#[derive(Default)]
struct App {
    data: PathBuf,
    /// The libraries, in the order shown (0.2.1: several libraries).
    bases: Arc<Mutex<Vec<Arc<Library>>>>,
    /// Libraries waiting for a check for changes and a reading, one at a time.
    queue: Arc<Mutex<VecDeque<Arc<Library>>>>,
    reading: Arc<Mutex<Option<Reading>>>,
    /// How long the last reading took, in seconds.
    last_run: Arc<Mutex<Option<u64>>>,
    /// The PDF engine's library, shipped with Mulu (D28).
    pdfium: Option<PathBuf>,
    /// Read as fast as the computer allows, instead of in the background (D29).
    fast: Arc<AtomicBool>,
    /// Changes when the pace changes, so that the reading starts again with it.
    generation: Arc<AtomicU64>,
    /// Whether the full speed was used during the current or last reading.
    fast_used: Arc<AtomicBool>,
    /// The worker that checks and reads the libraries of the queue is running.
    busy: Arc<AtomicBool>,
    /// What the latest check of each library found (D16), by library folder.
    changes: Arc<Mutex<HashMap<String, Changes>>>,
    /// The user paused the reading: libraries are still checked for changes, but no
    /// file is read until the reading is resumed (0.2.2).
    paused: Arc<AtomicBool>,
}

/// How many reading processes to use: in the background, one for every two
/// processors, at most three, so the computer stays usable (D27); at full speed, one
/// per physical core, at most eight (D30).
fn reading_processes(fast: bool) -> usize {
    let logical = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    let background = (logical / 2).clamp(1, 3);
    if fast {
        let cores = mulu_reader::physical_cores().unwrap_or(logical / 2);
        cores.clamp(1, 8).max(background)
    } else {
        background
    }
}

/// One library as the window shows it.
#[derive(Serialize)]
struct BaseStatus {
    root: String,
    /// The folder is there (an external disk may be unplugged).
    present: bool,
    /// Files in the library, and those already read (with or without text).
    files: u64,
    done: u64,
    problems: usize,
    /// When the library was last checked for changes, in seconds since 1970.
    checked: Option<i64>,
    changes: Option<Changes>,
}

#[derive(Serialize)]
struct Status {
    bases: Vec<BaseStatus>,
    /// The library being read now, if any.
    reading: Option<Reading>,
    last_run: Option<u64>,
    fast: bool,
    paused: bool,
}

type CommandResult<T> = Result<T, String>;

fn text_error(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn root_text(library: &Library) -> String {
    library.root().display().to_string()
}

impl App {
    fn bases(&self) -> Vec<Arc<Library>> {
        self.bases.lock().map(|b| b.clone()).unwrap_or_default()
    }

    /// The library whose folder is `root`, as the window names it.
    fn base(&self, root: &str) -> CommandResult<Arc<Library>> {
        self.bases()
            .into_iter()
            .find(|l| root_text(l) == root)
            .ok_or_else(|| "unknown library".to_owned())
    }

    fn config_path(&self) -> PathBuf {
        self.data.join("config.json")
    }

    fn load_config(&self) -> Config {
        std::fs::read_to_string(self.config_path())
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    fn save_config(&self) -> std::io::Result<()> {
        let config = Config {
            libraries: self
                .bases()
                .iter()
                .map(|l| l.root().to_path_buf())
                .collect(),
            library: None,
            paused: self.paused.load(Ordering::SeqCst),
        };
        std::fs::create_dir_all(&self.data)?;
        let text = serde_json::to_string_pretty(&config).map_err(std::io::Error::other)?;
        std::fs::write(self.config_path(), text)
    }

    /// Each library folder has its own catalog and index, in Mulu's data folder.
    fn store(&self, root: &Path) -> PathBuf {
        self.data
            .join(format!("library-{:016x}", fingerprint(root)))
    }

    /// Removes the catalogs and indexes of libraries that are no longer in the list
    /// (they are Mulu's own data; the library folders are never touched). Done at start,
    /// when no library is open, so no file of them is in use.
    fn remove_stale_stores(&self) {
        let keep: Vec<PathBuf> = self.bases().iter().map(|l| self.store(l.root())).collect();
        let Ok(entries) = std::fs::read_dir(&self.data) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let is_store = entry
                .file_name()
                .to_str()
                .is_some_and(|n| n.starts_with("library-"));
            if is_store && path.is_dir() && !keep.contains(&path) {
                let _ = std::fs::remove_dir_all(&path);
            }
        }
    }

    /// Puts libraries in the queue for a check for changes and a reading of what is new
    /// or changed (D16, D31), and starts the worker if it is not running. Libraries are
    /// handled one at a time; searches work all the while.
    fn run(&self, libraries: Vec<Arc<Library>>) {
        if let Ok(mut queue) = self.queue.lock() {
            for library in libraries {
                if !queue.iter().any(|q| Arc::ptr_eq(q, &library)) {
                    queue.push_back(library);
                }
            }
        }
        if self
            .busy
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return;
        }
        let queue = Arc::clone(&self.queue);
        let bases = Arc::clone(&self.bases);
        let reading = Arc::clone(&self.reading);
        let last_run = Arc::clone(&self.last_run);
        let fast = Arc::clone(&self.fast);
        let generation = Arc::clone(&self.generation);
        let fast_used = Arc::clone(&self.fast_used);
        let busy = Arc::clone(&self.busy);
        let changes = Arc::clone(&self.changes);
        let paused = Arc::clone(&self.paused);
        fast_used.store(self.fast.load(Ordering::SeqCst), Ordering::SeqCst);
        let settings = mulu_reader::Settings {
            pdfium: self.pdfium.clone(),
            ..mulu_reader::Settings::default()
        };
        std::thread::spawn(move || {
            let set = |r: Option<Reading>| {
                if let Ok(mut slot) = reading.lock() {
                    *slot = r;
                }
            };
            let still_listed = |library: &Arc<Library>| {
                bases
                    .lock()
                    .is_ok_and(|b| b.iter().any(|l| Arc::ptr_eq(l, library)))
            };
            loop {
                let next = queue.lock().ok().and_then(|mut q| q.pop_front());
                let Some(library) = next else {
                    // Checked under the lock of the queue: a library queued after this
                    // point starts a new worker.
                    let queue = queue.lock();
                    if queue.as_ref().is_ok_and(|q| q.is_empty()) {
                        busy.store(false, Ordering::SeqCst);
                        set(None);
                        return;
                    }
                    continue;
                };
                if !still_listed(&library) {
                    continue;
                }
                let base = root_text(&library);
                set(Some(Reading {
                    base: base.clone(),
                    progress: Progress {
                        done: 0,
                        total: 0,
                        current: None,
                    },
                }));
                let started = std::time::Instant::now();
                match library.refresh() {
                    Ok(found) => {
                        if let Ok(mut slot) = changes.lock() {
                            slot.insert(base.clone(), found);
                        }
                    }
                    Err(_) => continue,
                }
                let Ok(program) = std::env::current_exe() else {
                    continue;
                };
                let mut read_any = false;
                // Paused: the library was checked, and its new files wait.
                if paused.load(Ordering::SeqCst) {
                    continue;
                }
                // The pace can change while the library is read (D29): the reading
                // stops after the files in hand and starts again with the new pace.
                // A library removed from the list stops being read.
                loop {
                    let run = generation.load(Ordering::SeqCst);
                    let fast = fast.load(Ordering::SeqCst);
                    if fast {
                        fast_used.store(true, Ordering::SeqCst);
                    }
                    let settings = mulu_reader::Settings {
                        low_priority: !fast,
                        ..settings.clone()
                    };
                    let readers = (0..reading_processes(fast))
                        .map(|_| mulu_reader::Reader::new(&program, settings.clone()))
                        .collect();
                    let _ = library.index_pending(readers, |p| {
                        read_any |= p.current.is_some();
                        set(Some(Reading {
                            base: base.clone(),
                            progress: p.clone(),
                        }));
                        generation.load(Ordering::SeqCst) == run
                            && still_listed(&library)
                            && !paused.load(Ordering::SeqCst)
                    });
                    if generation.load(Ordering::SeqCst) == run || !still_listed(&library) {
                        break;
                    }
                }
                // The reading time of the report is that of a reading, not of a check
                // that found nothing to read.
                if read_any && let Ok(mut slot) = last_run.lock() {
                    *slot = Some(started.elapsed().as_secs());
                }
            }
        });
    }
}

/// A stable number for a folder path (FNV-1a), to name its data folder.
fn fingerprint(path: &Path) -> u64 {
    path.to_string_lossy()
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}

#[tauri::command]
fn status(app: State<'_, App>) -> CommandResult<Status> {
    let changes = app.changes.lock().map(|c| c.clone()).unwrap_or_default();
    let mut bases = Vec::new();
    for library in app.bases() {
        let root = root_text(&library);
        let progress = library.progress().map_err(text_error)?;
        bases.push(BaseStatus {
            files: progress.total,
            done: progress.done,
            problems: library.problems().map_err(text_error)?.len(),
            checked: library.checked().map_err(text_error)?,
            changes: changes.get(&root).cloned(),
            present: library.root().is_dir(),
            root,
        });
    }
    Ok(Status {
        bases,
        reading: app.reading.lock().ok().and_then(|r| r.clone()),
        last_run: app.last_run.lock().ok().and_then(|t| *t),
        fast: app.fast.load(Ordering::SeqCst),
        paused: app.paused.load(Ordering::SeqCst),
    })
}

/// Why a folder cannot become a library: it is a library already, or it is inside
/// one (its books would be counted twice; the scope tree already narrows a search to
/// it). The window explains it in the user's language.
#[derive(Serialize)]
#[serde(tag = "kind", content = "library", rename_all = "lowercase")]
enum Overlap {
    Inside(String),
    Same(String),
}

/// A folder about to become a library, as the survey shows it before the first
/// reading (instructions, section 1).
#[derive(Serialize)]
struct Location {
    root: String,
    survey: Survey,
    /// Libraries inside this folder: they stay libraries of their own and are left
    /// out of this one, so no book is counted twice.
    holds: Vec<String>,
}

/// Checks that `root` can become a library and counts its books.
fn locate(bases: &[Arc<Library>], root: &Path) -> Result<Location, Overlap> {
    let mut holds = Vec::new();
    let mut skip = Skip::default();
    for library in bases {
        let other = library.root();
        let name = other.display().to_string();
        if other == root {
            return Err(Overlap::Same(name));
        }
        if root.starts_with(other) {
            return Err(Overlap::Inside(name));
        }
        if let Ok(inside) = other.strip_prefix(root) {
            skip.folders.push(
                inside
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/"),
            );
            holds.push(name);
        }
    }
    Ok(Location {
        root: root.display().to_string(),
        survey: mulu_library::survey(root, &skip),
        holds,
    })
}

/// Asks for a folder and counts its books, without reading them yet: the window then
/// shows the survey, where formats and folders can be left out.
// Asynchronous, so that the folder dialog and the count do not block the window.
#[tauri::command]
async fn survey_folder(
    handle: tauri::AppHandle,
    app: State<'_, App>,
) -> CommandResult<Result<Option<Location>, Overlap>> {
    let Some(choice) = handle.dialog().file().blocking_pick_folder() else {
        return Ok(Ok(None));
    };
    let root = choice.into_path().map_err(text_error)?;
    let bases = app.bases();
    tauri::async_runtime::spawn_blocking(move || locate(&bases, &root).map(Some))
        .await
        .map_err(text_error)
}

/// The places "whole computer" means (instructions, section 1): the user's home folder
/// and the other disks. The system, programs, hidden and temporary folders are left
/// out by the walk itself.
fn computer_locations() -> Vec<PathBuf> {
    let mut places = Vec::new();
    #[cfg(windows)]
    {
        if let Some(home) = std::env::var_os("USERPROFILE") {
            places.push(PathBuf::from(home));
        }
        let system = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_owned());
        for letter in b'A'..=b'Z' {
            let drive = format!("{}:", char::from(letter));
            if drive.eq_ignore_ascii_case(&system) {
                continue;
            }
            let root = PathBuf::from(format!("{drive}\\"));
            if root.is_dir() {
                places.push(root);
            }
        }
    }
    #[cfg(not(windows))]
    {
        if let Some(home) = std::env::var_os("HOME") {
            places.push(PathBuf::from(home));
        }
        // Other disks: /Volumes on macOS (without the link to the system disk), and
        // the usual mount points on Linux.
        let user = std::env::var("USER").unwrap_or_default();
        for parent in [
            "/Volumes".to_owned(),
            format!("/media/{user}"),
            "/mnt".to_owned(),
        ] {
            let Ok(entries) = std::fs::read_dir(&parent) else {
                continue;
            };
            for entry in entries.flatten() {
                let is_link = entry.file_type().is_ok_and(|t| t.is_symlink());
                if !is_link && entry.path().is_dir() {
                    places.push(entry.path());
                }
            }
        }
    }
    places
}

/// Counts the books of the whole computer, place by place, without reading them yet.
/// Places that are already libraries, or inside one, are left out.
#[tauri::command]
async fn survey_computer(app: State<'_, App>) -> CommandResult<Vec<Location>> {
    let bases = app.bases();
    tauri::async_runtime::spawn_blocking(move || {
        computer_locations()
            .iter()
            .filter_map(|place| locate(&bases, place).ok())
            .collect()
    })
    .await
    .map_err(text_error)
}

/// What the survey keeps out of one new library.
#[derive(Deserialize)]
struct NewLibrary {
    root: String,
    formats: Vec<String>,
    folders: Vec<String>,
}

/// Creates the libraries chosen in the survey and starts reading them, one after the
/// other. The libraries a new one holds are left out of it (see [`locate`]).
#[tauri::command]
fn create_libraries(app: State<'_, App>, libraries: Vec<NewLibrary>) -> CommandResult<()> {
    for new in libraries {
        let root = PathBuf::from(&new.root);
        let mut folders = new.folders;
        for library in app.bases() {
            if library.root() == root || root.starts_with(library.root()) {
                return Err("overlap".to_owned());
            }
            if let Ok(inside) = library.root().strip_prefix(&root) {
                folders.push(
                    inside
                        .components()
                        .map(|c| c.as_os_str().to_string_lossy())
                        .collect::<Vec<_>>()
                        .join("/"),
                );
            }
        }
        let library = Arc::new(Library::open(&root, &app.store(&root)).map_err(text_error)?);
        library
            .set_skip(&Skip {
                formats: new.formats,
                folders,
            })
            .map_err(text_error)?;
        if let Ok(mut bases) = app.bases.lock() {
            bases.push(Arc::clone(&library));
        }
        // Adding a library asks for it to be read: a paused reading resumes.
        app.paused.store(false, Ordering::SeqCst);
        app.run(vec![library]);
    }
    app.save_config().map_err(text_error)
}

/// Takes a library out of Mulu. Its folder and files are not touched; Mulu's own
/// catalog and index of it are deleted the next time Mulu starts.
#[tauri::command]
fn remove_library(app: State<'_, App>, base: String) -> CommandResult<()> {
    let library = app.base(&base)?;
    if let Ok(mut bases) = app.bases.lock() {
        bases.retain(|l| !Arc::ptr_eq(l, &library));
    }
    if let Ok(mut changes) = app.changes.lock() {
        changes.remove(&base);
    }
    app.save_config().map_err(text_error)
}

/// A file of the results, with the library it belongs to.
#[derive(Serialize)]
struct BaseFile {
    base: String,
    #[serde(flatten)]
    file: FoundFile,
}

/// The answer to a search in all the libraries.
#[derive(Serialize)]
struct AllAnswer {
    query: String,
    files: Vec<BaseFile>,
    by_name: Vec<BaseFile>,
    total: u64,
    elapsed_ms: u64,
}

/// The folders left out of the search in one library (unticked in its tree).
#[derive(Deserialize)]
struct Scope {
    base: String,
    folders: Vec<String>,
}

/// Searches every library, leaving out the files directly in the unticked folders,
/// and puts the results together: by number of occurrences, then in the order of
/// the libraries.
#[tauri::command]
fn search(
    app: State<'_, App>,
    query: String,
    excluded: Option<Vec<Scope>>,
) -> CommandResult<Option<AllAnswer>> {
    let bases = app.bases();
    if bases.is_empty() {
        return Ok(None);
    }
    let excluded = excluded.unwrap_or_default();
    let started = std::time::Instant::now();
    let mut all = AllAnswer {
        query: query.clone(),
        files: Vec::new(),
        by_name: Vec::new(),
        total: 0,
        elapsed_ms: 0,
    };
    for library in bases {
        let base = root_text(&library);
        let out: &[String] = excluded
            .iter()
            .find(|s| s.base == base)
            .map_or(&[], |s| s.folders.as_slice());
        let answer: Answer = library.search_in(&query, out).map_err(text_error)?;
        all.total += answer.total;
        let tag = |file| BaseFile {
            base: base.clone(),
            file,
        };
        all.files.extend(answer.files.into_iter().map(tag));
        all.by_name.extend(answer.by_name.into_iter().map(tag));
    }
    // Stable: files with the same count keep the order each library gave them.
    all.files.sort_by_key(|f| std::cmp::Reverse(f.file.count));
    all.elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    Ok(Some(all))
}

/// The folders of a library with book files, for its scope tree.
#[tauri::command]
fn folders(app: State<'_, App>, base: String) -> CommandResult<Vec<Folder>> {
    app.base(&base)?.folders().map_err(text_error)
}

/// The refresh button: checks a library (or, without `base`, all of them) for
/// changes now (D16). A library already waiting is not queued twice.
#[tauri::command]
fn refresh(app: State<'_, App>, base: Option<String>) -> CommandResult<()> {
    let libraries = match base {
        Some(base) => vec![app.base(&base)?],
        None => app.bases(),
    };
    // The refresh button asks for the changes to be read: a paused reading resumes.
    if app.paused.swap(false, Ordering::SeqCst) {
        app.save_config().map_err(text_error)?;
    }
    app.run(libraries);
    Ok(())
}

/// Pauses the reading (`true`) or resumes it (`false`). The reading stops after the
/// files in hand; read files stay in the index, and the rest are read on resuming,
/// even after Mulu is closed and opened again (0.2.2).
#[tauri::command]
fn set_paused(app: State<'_, App>, paused: bool) -> CommandResult<()> {
    app.paused.store(paused, Ordering::SeqCst);
    app.save_config().map_err(text_error)?;
    if !paused {
        let present: Vec<Arc<Library>> = app
            .bases()
            .into_iter()
            .filter(|l| l.root().is_dir())
            .collect();
        app.run(present);
    }
    Ok(())
}

#[tauri::command]
fn passages(
    app: State<'_, App>,
    base: String,
    query: String,
    file: u64,
    skip: usize,
    limit: usize,
) -> CommandResult<Vec<Passage>> {
    app.base(&base)?
        .passages(&query, file, skip, limit.min(200))
        .map_err(text_error)
}

/// The files that could not be read, in every library.
#[tauri::command]
fn problems(app: State<'_, App>) -> CommandResult<Vec<BaseFile>> {
    let mut all = Vec::new();
    for library in app.bases() {
        let base = root_text(&library);
        all.extend(
            library
                .problems()
                .map_err(text_error)?
                .into_iter()
                .map(|file| BaseFile {
                    base: base.clone(),
                    file,
                }),
        );
    }
    Ok(all)
}

/// Reads the library at full speed (`true`) or in the background (`false`) (D29).
#[tauri::command]
fn set_fast(app: State<'_, App>, fast: bool) {
    if app.fast.swap(fast, Ordering::SeqCst) != fast {
        app.generation.fetch_add(1, Ordering::SeqCst);
    }
}

/// Figures about the reading of each library, for the report the user can copy and
/// send (0.1.1).
#[tauri::command]
fn report(app: State<'_, App>) -> CommandResult<Option<ReadingReport>> {
    let bases = app.bases();
    if bases.is_empty() {
        return Ok(None);
    }
    let mut figures = Vec::new();
    for library in bases {
        figures.push(BaseReport {
            base: root_text(&library),
            figures: library.report(25).map_err(text_error)?,
        });
    }
    Ok(Some(ReadingReport {
        bases: figures,
        last_run: app.last_run.lock().ok().and_then(|t| *t),
        processors: std::thread::available_parallelism().map_or(1, std::num::NonZero::get),
        cores: mulu_reader::physical_cores(),
        background: reading_processes(false),
        full_speed: reading_processes(true),
        full_speed_used: app.fast_used.load(Ordering::SeqCst),
    }))
}

#[derive(Serialize)]
struct BaseReport {
    base: String,
    #[serde(flatten)]
    figures: Report,
}

/// The reading report, with how many files were read at once (D30).
#[derive(Serialize)]
struct ReadingReport {
    bases: Vec<BaseReport>,
    last_run: Option<u64>,
    processors: usize,
    cores: Option<usize>,
    background: usize,
    full_speed: usize,
    full_speed_used: bool,
}

/// Opens a file of the library with the program the system uses for it, or shows it in
/// its folder. Only files listed in the catalog can be opened.
#[tauri::command]
fn open_file(app: State<'_, App>, base: String, file: u64, reveal: bool) -> CommandResult<()> {
    let path = app
        .base(&base)?
        .path_of(file)
        .map_err(text_error)?
        .ok_or("unknown file")?;
    let mut command = system_open(&path, reveal);
    command.spawn().map(|_| ()).map_err(text_error)
}

fn system_open(path: &Path, reveal: bool) -> std::process::Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let mut command = std::process::Command::new("explorer.exe");
        if reveal {
            // Explorer wants `/select,"C:\path with spaces\file.pdf"`: the quotes only
            // around the path. Passed as one ordinary argument, the whole thing would be
            // quoted, and Explorer would open the Documents folder instead (0.1.0 bug).
            let mut arg = std::ffi::OsString::from("/select,\"");
            arg.push(path);
            arg.push("\"");
            command.raw_arg(arg);
        } else {
            command.arg(path);
        }
        command
    }
    #[cfg(target_os = "macos")]
    {
        let mut command = std::process::Command::new("open");
        if reveal {
            command.arg("-R");
        }
        command.arg(path);
        command
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let mut command = std::process::Command::new("xdg-open");
        command.arg(if reveal {
            path.parent().unwrap_or(path)
        } else {
            path
        });
        command
    }
}

fn main() {
    // Started with `--read-worker`, this program is Mulu's reading process: it reads
    // files for the window process and never opens a window (D25).
    if std::env::args().nth(1).as_deref() == Some(mulu_reader::WORKER_FLAG) {
        let settings = mulu_reader::Settings::from_args(std::env::args().skip(2));
        mulu_reader::serve(&ALLOCATOR, settings);
        return;
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data = app.path().app_data_dir()?;
            let pdfium = app
                .path()
                .resource_dir()
                .ok()
                .map(|dir| dir.join("pdfium").join(PDFIUM_FILE))
                .filter(|path| path.is_file());
            let state = App {
                data,
                pdfium,
                ..App::default()
            };
            // Reopen the libraries chosen before and check them for changes, one after
            // the other. A library whose folder is missing (an external disk not
            // plugged in) is kept in the list but not checked: checking it would take
            // all its files out of the index.
            let config = state.load_config();
            state.paused.store(config.paused, Ordering::SeqCst);
            for root in config.roots() {
                if let Ok(library) = Library::open(&root, &state.store(&root))
                    && let Ok(mut bases) = state.bases.lock()
                {
                    bases.push(Arc::new(library));
                }
            }
            state.remove_stale_stores();
            if config.library.is_some() {
                let _ = state.save_config();
            }
            let present: Vec<Arc<Library>> = state
                .bases()
                .into_iter()
                .filter(|l| l.root().is_dir())
                .collect();
            state.run(present);
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            status,
            survey_folder,
            survey_computer,
            create_libraries,
            remove_library,
            search,
            folders,
            refresh,
            passages,
            problems,
            report,
            set_fast,
            set_paused,
            open_file
        ])
        .run(tauri::generate_context!())
        .expect("Mulu could not start");
}

#[cfg(test)]
mod tests {
    /// A folder inside a library, or a library itself, cannot become a library; a
    /// folder that holds a library can, leaving that library out (0.2.1).
    #[test]
    fn libraries_never_overlap() {
        use super::{Library, Overlap, locate};
        use std::sync::Arc;
        let top = std::env::temp_dir().join(format!("mulu-overlap-{}", std::process::id()));
        let home = top.join("home");
        let store = top.join("store");
        for (path, text) in [
            ("home/Documentos/Biblioteca/Homero.txt", "Aquiles"),
            ("home/Documentos/Cursos/Programa.txt", "Aquiles"),
            ("Outros/Hesíodo.txt", "Musas"),
        ] {
            let path = top.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        let bases = vec![Arc::new(Library::open(&home, &store).unwrap())];

        assert!(matches!(locate(&bases, &home), Err(Overlap::Same(_))));
        assert!(matches!(
            locate(&bases, &home.join("Documentos")),
            Err(Overlap::Inside(_))
        ));
        let Ok(outer) = locate(&bases, &top) else {
            panic!("a folder holding a library can become one");
        };
        assert_eq!(outer.holds, [home.display().to_string()]);
        // The books of the library inside are not counted again.
        assert_eq!(outer.survey.files, 1);
        let _ = std::fs::remove_dir_all(&top);
    }

    /// The version shown to the user, the installer version and the package version
    /// must always be the same.
    #[test]
    fn versions_agree() {
        let version = env!("CARGO_PKG_VERSION");
        let config = include_str!("../tauri.conf.json");
        assert!(
            config.contains(&format!("\"version\": \"{version}\"")),
            "tauri.conf.json"
        );
        let page = include_str!("../../../ui/index.html");
        assert!(
            page.contains(&format!("data-version=\"{version}\"")),
            "ui/index.html"
        );
    }

    /// Mulu makes no network connections (privacy principle): the page may only load
    /// its own files and talk to the application itself.
    #[test]
    fn content_security_policy_is_local_only() {
        let config = include_str!("../tauri.conf.json");
        let csp = config
            .lines()
            .find(|line| line.trim_start().starts_with("\"csp\""))
            .expect("a content security policy is set");
        assert!(csp.contains("default-src 'self'"));
        let without_ipc = csp.replace("http://ipc.localhost", "");
        for remote in ["http:", "https:", "ws:", "wss:", "*"] {
            assert!(
                !without_ipc.contains(remote),
                "the policy must not allow {remote}"
            );
        }
    }
}
