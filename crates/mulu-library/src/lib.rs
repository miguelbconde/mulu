//! # mulu-library
//!
//! One library: a folder of books chosen by the user, its catalog (a SQLite file listing
//! every book file found and what happened when it was read) and its search index.
//!
//! - [`Library::scan`] walks the folder and records the book files, without reading them.
//! - [`Library::index_pending`] reads the files not yet read, one by one, in the
//!   separate reading process (D25), and adds their text to the index. Searches work
//!   while it runs: the index is saved every few files.
//! - [`Library::search`] answers a search: files with occurrences in their text, most
//!   occurrences first, and files whose *name* matches, including files whose text
//!   could not be read (D21).
//!
//! Mulu only reads the books: nothing in the chosen folder is ever written, moved or
//! deleted. Files stored in the cloud and not downloaded are listed, never read, so the
//! cloud service is never asked to download a whole library.

mod catalog;
mod names;
mod walk;

use catalog::{Catalog, Outcome};
pub use catalog::{FileEntry, Status};
use mulu_extract::{Place, Problem};
use mulu_index::{FileResult, LibraryIndex, Page, Results};
use mulu_reader::Reader;
use serde::Serialize;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

/// A file read by a reading process, on its way to the index.
struct Read {
    entry: FileEntry,
    outcome: std::result::Result<mulu_extract::Document, Problem>,
    read_ms: u64,
}

fn millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

/// Memory the index may use while adding text, in megabytes.
const WRITER_MEMORY_MB: usize = 64;

/// The index is saved, and new text becomes searchable, at least this often.
const SAVE_EVERY: Duration = Duration::from_secs(3);

pub struct Library {
    root: PathBuf,
    catalog: Mutex<Catalog>,
    index: LibraryIndex,
    last: Mutex<Option<(String, Results)>>,
}

/// Progress of the indexing, for the progress bar.
#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    /// Files read so far (with or without text), and files found in all.
    pub done: u64,
    pub total: u64,
    /// The file being read now.
    pub current: Option<String>,
}

/// One file in the results of a search.
#[derive(Debug, Clone, Serialize)]
pub struct FoundFile {
    pub id: u64,
    pub name: String,
    /// The folder of the file, relative to the library folder ("" for the top).
    pub folder: String,
    pub path: String,
    /// Occurrences in the text; 0 for a file found only by its name.
    pub count: u32,
    /// Why the text of this file could not be read, if it could not.
    pub problem: Option<Problem>,
}

/// The answer to a search.
#[derive(Debug, Clone, Serialize)]
pub struct Answer {
    pub query: String,
    pub files: Vec<FoundFile>,
    /// Files whose name matches but whose text does not (or could not be read).
    pub by_name: Vec<FoundFile>,
    pub total: u64,
    pub elapsed_ms: u64,
}

/// Figures about the reading of the library, to find what makes it slow.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub files: u64,
    pub indexed: u64,
    pub megabytes: f64,
    /// Time spent in the reading processes, and adding text to the index, all files
    /// together (reading processes work in parallel, so the wall time is shorter).
    pub read_ms: u64,
    pub index_ms: u64,
    pub slowest: Vec<SlowFile>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlowFile {
    pub path: String,
    pub megabytes: f64,
    pub read_ms: u64,
    pub index_ms: u64,
    pub sections: u32,
    pub problem: Option<Problem>,
}

/// One passage around an occurrence.
#[derive(Debug, Clone, Serialize)]
pub struct Passage {
    /// Where it was found, as stored by the reader: "p:12" (page 12, or its printed
    /// number), "s:Chapter title", "n:3" (third part), or "" (the whole file).
    pub place: String,
    pub before: String,
    pub matched: String,
    pub after: String,
}

impl Library {
    /// Opens the library of `root`, keeping its catalog and index in `store` (a folder
    /// of Mulu's own, never inside the library).
    pub fn open(root: &Path, store: &Path) -> Result<Library> {
        std::fs::create_dir_all(store)?;
        let catalog = Catalog::open(&store.join("catalog.sqlite"))?;
        let index = LibraryIndex::open(&store.join("index"))?;
        Ok(Library {
            root: root.to_owned(),
            catalog: Mutex::new(catalog),
            index,
            last: Mutex::new(None),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn catalog(&self) -> std::sync::MutexGuard<'_, Catalog> {
        self.catalog
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Finds the book files of the library folder and records the new ones. Files no
    /// longer present are left for version 0.2 (incremental updates).
    pub fn scan(&self) -> Result<u64> {
        let found = walk::book_files(&self.root);
        let mut catalog = self.catalog();
        catalog.add_new(&self.root, &found)?;
        catalog.count()
    }

    pub fn progress(&self) -> Result<Progress> {
        let catalog = self.catalog();
        Ok(Progress {
            done: catalog.count_done()?,
            total: catalog.count()?,
            current: None,
        })
    }

    /// Reads every file not read yet and adds its text to the index, with one reading
    /// process per reader given (files are read in parallel; the text is added to the
    /// index one file at a time). `report` is called after each file; it returns
    /// `false` to stop (the remaining files stay pending and are read the next time).
    pub fn index_pending(
        &self,
        readers: Vec<Reader>,
        mut report: impl FnMut(&Progress) -> bool,
    ) -> Result<()> {
        let pending: VecDeque<FileEntry> = self.catalog().pending()?.into();
        let total = self.catalog().count()?;
        let mut done = self.catalog().count_done()?;
        if pending.is_empty() {
            report(&Progress {
                done,
                total,
                current: None,
            });
            return Ok(());
        }
        let queue = Arc::new(Mutex::new(pending));
        let stop = Arc::new(AtomicBool::new(false));
        let (sender, outcomes) = mpsc::channel::<Read>();
        let mut workers = Vec::new();
        for mut reader in readers {
            let queue = Arc::clone(&queue);
            let stop = Arc::clone(&stop);
            let sender = sender.clone();
            let root = self.root.clone();
            workers.push(thread::spawn(move || {
                loop {
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    let Some(entry) = queue.lock().ok().and_then(|mut q| q.pop_front()) else {
                        break;
                    };
                    let path = root.join(&entry.relative);
                    let start = Instant::now();
                    let outcome = if walk::is_cloud_placeholder(&path) {
                        Err(Problem::InCloud)
                    } else {
                        reader.read(&path)
                    };
                    let read_ms = millis(start.elapsed());
                    if sender
                        .send(Read {
                            entry,
                            outcome,
                            read_ms,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            }));
        }
        drop(sender);

        let mut writer = self.index.writer(WRITER_MEMORY_MB)?;
        let mut last_save = Instant::now();
        let mut unsaved: Vec<Outcome> = Vec::new();
        for Read {
            entry,
            outcome,
            read_ms,
        } in outcomes
        {
            let start = Instant::now();
            writer.remove_file(entry.id);
            let (status, problem, sections) = match outcome {
                Ok(document) => {
                    for (i, section) in document.sections.iter().enumerate() {
                        writer.add_page(&Page {
                            file_id: entry.id,
                            number: u32::try_from(i + 1).unwrap_or(u32::MAX),
                            label: place_code(&section.place),
                            text: section.text.clone(),
                        })?;
                    }
                    let sections = u32::try_from(document.sections.len()).unwrap_or(u32::MAX);
                    (Status::Indexed, None, sections)
                }
                Err(problem) => (Status::Problem, Some(problem), 0),
            };
            unsaved.push(Outcome {
                id: entry.id,
                status,
                problem,
                sections,
                read_ms,
                index_ms: millis(start.elapsed()),
            });
            done += 1;
            if !report(&Progress {
                done,
                total,
                current: Some(entry.name.clone()),
            }) {
                // Files already read but not yet indexed are read again next time.
                stop.store(true, Ordering::Relaxed);
                break;
            }
            if last_save.elapsed() >= SAVE_EVERY {
                self.save(&mut writer, &mut unsaved)?;
                last_save = Instant::now();
            }
        }
        for worker in workers {
            let _ = worker.join();
        }
        self.save(&mut writer, &mut unsaved)?;
        report(&Progress {
            done,
            total,
            current: None,
        });
        Ok(())
    }

    /// Saves the index first, then records the files as read: if Mulu stops in between,
    /// those files are simply read again.
    fn save(
        &self,
        writer: &mut mulu_index::IndexWriterHandle,
        unsaved: &mut Vec<Outcome>,
    ) -> Result<()> {
        writer.commit()?;
        self.index.refresh()?;
        self.catalog().record(unsaved)?;
        unsaved.clear();
        Ok(())
    }

    /// Runs a search: files whose text contains it, then files whose name does.
    pub fn search(&self, query: &str) -> Result<Answer> {
        let start = Instant::now();
        let results = self.index.search(query)?;
        let catalog = self.catalog();
        let mut files = Vec::with_capacity(results.files.len());
        for file in &results.files {
            if let Some(entry) = catalog.get(file.file_id)? {
                files.push(found(&entry, file.count));
            }
        }
        let in_text: std::collections::HashSet<u64> = files.iter().map(|f| f.id).collect();
        let parts: Vec<mulu_text::Search> = mulu_index::split_query(query)
            .iter()
            .map(|p| mulu_text::search(p))
            .filter(|s| *s != mulu_text::Search::Nothing)
            .collect();
        let by_name = if parts.is_empty() {
            Vec::new()
        } else {
            catalog
                .all()?
                .into_iter()
                .filter(|e| !in_text.contains(&e.id) && names::matches(&e.name, &parts))
                .take(500)
                .map(|e| found(&e, 0))
                .collect()
        };
        drop(catalog);
        let total = results.total();
        *self
            .last
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some((query.to_owned(), results));
        Ok(Answer {
            query: query.to_owned(),
            files,
            by_name,
            total,
            elapsed_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
        })
    }

    /// Passages of one file for the latest search, skipping the first `skip`.
    pub fn passages(
        &self,
        query: &str,
        file_id: u64,
        skip: usize,
        limit: usize,
    ) -> Result<Vec<Passage>> {
        let mut last = self
            .last
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if last.as_ref().is_none_or(|(q, _)| q != query) {
            *last = Some((query.to_owned(), self.index.search(query)?));
        }
        let Some((_, results)) = last.as_ref() else {
            return Ok(Vec::new());
        };
        let Some(file) = results
            .files
            .iter()
            .find(|f: &&FileResult| f.file_id == file_id)
        else {
            return Ok(Vec::new());
        };
        Ok(self
            .index
            .passages(results, file, skip, limit)?
            .into_iter()
            .map(|hit| Passage {
                place: hit.label,
                before: hit.passage.before,
                matched: hit.passage.matched,
                after: hit.passage.after,
            })
            .collect())
    }

    /// Files whose text could not be read, with the reason.
    pub fn problems(&self) -> Result<Vec<FoundFile>> {
        Ok(self
            .catalog()
            .problems()?
            .iter()
            .map(|e| found(e, 0))
            .collect())
    }

    /// Figures about the reading, with the `slowest` slowest files.
    pub fn report(&self, slowest: usize) -> Result<Report> {
        self.catalog().report(slowest)
    }

    /// The full path of a file of the library.
    pub fn path_of(&self, file_id: u64) -> Result<Option<PathBuf>> {
        Ok(self
            .catalog()
            .get(file_id)?
            .map(|e| self.root.join(e.relative)))
    }
}

fn found(entry: &FileEntry, count: u32) -> FoundFile {
    let relative = Path::new(&entry.relative);
    FoundFile {
        id: entry.id,
        name: entry.name.clone(),
        folder: relative
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default(),
        path: entry.relative.clone(),
        count,
        problem: entry.problem.clone(),
    }
}

/// How a section's place is stored in the index (see [`Passage::place`]).
fn place_code(place: &Place) -> String {
    match place {
        Place::Whole => String::new(),
        Place::Page { number, label } => {
            format!("p:{}", label.clone().unwrap_or_else(|| number.to_string()))
        }
        Place::Part {
            title: Some(title), ..
        } => format!("s:{title}"),
        Place::Part {
            number,
            title: None,
        } => format!("n:{number}"),
    }
}
