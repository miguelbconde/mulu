//! Mulu (目录): the desktop application.
//!
//! The window (in `ui/`) asks this program for everything through a few commands:
//! choose the library folder, follow the indexing, search, read passages, list the
//! files that could not be read, open a file. The library itself lives in
//! `mulu-library`; files are read in the separate reading process (`mulu-reader`),
//! which is this same program started with `--read-worker` (D25).

// On Windows, do not open a console window next to the application.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use mulu_library::{Answer, FoundFile, Library, Passage, Progress, Report};
use serde::{Deserialize, Serialize};
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
    library: Option<PathBuf>,
}

#[derive(Default)]
struct App {
    data: PathBuf,
    library: Mutex<Option<Arc<Library>>>,
    progress: Arc<Mutex<Option<Progress>>>,
    /// How long the last reading of the library took, in seconds.
    last_run: Arc<Mutex<Option<u64>>>,
    /// The PDF engine's library, shipped with Mulu (D28).
    pdfium: Option<PathBuf>,
    /// Read as fast as the computer allows, instead of in the background (D29).
    fast: Arc<AtomicBool>,
    /// Changes when the pace changes, so that the reading starts again with it.
    generation: Arc<AtomicU64>,
    /// Whether the full speed was used during the current or last reading.
    fast_used: Arc<AtomicBool>,
}

/// How many files are read at once. In the background (the default), at most half the
/// processors and three files (D27); at full speed, one per physical core, at most
/// eight, at normal priority (D29, D30).
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

#[derive(Serialize)]
struct Status {
    library: Option<String>,
    progress: Option<Progress>,
    /// Files in the library.
    files: u64,
    problems: usize,
    last_run: Option<u64>,
    fast: bool,
}

type CommandResult<T> = Result<T, String>;

fn text_error(e: impl std::fmt::Display) -> String {
    e.to_string()
}

impl App {
    fn library(&self) -> Option<Arc<Library>> {
        self.library.lock().ok().and_then(|l| l.clone())
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

    fn save_config(&self, config: &Config) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.data)?;
        let text = serde_json::to_string_pretty(config).map_err(std::io::Error::other)?;
        std::fs::write(self.config_path(), text)
    }

    /// Opens the library of `root` and starts finding and reading its files in the
    /// background. Searches work while it runs.
    fn start(&self, root: &Path) -> Result<(), String> {
        // Each library folder has its own catalog and index.
        let store = self
            .data
            .join(format!("library-{:016x}", fingerprint(root)));
        let library = Arc::new(Library::open(root, &store).map_err(text_error)?);
        if let Ok(mut slot) = self.library.lock() {
            *slot = Some(Arc::clone(&library));
        }
        let progress = Arc::clone(&self.progress);
        let last_run = Arc::clone(&self.last_run);
        let fast = Arc::clone(&self.fast);
        let generation = Arc::clone(&self.generation);
        let fast_used = Arc::clone(&self.fast_used);
        fast_used.store(self.fast.load(Ordering::SeqCst), Ordering::SeqCst);
        let settings = mulu_reader::Settings {
            pdfium: self.pdfium.clone(),
            ..mulu_reader::Settings::default()
        };
        std::thread::spawn(move || {
            let started = std::time::Instant::now();
            let set = |p: Option<Progress>| {
                if let Ok(mut slot) = progress.lock() {
                    *slot = p;
                }
            };
            set(Some(Progress {
                done: 0,
                total: 0,
                current: None,
            }));
            if library.scan().is_err() {
                set(None);
                return;
            }
            let Ok(program) = std::env::current_exe() else {
                set(None);
                return;
            };
            // The pace can change while the library is read (D29): the reading stops
            // after the files in hand and starts again with the new pace.
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
                    set(Some(p.clone()));
                    generation.load(Ordering::SeqCst) == run
                });
                if generation.load(Ordering::SeqCst) == run {
                    break;
                }
            }
            if let Ok(mut slot) = last_run.lock() {
                *slot = Some(started.elapsed().as_secs());
            }
            set(None);
        });
        Ok(())
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
    let library = app.library();
    let (files, problems) = match &library {
        Some(l) => (
            l.progress().map_err(text_error)?.total,
            l.problems().map_err(text_error)?.len(),
        ),
        None => (0, 0),
    };
    Ok(Status {
        library: library.map(|l| l.root().display().to_string()),
        progress: app.progress.lock().ok().and_then(|p| p.clone()),
        files,
        problems,
        last_run: app.last_run.lock().ok().and_then(|t| *t),
        fast: app.fast.load(Ordering::SeqCst),
    })
}

// Asynchronous, so that the folder dialog does not block the window while it is open.
#[tauri::command]
async fn choose_folder(
    handle: tauri::AppHandle,
    app: State<'_, App>,
) -> CommandResult<Option<String>> {
    let Some(choice) = handle.dialog().file().blocking_pick_folder() else {
        return Ok(None);
    };
    let root = choice.into_path().map_err(text_error)?;
    app.start(&root)?;
    app.save_config(&Config {
        library: Some(root.clone()),
    })
    .map_err(text_error)?;
    Ok(Some(root.display().to_string()))
}

#[tauri::command]
fn search(app: State<'_, App>, query: String) -> CommandResult<Option<Answer>> {
    match app.library() {
        Some(library) => library.search(&query).map(Some).map_err(text_error),
        None => Ok(None),
    }
}

#[tauri::command]
fn passages(
    app: State<'_, App>,
    query: String,
    file: u64,
    skip: usize,
    limit: usize,
) -> CommandResult<Vec<Passage>> {
    match app.library() {
        Some(library) => library
            .passages(&query, file, skip, limit.min(200))
            .map_err(text_error),
        None => Ok(Vec::new()),
    }
}

#[tauri::command]
fn problems(app: State<'_, App>) -> CommandResult<Vec<FoundFile>> {
    match app.library() {
        Some(library) => library.problems().map_err(text_error),
        None => Ok(Vec::new()),
    }
}

/// Reads the library at full speed (`true`) or in the background (`false`) (D29).
#[tauri::command]
fn set_fast(app: State<'_, App>, fast: bool) {
    if app.fast.swap(fast, Ordering::SeqCst) != fast {
        app.generation.fetch_add(1, Ordering::SeqCst);
    }
}

/// Figures about the reading, for the report the user can copy and send (0.1.1).
#[tauri::command]
fn report(app: State<'_, App>) -> CommandResult<Option<ReadingReport>> {
    let Some(library) = app.library() else {
        return Ok(None);
    };
    Ok(Some(ReadingReport {
        figures: library.report(25).map_err(text_error)?,
        processors: std::thread::available_parallelism().map_or(1, std::num::NonZero::get),
        cores: mulu_reader::physical_cores(),
        background: reading_processes(false),
        full_speed: reading_processes(true),
        full_speed_used: app.fast_used.load(Ordering::SeqCst),
    }))
}

/// The reading report, with how many files were read at once (D30).
#[derive(Serialize)]
struct ReadingReport {
    #[serde(flatten)]
    figures: Report,
    processors: usize,
    cores: Option<usize>,
    background: usize,
    full_speed: usize,
    full_speed_used: bool,
}

/// Opens a file of the library with the program the system uses for it, or shows it in
/// its folder. Only files listed in the catalog can be opened.
#[tauri::command]
fn open_file(app: State<'_, App>, file: u64, reveal: bool) -> CommandResult<()> {
    let library = app.library().ok_or("no library")?;
    let path = library
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
            // Reopen the library chosen before, and look for new files in it.
            if let Some(root) = state.load_config().library
                && root.is_dir()
            {
                let _ = state.start(&root);
            }
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            status,
            choose_folder,
            search,
            passages,
            problems,
            report,
            set_fast,
            open_file
        ])
        .run(tauri::generate_context!())
        .expect("Mulu could not start");
}

#[cfg(test)]
mod tests {
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
