//! Management of Claude Code's `statusLine` setting in `<config dir>/settings.json`.
//!
//! Rules that hold for every operation here:
//! - a settings.json that is not a valid JSON object is never modified (state `Unreadable`);
//! - the user's key order, unknown keys and formatting survive, and the previous `statusLine` is
//!   saved and put back verbatim on uninstall. The record is kept twice, in the app data dir and
//!   beside settings.json (`settings.json.aism-state`), so a wiped data dir cannot lose it; the
//!   backup is never a source, it holds the first original and may be older;
//! - every file an install owns in the app data dir is per config dir (`paths::profile_key`), so
//!   installing into, or removing from, one Claude profile never reaches another;
//! - the file is re-read right before it is replaced, and the operation aborts if it changed in
//!   between (the narrow window left is the gap between that read and the rename);
//! - everything the settings file refers to is written before the settings file itself.
//!
//! Only the user-level settings.json is managed. A `statusLine` in project settings or in a managed
//! policy file overrides it and cannot be seen from here. The public functions block on file I/O:
//! async callers run them on a blocking thread.

mod command;
mod doc;
mod install;
mod refresh;
mod state;
mod uninstall;

#[cfg(test)]
mod tests;

use serde_json::Value;
use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, ErrorCode, HarnessError, ShimState, ShimStatus};

use crate::paths::Paths;
use doc::SettingsDoc;

pub use install::install_shim;
pub use refresh::refresh_shim_copy;
pub use uninstall::uninstall_shim;

/// Where Claude Code's settings.json stands. Never writes.
pub fn shim_status(settings: &ClaudeSettings, ctx: &Ctx) -> ShimStatus {
    match Paths::resolve(settings, ctx) {
        Some(paths) => status_at(&paths),
        None => ShimStatus { state: ShimState::Unreadable, chained: false, settings_path: None },
    }
}

pub(crate) fn status_at(paths: &Paths) -> ShimStatus {
    let settings_path = Some(paths.settings_json.display().to_string());
    let (state, chained) = match SettingsDoc::load(&paths.settings_json) {
        Err(_) => (ShimState::Unreadable, false),
        Ok(doc) => match classify(paths, &doc) {
            Current::NotInstalled => (ShimState::NotInstalled, false),
            Current::Foreign => (ShimState::Foreign, false),
            Current::Ours { healthy: true, chained } => (ShimState::Installed, chained),
            Current::Ours { healthy: false, chained } => (ShimState::Outdated, chained),
        },
    };
    ShimStatus { state, chained, settings_path }
}

/// How the `statusLine` currently in the document relates to us.
pub(crate) enum Current {
    NotInstalled,
    /// Somebody else's command (or a shape we do not understand).
    Foreign,
    /// Our shim. `healthy`: it points at the current executable and data file, and the
    /// executable exists.
    Ours {
        healthy: bool,
        chained: bool,
    },
}

pub(crate) fn classify(paths: &Paths, doc: &SettingsDoc) -> Current {
    let Some(value) = doc.status_line().filter(|value| !value.is_null()) else {
        return Current::NotInstalled;
    };
    let parsed = value
        .get("command")
        .and_then(Value::as_str)
        .and_then(|text| command::parse(text, &paths.home));
    let Some(parsed) = parsed else { return Current::Foreign };
    let healthy = command::same_path(&parsed.exe, &paths.exe)
        && parsed.out.as_deref().is_some_and(|out| command::same_path(out, &paths.rate_limits))
        && paths.exe.is_file();
    let chained = parsed.chain.is_some() && paths.chain.is_file();
    Current::Ours { healthy, chained }
}

/// `InstallFailed` with a machine-readable reason (never a path, which may name the user).
pub(crate) fn failed(reason: &'static str, retryable: bool) -> HarnessError {
    HarnessError::new(ErrorCode::InstallFailed, retryable).with_param("reason", reason)
}

pub(crate) fn unreadable() -> HarnessError {
    HarnessError::new(ErrorCode::SettingsUnreadable, false)
}

pub(crate) fn invalid(field: &'static str) -> HarnessError {
    HarnessError::new(ErrorCode::InvalidInput, false).with_param("field", field)
}
