//! Where opencode.db lives. The directory setting is untrusted input, so it is validated before
//! any file is touched.

use std::path::{Component, Path, PathBuf, Prefix};

use usage_model::Ctx;
use usage_model::settings::OpencodeSettings;

use super::Reason;

const DB_FILE: &str = "opencode.db";

/// The configured directory, else OpenCode's XDG-style default under the home directory (that is
/// where it lives on Windows too). The process environment is deliberately never consulted.
pub(crate) fn resolve(settings: &OpencodeSettings, ctx: &Ctx) -> Result<PathBuf, Reason> {
    let configured = settings.data_dir.as_deref().map(str::trim).filter(|dir| !dir.is_empty());
    let Some(dir) = configured else {
        return Ok(ctx.home_dir.join(".local").join("share").join("opencode").join(DB_FILE));
    };
    if dir.contains('\0') || !is_local_absolute(Path::new(dir)) {
        return Err(Reason::InvalidPath);
    }
    Ok(Path::new(dir).join(DB_FILE))
}

/// A relative path would resolve against whatever the process cwd is, and a network share is
/// both a credential-leak vector (SMB auth to a chosen host) and unsupported by SQLite's WAL mode.
fn is_local_absolute(path: &Path) -> bool {
    if !path.is_absolute() {
        return false;
    }
    match path.components().next() {
        Some(Component::Prefix(prefix)) => {
            matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_))
        }
        Some(_) => true,
        None => false,
    }
}
