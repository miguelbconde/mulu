//! Finding the book files of a folder, and recognizing files that live in the cloud.

use mulu_extract::Format;
use std::path::{Path, PathBuf};

pub struct Found {
    pub path: PathBuf,
    pub size: u64,
    /// Last modification, in seconds since 1970.
    pub modified: i64,
}

/// Every file of a supported format under `root`. Hidden files and folders, system
/// folders and links are skipped (links could lead outside the library, or in circles);
/// macOS's placeholders for files kept only in iCloud are listed under their real name.
pub fn book_files(root: &Path) -> Vec<Found> {
    let mut found = Vec::new();
    let mut folders = vec![root.to_owned()];
    while let Some(folder) = folders.pop() {
        let Ok(entries) = std::fs::read_dir(&folder) else {
            continue;
        };
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
                if !name.starts_with('.') {
                    folders.push(path);
                }
                continue;
            }
            let real = icloud_real_name(&name);
            if name.starts_with('.') && real.is_none() {
                continue;
            }
            let shown = real.map_or_else(|| path.clone(), |real| path.with_file_name(real));
            if Format::from_path(&shown).is_none() {
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
