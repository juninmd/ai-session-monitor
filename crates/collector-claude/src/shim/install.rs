//! `install_shim`: puts our shim in front of whatever `statusLine` the user already has.

use std::io::ErrorKind;
use std::path::Path;

use serde_json::{Map, Value};
use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, HarnessError, ShimStatus};

use super::command::{self, Args};
use super::doc::{MAX_SETTINGS_BYTES, SettingsDoc};
use super::state::{self, Saved};
use super::{Current, classify, failed, invalid, status_at, unreadable};
use crate::paths::Paths;

const MAX_EXE_BYTES: usize = 64 * 1024 * 1024;

/// Test seam: `before_commit` runs once everything is prepared, right before settings.json is
/// re-read and replaced, which is where a concurrent edit would have to land.
pub(crate) struct Hooks<'a> {
    pub before_commit: &'a dyn Fn(),
}

fn noop() {}

impl Hooks<'static> {
    pub(crate) const NONE: Self = Self { before_commit: &noop };
}

/// What happens to `chain.json`.
enum ChainFile {
    Write(String),
    Remove,
    /// The state that says what it should hold is gone: leave it as it is.
    Keep {
        configured: bool,
    },
}

/// Copies `shim_exe` to the app data dir, saves the current `statusLine` and points it at the
/// copy, which chains to the saved command. Idempotent: a second run changes nothing, and never
/// nests the shim inside itself. Needs `settings.shim_consent`.
pub fn install_shim(
    settings: &ClaudeSettings,
    ctx: &Ctx,
    shim_exe: &Path,
) -> Result<ShimStatus, HarnessError> {
    install_with(settings, ctx, shim_exe, &Hooks::NONE)
}

pub(crate) fn install_with(
    settings: &ClaudeSettings,
    ctx: &Ctx,
    shim_exe: &Path,
    hooks: &Hooks<'_>,
) -> Result<ShimStatus, HarnessError> {
    if !settings.shim_consent {
        return Err(invalid("shimConsent"));
    }
    let paths = Paths::resolve(settings, ctx).ok_or_else(|| invalid("configDir"))?;
    let exe_bytes = read_exe(shim_exe)?;
    let mut doc = SettingsDoc::load(&paths.settings_json).map_err(|_| unreadable())?;

    let current = classify(&paths, &doc);
    let ours = matches!(current, Current::Ours { .. });
    // Ours already: what the first install saved stays authoritative, from the app data dir or,
    // when that was wiped, from the copy beside settings.json. Anything else: whatever is
    // configured now is what an uninstall must bring back.
    let saved = if ours {
        state::load_either(&paths.state, &paths.state_copy)
    } else {
        Some(Saved::of(doc.status_line()))
    };
    let previous = saved.as_ref().and_then(Saved::previous).cloned();
    let chain_file = match (&saved, &current) {
        (Some(saved), _) => match saved.previous().and_then(state::chain_command) {
            Some(command) if !state::fits_the_shim(&command) => {
                return Err(failed("status_line_too_large", false));
            }
            Some(command) => ChainFile::Write(command),
            None => ChainFile::Remove,
        },
        (None, Current::Ours { chained, .. }) => ChainFile::Keep { configured: *chained },
        (None, _) => ChainFile::Remove,
    };
    let chained = match &chain_file {
        ChainFile::Write(_) => true,
        ChainFile::Remove => false,
        ChainFile::Keep { configured } => *configured,
    };
    let command = command::build(&Args {
        exe: &paths.exe,
        out: &paths.rate_limits,
        chain: chained.then_some(paths.chain.as_path()),
        home: &paths.home,
    })
    .map_err(|_| failed("path_unsupported", false))?;

    let object = status_line_object(&doc, ours, previous.as_ref(), command);
    doc.set_status_line(Value::Object(object));
    let new_bytes = doc.to_bytes().map_err(|_| failed("serialize_failed", false))?;

    place_exe(&paths, &exe_bytes)?;
    if !ours {
        back_up_once(&paths, &doc)?;
    }
    if let Some(saved) = &saved {
        state::save_both(&paths.state, &paths.state_copy, saved)
            .map_err(|_| failed("state_failed", true))?;
    }
    let chain_written = match &chain_file {
        ChainFile::Write(command) => state::save_chain(&paths.chain, command),
        ChainFile::Remove => state::remove(&paths.chain),
        ChainFile::Keep { .. } => Ok(()),
    };
    chain_written.map_err(|_| failed("state_failed", true))?;

    (hooks.before_commit)();
    if !unchanged_since(&paths.settings_json, doc.original()) {
        return Err(failed("concurrent_modification", true));
    }
    if doc.original() != Some(new_bytes.as_slice()) {
        usage_support::atomic_write(&paths.settings_json, &new_bytes)
            .map_err(|_| failed("write_failed", true))?;
    }
    Ok(status_at(&paths))
}

/// The new `statusLine` object. Other keys of a command-type `statusLine` (for example `padding`)
/// carry over; a `statusLine` of any other shape is saved for the restore but not extended.
fn status_line_object(
    doc: &SettingsDoc,
    ours: bool,
    previous: Option<&Value>,
    command: String,
) -> Map<String, Value> {
    let base = if ours {
        doc.status_line()
    } else {
        previous.filter(|value| state::chain_command(value).is_some())
    };
    let mut object = base.and_then(Value::as_object).cloned().unwrap_or_default();
    object.insert("type".to_owned(), Value::String("command".to_owned()));
    object.insert("command".to_owned(), Value::String(command));
    object
}

pub(super) fn read_exe(path: &Path) -> Result<Vec<u8>, HarnessError> {
    match usage_support::read_capped(path, MAX_EXE_BYTES) {
        Ok(bytes) if !bytes.is_empty() => Ok(bytes),
        _ => Err(failed("shim_missing", false)),
    }
}

pub(super) fn place_exe(paths: &Paths, bytes: &[u8]) -> Result<(), HarnessError> {
    let same =
        usage_support::read_capped(&paths.exe, MAX_EXE_BYTES).is_ok_and(|have| have == bytes);
    if same {
        return Ok(());
    }
    usage_support::atomic_write(&paths.exe, bytes).map_err(|_| failed("copy_failed", true))
}

/// The first install keeps the user's original file next to it; later installs leave it alone.
fn back_up_once(paths: &Paths, doc: &SettingsDoc) -> Result<(), HarnessError> {
    match doc.original() {
        Some(original) if !paths.backup.exists() => {
            usage_support::atomic_write(&paths.backup, original)
                .map_err(|_| failed("backup_failed", true))
        }
        _ => Ok(()),
    }
}

/// True when the file still holds exactly what was read (or is still absent).
pub(super) fn unchanged_since(path: &Path, before: Option<&[u8]>) -> bool {
    match (usage_support::read_capped(path, MAX_SETTINGS_BYTES), before) {
        (Ok(now), Some(before)) => now == before,
        (Err(e), None) => e.kind() == ErrorKind::NotFound,
        _ => false,
    }
}
