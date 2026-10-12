//! The history file: bounded on read and on write, and never trusted. A damaged file is set aside
//! and the history starts empty; nothing about its content is reported anywhere.

use std::io;
use std::path::Path;

use usage_support::fsutil::{atomic_write, read_json_capped};

use super::ledger::{Ledger, VERSION};

/// What a save may write. The read limit is looser so a file near this size never counts as damage.
pub(super) const MAX_WRITE_BYTES: usize = 1 << 20;
const MAX_READ_BYTES: usize = 2 << 20;
/// One set-aside copy: a newer damaged file replaces it instead of piling up.
pub(super) const QUARANTINE_NAME: &str = "history.corrupt.json";

/// Missing, unreadable or damaged files all give an empty ledger; only a damaged one is moved aside.
pub(super) fn load(path: &Path, now: i64) -> Ledger {
    match read_json_capped::<Ledger>(path, MAX_READ_BYTES) {
        Ok(mut ledger) if ledger.version == VERSION => {
            ledger.sanitize();
            ledger.prune(now);
            ledger
        }
        Ok(_) => set_aside(path),
        Err(error) => match error.kind() {
            io::ErrorKind::InvalidData | io::ErrorKind::FileTooLarge => set_aside(path),
            _ => Ledger::default(),
        },
    }
}

fn set_aside(path: &Path) -> Ledger {
    // A failed rename leaves the bad file where the next save replaces it; history is expendable.
    let _ = std::fs::rename(path, path.with_file_name(QUARANTINE_NAME));
    Ledger::default()
}

/// Serializes within `max_bytes`, dropping the oldest points of `ledger` itself until it fits.
pub(super) fn encode(ledger: &mut Ledger, max_bytes: usize) -> Option<Vec<u8>> {
    loop {
        let bytes = serde_json::to_vec(ledger).ok()?;
        if bytes.len() <= max_bytes {
            return Some(bytes);
        }
        ledger.cap_points(ledger.point_count() * 9 / 10);
    }
}

pub(super) fn save(path: &Path, ledger: &mut Ledger) -> io::Result<()> {
    let bytes = encode(ledger, MAX_WRITE_BYTES)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "history not serializable"))?;
    atomic_write(path, &bytes)
}
