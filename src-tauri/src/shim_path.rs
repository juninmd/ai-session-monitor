//! Where the packaged `statusline-shim` lives. Tauri installs an `externalBin` sidecar next to the
//! app executable without the target triple, and a plain workspace build leaves it in the same
//! folder, so one candidate covers installs and dev runs. `build.rs` stages a tiny placeholder in
//! debug builds, hence the content check before the path is handed to Claude Code's settings.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use usage_model::{ErrorCode, HarnessError};

#[cfg(test)]
mod tests;

const SHIM_FILE: &str = if cfg!(windows) { "statusline-shim.exe" } else { "statusline-shim" };
/// A real executable is far larger; matches the guard in build.rs.
const MIN_BYTES: u64 = 4096;

#[derive(Debug, PartialEq, Eq)]
pub enum ShimPathError {
    Missing,
    /// A file is there but is not an executable (the debug placeholder, a truncated copy).
    Invalid,
}

impl From<ShimPathError> for HarnessError {
    fn from(err: ShimPathError) -> Self {
        let reason = match err {
            ShimPathError::Missing => "shim_missing",
            ShimPathError::Invalid => "shim_invalid",
        };
        HarnessError::new(ErrorCode::InstallFailed, false).with_param("reason", reason)
    }
}

/// The shim that ships with the running app, found next to its executable.
pub fn bundled_shim_exe() -> Result<PathBuf, ShimPathError> {
    let exe = std::env::current_exe().map_err(|_| ShimPathError::Missing)?;
    shim_exe_path(exe.parent().ok_or(ShimPathError::Missing)?)
}

/// The shim next to the app executable, checked to be a plausible executable.
pub fn shim_exe_path(exe_dir: &Path) -> Result<PathBuf, ShimPathError> {
    let path = exe_dir.join(SHIM_FILE);
    let meta = std::fs::metadata(&path).map_err(|_| ShimPathError::Missing)?;
    if !meta.is_file() {
        return Err(ShimPathError::Invalid);
    }
    if meta.len() < MIN_BYTES || !starts_like_an_executable(&path) {
        return Err(ShimPathError::Invalid);
    }
    Ok(path)
}

/// Windows executables start with "MZ". Other platforms are not shipped, so only size is checked.
fn starts_like_an_executable(path: &Path) -> bool {
    if !cfg!(windows) {
        return true;
    }
    let mut magic = [0_u8; 2];
    File::open(path).and_then(|mut file| file.read_exact(&mut magic)).is_ok() && &magic == b"MZ"
}
