//! What a collector receives from the outside besides its own settings. Passing the clock and the
//! directories in keeps collectors deterministic and testable against temp dirs.

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Ctx {
    /// Epoch seconds "now". Collectors never read the system clock themselves.
    pub now_secs: i64,
    /// App-owned data directory (`%LOCALAPPDATA%\dev.aisessionmonitor.tray`); the Claude shim
    /// writes here.
    pub data_dir: PathBuf,
    /// The user's home directory (`%USERPROFILE%`). Tests point it at a temp dir.
    pub home_dir: PathBuf,
}

impl Ctx {
    pub fn new(now_secs: i64, data_dir: impl Into<PathBuf>, home_dir: impl Into<PathBuf>) -> Self {
        Self { now_secs, data_dir: data_dir.into(), home_dir: home_dir.into() }
    }
}
