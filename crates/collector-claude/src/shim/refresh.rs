//! `refresh_shim_copy`: keeps the installed copy of the shim in step with the bundled build.

use std::path::Path;

use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, HarnessError, ShimState};

use super::install::{place_exe, read_exe};
use super::status_at;
use crate::paths::Paths;

/// Replaces the installed copy of the shim with `shim_exe`, the build bundled with the running app,
/// when the bytes differ. An app update ships a fixed shim, but only an install ever wrote the copy.
///
/// Copy only: it acts when the user consented and settings.json holds our healthy command, and it
/// never writes settings.json, the saved state or the chain file, nor wraps a `statusLine`. Anything
/// else (somebody else's command, a copy that went missing, an unreadable file) is for the user to
/// settle from the tab, so it is `Ok` and untouched. A copy that cannot be replaced right now
/// (Windows refuses one Claude Code is running) is `copy_failed`, retryable at the next start.
pub fn refresh_shim_copy(
    settings: &ClaudeSettings,
    ctx: &Ctx,
    shim_exe: &Path,
) -> Result<(), HarnessError> {
    refresh_with(settings, ctx, shim_exe, &|| {})
}

/// `before_place` is a test seam: it runs after the bundled build is read, where a concurrent
/// uninstall would have to land.
pub(crate) fn refresh_with(
    settings: &ClaudeSettings,
    ctx: &Ctx,
    shim_exe: &Path,
    before_place: &dyn Fn(),
) -> Result<(), HarnessError> {
    if !settings.shim_consent {
        return Ok(());
    }
    let Some(paths) = Paths::resolve(settings, ctx) else { return Ok(()) };
    if status_at(&paths).state != ShimState::Installed {
        return Ok(());
    }
    let bytes = read_exe(shim_exe)?;
    before_place();
    // An uninstall since the check above must not get its copy written back.
    if status_at(&paths).state != ShimState::Installed {
        return Ok(());
    }
    place_exe(&paths, &bytes)
}
