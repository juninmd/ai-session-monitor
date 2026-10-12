//! Where the app keeps its files, plus the debug-only switches smoke runs use to stay away from
//! the real profile, credential store and autostart entry.

use std::path::PathBuf;

use app_core::AppPaths;

#[cfg(test)]
mod tests;

#[cfg(any(debug_assertions, test))]
const DATA_DIR: &str = "AISM_DATA_DIR";
#[cfg(any(debug_assertions, test))]
const HOME_DIR: &str = "AISM_HOME_DIR";
#[cfg(any(debug_assertions, test))]
const MEMORY_SECRETS: &str = "AISM_MEMORY_SECRETS";
#[cfg(any(debug_assertions, test))]
const SHOW_ON_START: &str = "AISM_SHOW_ON_START";

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("{0} must be an absolute path")]
pub struct OverrideError(&'static str);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Overrides {
    pub data_dir: Option<PathBuf>,
    pub home_dir: Option<PathBuf>,
    /// Keep secrets in memory instead of the Credential Manager.
    pub memory_secrets: bool,
    /// Open the flyout right after start.
    pub show_on_start: bool,
}

// The lookup is compiled out of release builds, so no variable can redirect a shipped app.
#[cfg(any(debug_assertions, test))]
impl Overrides {
    /// Reads the `AISM_*` switches through `lookup`. A path that is set but not absolute is an
    /// error rather than a silent fallback to the real profile.
    pub fn from_lookup(
        lookup: impl Fn(&str) -> Option<std::ffi::OsString>,
    ) -> Result<Self, OverrideError> {
        let dir = |name: &'static str| -> Result<Option<PathBuf>, OverrideError> {
            match lookup(name) {
                None => Ok(None),
                Some(raw) => {
                    let path = PathBuf::from(raw);
                    if path.is_absolute() { Ok(Some(path)) } else { Err(OverrideError(name)) }
                }
            }
        };
        let flag = |name: &str| lookup(name).is_some_and(|value| value == "1");
        Ok(Self {
            data_dir: dir(DATA_DIR)?,
            home_dir: dir(HOME_DIR)?,
            memory_secrets: flag(MEMORY_SECRETS),
            show_on_start: flag(SHOW_ON_START),
        })
    }
}

impl Overrides {
    /// Debug builds read the process environment; release builds always get the defaults.
    pub fn from_env() -> Result<Self, OverrideError> {
        #[cfg(debug_assertions)]
        {
            Self::from_lookup(|name| std::env::var_os(name))
        }
        #[cfg(not(debug_assertions))]
        {
            Ok(Self::default())
        }
    }

    /// Any switch that makes this a test run: autostart and the real profile stay untouched.
    pub fn is_active(&self) -> bool {
        self != &Self::default()
    }

    /// A test run never reaches the real Credential Manager, whichever switch made it one.
    pub fn uses_memory_secrets(&self) -> bool {
        self.is_active()
    }

    /// Where a test run keeps its files. Overriding only the home directory still moves the data
    /// directory, so half of a test profile can never fall through to the real one.
    pub fn data_root(&self) -> Option<PathBuf> {
        self.data_dir.clone().or_else(|| self.home_dir.as_ref().map(|home| home.join("aism-data")))
    }

    /// The home directory a test run reads other tools' state from; see `data_root`.
    pub fn home_root(&self) -> Option<PathBuf> {
        self.home_dir.clone().or_else(|| self.data_dir.as_ref().map(|data| data.join("home")))
    }

    pub fn resolve(&self, data_dir: PathBuf, home_dir: PathBuf) -> AppPaths {
        AppPaths {
            data_dir: self.data_root().unwrap_or(data_dir),
            home_dir: self.home_root().unwrap_or(home_dir),
        }
    }
}
