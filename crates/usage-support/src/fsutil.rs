//! File helpers with the guarantees collectors rely on: writes are atomic, reads are bounded.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde::de::DeserializeOwned;

/// Windows only: another process may hold the target without delete sharing for a moment.
const RENAME_ATTEMPTS: u32 = 8;
const RENAME_BACKOFF: Duration = Duration::from_millis(25);
const TEMP_NAME_ATTEMPTS: u32 = 8;
/// Keeps the temp name under the 255 character file name limit for long target names.
const TEMP_NAME_STEM_CHARS: usize = 64;
/// Largest buffer reserved up front, so a lying file size cannot trigger a huge allocation.
const PREALLOCATE_LIMIT: u64 = 1 << 20;
const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

/// The directory a CLI child runs in, so it starts outside any project. It lives under the app's
/// own data dir: a shared temp dir may hold a `.claude` folder that someone else left there.
pub fn neutral_dir(data_dir: &Path) -> io::Result<PathBuf> {
    let dir = data_dir.join("cli-cwd");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Replaces `path` with `bytes` so a concurrent reader sees the old or the new content, never a
/// partial or empty file: the bytes go to a temp file in the same directory, are flushed and
/// synced, then renamed over the target. Missing parent directories are created; the temp file is
/// removed on any failure. Blocking: from async code call it through `spawn_blocking`.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no file name"))?;
    let dir = path.parent().unwrap_or_else(|| Path::new(""));
    fs::create_dir_all(dir)?;

    let (mut file, temp) = create_temp(dir, &name.to_string_lossy())?;
    let mut guard = TempGuard { path: temp, armed: true };
    file.write_all(bytes)?;
    file.flush()?;
    file.sync_all()?;
    drop(file);

    rename_over(&guard.path, path)?;
    guard.armed = false;
    Ok(())
}

/// Removes the temp file unless the rename consumed it, including when a panic unwinds past it.
struct TempGuard {
    path: PathBuf,
    armed: bool,
}

impl Drop for TempGuard {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn create_temp(dir: &Path, target_name: &str) -> io::Result<(File, PathBuf)> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut last = None;
    for _ in 0..TEMP_NAME_ATTEMPTS {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let stem: String = target_name.chars().take(TEMP_NAME_STEM_CHARS).collect();
        let temp = dir.join(format!(".{stem}.{}.{n}.tmp", std::process::id()));
        match OpenOptions::new().write(true).create_new(true).open(&temp) {
            Ok(file) => return Ok((file, temp)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => last = Some(e),
            Err(e) => return Err(e),
        }
    }
    Err(last.unwrap_or_else(|| io::Error::other("no free temp file name")))
}

fn rename_over(from: &Path, to: &Path) -> io::Result<()> {
    let mut attempt = 1;
    loop {
        match fs::rename(from, to) {
            Err(e) if attempt < RENAME_ATTEMPTS && is_transient_rename_error(&e) => {
                std::thread::sleep(RENAME_BACKOFF * attempt);
                attempt += 1;
            }
            result => return result,
        }
    }
}

/// 5 = access denied, 32 = sharing violation, 33 = lock violation: a reader holds the target.
fn is_transient_rename_error(e: &io::Error) -> bool {
    cfg!(windows)
        && (e.kind() == io::ErrorKind::PermissionDenied
            || matches!(e.raw_os_error(), Some(5 | 32 | 33)))
}

/// Reads the whole file, failing with `FileTooLarge` when it holds more than `max_bytes`. The
/// size is checked before reading and the read itself stops one byte past the cap, so neither a
/// huge nor a growing file is ever loaded in full.
pub fn read_capped(path: &Path, max_bytes: usize) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let cap = u64::try_from(max_bytes).unwrap_or(u64::MAX);
    let size = file.metadata()?.len();
    if size > cap {
        return Err(too_large(max_bytes));
    }
    let mut buf = Vec::with_capacity(usize::try_from(size.min(PREALLOCATE_LIMIT)).unwrap_or(0));
    file.take(cap.saturating_add(1)).read_to_end(&mut buf)?;
    if buf.len() > max_bytes {
        return Err(too_large(max_bytes));
    }
    Ok(buf)
}

fn too_large(max_bytes: usize) -> io::Error {
    io::Error::new(io::ErrorKind::FileTooLarge, format!("file exceeds the {max_bytes} byte limit"))
}

/// `read_capped` plus JSON decoding. A UTF-8 byte order mark is accepted, as Windows tools write
/// one. The decode error says where and what kind, never the offending text: serde echoes values.
pub fn read_json_capped<T: DeserializeOwned>(path: &Path, max_bytes: usize) -> io::Result<T> {
    let bytes = read_capped(path, max_bytes)?;
    let json = bytes.strip_prefix(UTF8_BOM).unwrap_or(&bytes);
    serde_json::from_slice(json).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid JSON ({:?}) at line {} column {}", e.classify(), e.line(), e.column()),
        )
    })
}
