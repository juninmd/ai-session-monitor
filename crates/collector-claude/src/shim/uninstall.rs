//! `uninstall_shim`: puts the user's previous `statusLine` back, exactly as it was.

use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, HarnessError, ShimStatus};

use super::doc::SettingsDoc;
use super::install::{Hooks, unchanged_since};
use super::state::{self, Saved};
use super::{Current, classify, failed, invalid, status_at, unreadable};
use crate::paths::Paths;

/// Restores the saved `statusLine` (or removes the key when there was none) and deletes our files.
/// A `statusLine` that is not ours is never touched: it is refused while our files still exist (it
/// may be a hand-edited copy of ours, which deleting the executable would break) and is a no-op
/// once they are gone. A settings.json that is missing or already clean only has our files removed.
/// Needs no consent: leaving is always allowed.
pub fn uninstall_shim(settings: &ClaudeSettings, ctx: &Ctx) -> Result<ShimStatus, HarnessError> {
    uninstall_with(settings, ctx, &Hooks::NONE)
}

pub(crate) fn uninstall_with(
    settings: &ClaudeSettings,
    ctx: &Ctx,
    hooks: &Hooks<'_>,
) -> Result<ShimStatus, HarnessError> {
    let paths = Paths::resolve(settings, ctx).ok_or_else(|| invalid("configDir"))?;
    let mut doc = SettingsDoc::load(&paths.settings_json).map_err(|_| unreadable())?;
    match classify(&paths, &doc) {
        Current::Foreign if leftovers(&paths) => return Err(failed("foreign_status_line", false)),
        Current::Foreign | Current::NotInstalled => {}
        Current::Ours { .. } => {
            // No saved state, in the data dir or beside settings.json, means there is nothing to
            // give back: the key goes. The backup is not a source: it holds the first original.
            let saved = state::load_either(&paths.state, &paths.state_copy);
            match saved.as_ref().and_then(Saved::previous) {
                Some(previous) => doc.set_status_line(previous.clone()),
                None => doc.remove_status_line(),
            }
            let bytes = doc.to_bytes().map_err(|_| failed("serialize_failed", false))?;
            (hooks.before_commit)();
            if !unchanged_since(&paths.settings_json, doc.original()) {
                return Err(failed("concurrent_modification", true));
            }
            usage_support::atomic_write(&paths.settings_json, &bytes)
                .map_err(|_| failed("write_failed", true))?;
        }
    }
    remove_own_files(&paths);
    Ok(status_at(&paths))
}

/// Whether anything of an earlier install is still on disk.
fn leftovers(paths: &Paths) -> bool {
    [&paths.exe, &paths.state, &paths.state_copy, &paths.chain].iter().any(|file| file.exists())
}

/// Best effort: a copy that is still running cannot be deleted on Windows, and nothing refers to
/// it any more. The recording and the OAuth throttle go too, so a later reinstall does not show
/// weeks-old numbers, and so does the copy of the saved record beside settings.json. The backup of
/// the original settings.json is the user's and stays.
fn remove_own_files(paths: &Paths) {
    let files = [
        &paths.chain,
        &paths.state,
        &paths.state_copy,
        &paths.rate_limits,
        &paths.oauth_cache,
        &paths.exe,
    ];
    for file in files {
        let _ = state::remove(file);
    }
    // `remove_dir` only removes empty directories: nothing foreign, and no other profile's file,
    // can be lost here.
    for dir in &paths.own_dirs {
        let _ = std::fs::remove_dir(dir);
    }
}
