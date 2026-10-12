//! Replaces a file atomically, so a reader in another process never sees a half-written one.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const RENAME_ATTEMPTS: u32 = 8;

/// Temp file in the same directory, flushed, then renamed over the target. Windows may refuse the
/// rename for a moment while a reader (or a scanner) has the target open, hence the short retry.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no file name"))?;
    let dir = path.parent().filter(|dir| !dir.as_os_str().is_empty()).unwrap_or(Path::new("."));
    fs::create_dir_all(dir)?;
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_nanos();
    let temp: PathBuf =
        dir.join(format!(".{}.{}.{nanos}.tmp", name.to_string_lossy(), std::process::id()));
    let result = write_and_rename(&temp, path, bytes);
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn write_and_rename(temp: &Path, target: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    for attempt in 1..=RENAME_ATTEMPTS {
        match fs::rename(temp, target) {
            Err(error) if attempt < RENAME_ATTEMPTS && is_transient(&error) => {
                std::thread::sleep(Duration::from_millis(10) * attempt);
            }
            other => return other,
        }
    }
    Ok(())
}

/// 5 = access denied, 32 = sharing violation, 33 = lock violation.
fn is_transient(error: &io::Error) -> bool {
    cfg!(windows)
        && (error.kind() == io::ErrorKind::PermissionDenied
            || matches!(error.raw_os_error(), Some(5 | 32 | 33)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_atomic_creates_parents_and_replaces_without_leaving_temp_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a b/ção/out.json");
        write_atomic(&path, b"one").unwrap();
        write_atomic(&path, b"two").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"two");
        let leftovers: Vec<_> = fs::read_dir(path.parent().unwrap()).unwrap().collect();
        assert_eq!(leftovers.len(), 1, "temp files must not stay behind");
    }

    #[test]
    fn the_target_is_replaced_not_rewritten_in_place() {
        let dir = tempfile::tempdir().unwrap();
        let (path, alias) = (dir.path().join("out.json"), dir.path().join("alias.json"));
        fs::write(&path, b"one").unwrap();
        fs::hard_link(&path, &alias).unwrap();

        write_atomic(&path, b"two").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"two");
        // A reader that still holds the old file sees it whole, never a half-written mix.
        assert_eq!(fs::read(&alias).unwrap(), b"one");
    }
}
