//! Finding the book files of a folder, and recognizing files that live in the cloud.

use mulu_extract::Format;
use std::path::{Path, PathBuf};

pub struct Found {
    pub path: PathBuf,
    pub size: u64,
    /// Last modification, in seconds since 1970.
    pub modified: i64,
}

/// What a library leaves out when it is read, chosen in the survey before the first
/// reading (instructions, section 1): formats, by their short name (see
/// [`format_name`]), and folders, relative to the library folder with `/` between
/// parts, each with everything below it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Skip {
    pub formats: Vec<String>,
    pub folders: Vec<String>,
}

impl Skip {
    /// One rule per line: `format:pdf`, `folder:Cursos/2019`.
    pub fn to_text(&self) -> String {
        let formats = self.formats.iter().map(|f| format!("format:{f}"));
        let folders = self.folders.iter().map(|f| format!("folder:{f}"));
        formats.chain(folders).collect::<Vec<_>>().join("\n")
    }

    pub fn from_text(text: &str) -> Skip {
        let mut skip = Skip::default();
        for line in text.lines() {
            if let Some(format) = line.strip_prefix("format:") {
                skip.formats.push(format.to_owned());
            } else if let Some(folder) = line.strip_prefix("folder:") {
                skip.folders.push(folder.to_owned());
            }
        }
        skip
    }

    pub fn is_empty(&self) -> bool {
        self.formats.is_empty() && self.folders.is_empty()
    }
}

/// The short name of a format, as the survey counts and the user chooses them.
pub fn format_name(format: Format) -> &'static str {
    match format {
        Format::Pdf => "pdf",
        Format::Epub => "epub",
        Format::Text => "txt",
        Format::Markdown => "md",
        Format::Html => "html",
    }
}

/// Folders left out at the top of a disk: the system and the programs (for a disk
/// chosen as a library, or "whole computer"). Elsewhere these names are ordinary
/// folders.
const DISK_SYSTEM_FOLDERS: &[&str] = &[
    "Windows",
    "Windows.old",
    "Program Files",
    "Program Files (x86)",
    "ProgramData",
    "Recovery",
    "PerfLogs",
    "$WinREAgent",
    "System",
    "Library",
    "Applications",
    "private",
    "usr",
    "bin",
    "sbin",
    "etc",
    "var",
    "opt",
    "tmp",
    "proc",
    "dev",
    "sys",
    "boot",
    "snap",
];

/// Every file of a supported format under `root`, leaving out the formats and folders
/// of `skip`. Hidden files and folders, system folders and links are skipped (links
/// could lead outside the library, or in circles); macOS's placeholders for files kept
/// only in iCloud are listed under their real name. Skipped folders are not even
/// opened, so leaving out a large folder also makes the walk faster.
pub fn book_files_skipping(root: &Path, skip: &Skip) -> Vec<Found> {
    let mut found = Vec::new();
    let at_disk_top = root.parent().is_none();
    let mut folders = vec![root.to_owned()];
    while let Some(folder) = folders.pop() {
        let Ok(entries) = std::fs::read_dir(&folder) else {
            continue;
        };
        let top = folder == root;
        for entry in entries.flatten() {
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let path = entry.path();
            if metadata.file_type().is_symlink() || is_system(&name, &metadata) {
                continue;
            }
            if metadata.is_dir() {
                let skipped =
                    !skip.folders.is_empty() && skip.folders.contains(&relative_key(root, &path));
                let system = top && at_disk_top && DISK_SYSTEM_FOLDERS.contains(&name.as_ref());
                if !name.starts_with('.') && !skipped && !system {
                    folders.push(path);
                }
                continue;
            }
            let real = icloud_real_name(&name);
            if name.starts_with('.') && real.is_none() {
                continue;
            }
            let shown = real.map_or_else(|| path.clone(), |real| path.with_file_name(real));
            let Some(format) = Format::from_path(&shown) else {
                continue;
            };
            if skip.formats.iter().any(|f| f == format_name(format)) {
                continue;
            }
            found.push(Found {
                path,
                size: metadata.len(),
                modified: metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .and_then(|d| i64::try_from(d.as_secs()).ok())
                    .unwrap_or(0),
            });
        }
    }
    found
}

/// A path relative to `root`, with `/` between parts.
fn relative_key(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// What a folder holds, for the survey shown before the first reading: how many book
/// files of each format, how many are only in the cloud, and how many are in each
/// folder (with everything below it).
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct Survey {
    pub files: u64,
    pub megabytes: u64,
    /// Files only in the cloud, which Mulu lists but never downloads.
    pub in_cloud: u64,
    pub formats: Vec<(String, u64)>,
    pub folders: Vec<crate::Folder>,
}

/// Counts the book files under `root`, leaving out the folders of `skip` (used for
/// folders that are already libraries of their own).
pub fn survey(root: &Path, skip: &Skip) -> Survey {
    let found = book_files_skipping(root, skip);
    let mut formats: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut folders: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut bytes = 0u64;
    let mut in_cloud = 0u64;
    for file in &found {
        bytes += file.size;
        if is_cloud_placeholder(&file.path) {
            in_cloud += 1;
        }
        let shown = std::path::PathBuf::from(shown_name(&file.path));
        if let Some(format) = Format::from_path(&shown) {
            *formats.entry(format_name(format).to_owned()).or_default() += 1;
        }
        let key = relative_key(root, &file.path);
        let mut at = key.rfind('/').map_or("", |i| &key[..i]);
        while !at.is_empty() {
            *folders.entry(at.to_owned()).or_default() += 1;
            at = at.rfind('/').map_or("", |i| &at[..i]);
        }
    }
    let mut formats: Vec<(String, u64)> = formats.into_iter().collect();
    formats.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    Survey {
        files: found.len() as u64,
        megabytes: bytes / 1_000_000,
        in_cloud,
        formats,
        folders: folders
            .into_iter()
            .map(|(path, files)| crate::Folder { path, files })
            .collect(),
    }
}

/// The name shown for a file: its own, or the real name of an iCloud placeholder.
pub fn shown_name(path: &Path) -> String {
    let raw = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    icloud_real_name(&raw).unwrap_or(&raw).to_owned()
}

/// Bytes read at each end of a file for its fingerprint.
const FINGERPRINT_BYTES: u64 = 64 * 1024;

/// A short summary of a file's content: SHA-256 of its size, its first 64 KB and its
/// last 64 KB. Two files with the same fingerprint are taken to be the same book, so a
/// book renamed or moved inside the library keeps its place in the index instead of
/// being read again. `None` if the file cannot be opened, and always for files in the
/// cloud, which must not be opened (opening them downloads them).
pub fn fingerprint(path: &Path) -> Option<Vec<u8>> {
    use sha2::{Digest, Sha256};
    use std::io::{Read, Seek, SeekFrom};
    if is_cloud_placeholder(path) {
        return None;
    }
    let mut file = std::fs::File::open(path).ok()?;
    let size = file.metadata().ok()?.len();
    let mut hash = Sha256::new();
    hash.update(size.to_le_bytes());
    let mut buffer = Vec::new();
    (&mut file)
        .take(FINGERPRINT_BYTES)
        .read_to_end(&mut buffer)
        .ok()?;
    hash.update(&buffer);
    if size > FINGERPRINT_BYTES {
        buffer.clear();
        file.seek(SeekFrom::Start(
            size.saturating_sub(FINGERPRINT_BYTES)
                .max(FINGERPRINT_BYTES),
        ))
        .ok()?;
        file.take(FINGERPRINT_BYTES).read_to_end(&mut buffer).ok()?;
        hash.update(&buffer);
    }
    Some(hash.finalize().to_vec())
}

/// For an iCloud placeholder (`.Livro.pdf.icloud`), the real name (`Livro.pdf`).
pub fn icloud_real_name(name: &str) -> Option<&str> {
    name.strip_prefix('.')?
        .strip_suffix(".icloud")
        .filter(|n| !n.is_empty())
}

fn is_system(name: &str, metadata: &std::fs::Metadata) -> bool {
    if matches!(
        name,
        "$RECYCLE.BIN" | "System Volume Information" | "node_modules" | "__MACOSX"
    ) {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const HIDDEN: u32 = 0x2;
        const SYSTEM: u32 = 0x4;
        if metadata.file_attributes() & (HIDDEN | SYSTEM) != 0 {
            return true;
        }
    }
    #[cfg(target_os = "macos")]
    {
        // Folders hidden in the Finder, such as the Library folder of the home folder.
        use std::os::macos::fs::MetadataExt;
        const UF_HIDDEN: u32 = 0x0000_8000;
        if metadata.st_flags() & UF_HIDDEN != 0 {
            return true;
        }
    }
    let _ = metadata;
    false
}

/// Whether a file is in the cloud and not on this computer, so that reading it would
/// make the cloud service download it. Checked without opening the file.
pub fn is_cloud_placeholder(path: &Path) -> bool {
    if path
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| icloud_real_name(n).is_some())
    {
        return true;
    }
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return false;
    };
    #[cfg(windows)]
    {
        // OneDrive, iCloud for Windows, Dropbox and Google Drive mark files that are
        // "online only" with these attributes.
        use std::os::windows::fs::MetadataExt;
        const OFFLINE: u32 = 0x0000_1000;
        const RECALL_ON_OPEN: u32 = 0x0004_0000;
        const RECALL_ON_DATA_ACCESS: u32 = 0x0040_0000;
        return metadata.file_attributes() & (OFFLINE | RECALL_ON_OPEN | RECALL_ON_DATA_ACCESS)
            != 0;
    }
    #[cfg(target_os = "macos")]
    {
        // Recent macOS versions keep iCloud files as "dataless" files instead.
        use std::os::macos::fs::MetadataExt;
        const SF_DATALESS: u32 = 0x4000_0000;
        return metadata.st_flags() & SF_DATALESS != 0;
    }
    #[allow(unreachable_code)]
    {
        let _ = metadata;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::icloud_real_name;

    #[test]
    fn icloud_placeholders() {
        assert_eq!(
            icloud_real_name(".Dom Casmurro.pdf.icloud"),
            Some("Dom Casmurro.pdf")
        );
        assert_eq!(icloud_real_name("Dom Casmurro.pdf"), None);
        assert_eq!(icloud_real_name("..icloud"), None);
    }
}
