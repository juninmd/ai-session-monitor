//! Where Cursor keeps `state.vscdb`. Derived from the home directory the shell passes in, so a
//! smoke run with `AISM_HOME_DIR` never reaches the real profile (Linux honours
//! `XDG_CONFIG_HOME`, as Electron does).

use std::path::{Path, PathBuf};

const RELATIVE: [&str; 4] = ["Cursor", "User", "globalStorage", "state.vscdb"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Os {
    Windows,
    Mac,
    Linux,
}

impl Os {
    pub(crate) const fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Mac
        } else {
            Self::Linux
        }
    }
}

/// `%APPDATA%\Cursor\...` (the roaming profile under the home directory), `~/Library/Application
/// Support/Cursor/...`, or `$XDG_CONFIG_HOME` (when absolute) / `~/.config` + `/Cursor/...`.
pub(crate) fn state_db(os: Os, home: &Path, xdg_config_home: Option<&Path>) -> PathBuf {
    let base = match os {
        Os::Windows => home.join("AppData").join("Roaming"),
        Os::Mac => home.join("Library").join("Application Support"),
        Os::Linux => match xdg_config_home.filter(|dir| dir.is_absolute()) {
            Some(dir) => dir.to_path_buf(),
            None => home.join(".config"),
        },
    };
    RELATIVE.iter().fold(base, |path, part| path.join(part))
}

pub(crate) fn default_state_db(home: &Path) -> PathBuf {
    let xdg = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);
    state_db(Os::current(), home, xdg.as_deref())
}
