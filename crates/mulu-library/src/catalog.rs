//! The catalog: one row per book file found, with what happened when it was read.

use crate::Result;
use crate::walk::Found;
use mulu_extract::Problem;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pending,
    Indexed,
    Problem,
}

impl Status {
    fn code(self) -> &'static str {
        match self {
            Status::Pending => "pending",
            Status::Indexed => "indexed",
            Status::Problem => "problem",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub id: u64,
    /// Path relative to the library folder.
    pub relative: String,
    pub name: String,
    pub problem: Option<Problem>,
}

pub struct Catalog {
    db: Connection,
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS files (
    id        INTEGER PRIMARY KEY,
    relative  TEXT NOT NULL UNIQUE,
    name      TEXT NOT NULL,
    size      INTEGER NOT NULL,
    modified  INTEGER NOT NULL,
    status    TEXT NOT NULL DEFAULT 'pending',
    problem   TEXT,
    sections  INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS files_status ON files(status);
PRAGMA user_version = 1;
";

impl Catalog {
    pub fn open(path: &Path) -> Result<Catalog> {
        let db = Connection::open(path)?;
        // Write-ahead logging: searches can read while the indexing writes.
        db.pragma_update(None, "journal_mode", "WAL")?;
        db.execute_batch(SCHEMA)?;
        Ok(Catalog { db })
    }

    pub fn add_new(&mut self, root: &Path, found: &[Found]) -> Result<()> {
        let tx = self.db.transaction()?;
        {
            let mut insert = tx.prepare(
                "INSERT OR IGNORE INTO files (relative, name, size, modified) VALUES (?1, ?2, ?3, ?4)",
            )?;
            for file in found {
                let relative = file.path.strip_prefix(root).unwrap_or(&file.path);
                let raw = file
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let name = crate::walk::icloud_real_name(&raw)
                    .unwrap_or(&raw)
                    .to_owned();
                insert.execute(params![
                    relative.to_string_lossy(),
                    name,
                    i64::try_from(file.size).unwrap_or(i64::MAX),
                    file.modified
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn count(&self) -> Result<u64> {
        let n: i64 = self
            .db
            .query_row("SELECT count(*) FROM files", [], |r| r.get(0))?;
        Ok(u64::try_from(n).unwrap_or(0))
    }

    pub fn count_done(&self) -> Result<u64> {
        let n: i64 = self.db.query_row(
            "SELECT count(*) FROM files WHERE status != 'pending'",
            [],
            |r| r.get(0),
        )?;
        Ok(u64::try_from(n).unwrap_or(0))
    }

    fn entries(&self, filter: &str) -> Result<Vec<FileEntry>> {
        let sql =
            format!("SELECT id, relative, name, problem FROM files {filter} ORDER BY relative");
        let mut query = self.db.prepare(&sql)?;
        let rows = query.query_map([], row_to_entry)?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn pending(&self) -> Result<Vec<FileEntry>> {
        self.entries("WHERE status = 'pending'")
    }

    pub fn problems(&self) -> Result<Vec<FileEntry>> {
        self.entries("WHERE status = 'problem'")
    }

    pub fn all(&self) -> Result<Vec<FileEntry>> {
        self.entries("")
    }

    pub fn get(&self, id: u64) -> Result<Option<FileEntry>> {
        let id = i64::try_from(id).unwrap_or(-1);
        Ok(self
            .db
            .query_row(
                "SELECT id, relative, name, problem FROM files WHERE id = ?1",
                [id],
                row_to_entry,
            )
            .optional()?)
    }

    pub fn record(&mut self, outcomes: &[(u64, Status, Option<Problem>, u32)]) -> Result<()> {
        let tx = self.db.transaction()?;
        {
            let mut update = tx.prepare(
                "UPDATE files SET status = ?2, problem = ?3, sections = ?4 WHERE id = ?1",
            )?;
            for (id, status, problem, sections) in outcomes {
                let problem = problem
                    .as_ref()
                    .map(|p| serde_json::to_string(p).unwrap_or_default());
                update.execute(params![
                    i64::try_from(*id).unwrap_or(-1),
                    status.code(),
                    problem,
                    sections
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<FileEntry> {
    let id: i64 = row.get(0)?;
    let problem: Option<String> = row.get(3)?;
    Ok(FileEntry {
        id: u64::try_from(id).unwrap_or(0),
        relative: row.get(1)?,
        name: row.get(2)?,
        problem: problem.and_then(|p| serde_json::from_str(&p).ok()),
    })
}
