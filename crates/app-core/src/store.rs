//! Settings persistence. Loading never trusts the file: unusable content is set aside, never
//! overwritten, and the caller is told so the user hears about it. Saving is atomic and serialized.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use std::{fs, io, thread};

use serde_json::Value;
use usage_model::{ErrorCode, HarnessError, Settings};
use usage_support::fsutil::{atomic_write, read_json_capped};

use crate::sync::lock;

/// Far above a real settings file; a bigger one is corrupt or hostile.
const MAX_SETTINGS_BYTES: usize = 256 * 1024;
/// A scanner, a backup tool or a logon storm can hold the file for a while; a false "corrupt"
/// would reset the user's settings. Waits grow by one step per attempt: 1.5 s in all.
const READ_ATTEMPTS: u32 = 6;
const READ_BACKOFF: Duration = Duration::from_millis(100);
const QUARANTINE_NAMES: u32 = 100;

pub(crate) struct SettingsStore {
    path: PathBuf,
    write: Mutex<()>,
}

pub(crate) struct Loaded {
    pub(crate) settings: Settings,
    /// File name of the set-aside copy when the settings file was unusable.
    pub(crate) recovered: Option<String>,
}

enum Read {
    Found(Box<Settings>),
    Missing,
    /// The bytes are there but are not a settings document.
    Corrupt,
    /// The bytes could not be read at all.
    Io,
}

fn unreadable() -> HarnessError {
    HarnessError::new(ErrorCode::SettingsUnreadable, false)
}

fn write_failed() -> HarnessError {
    HarnessError::new(ErrorCode::Internal, true).with_param("where", "settings_write")
}

fn defaults() -> Settings {
    let mut settings = Settings::default();
    settings.normalize();
    settings
}

impl SettingsStore {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self { path, write: Mutex::new(()) }
    }

    /// Missing file: defaults. Unusable file: moved aside, defaults, and `recovered` names the
    /// copy. Fails only when the unusable file cannot be moved aside, because carrying on would
    /// let the next save overwrite the user's only copy.
    pub(crate) fn load(&self, now_secs: i64) -> Result<Loaded, HarnessError> {
        self.load_with(now_secs, || self.read(), thread::sleep)
    }

    /// `load` with the read and the pause between attempts injected, so the retry rules can be
    /// tested without a file that fails on demand.
    fn load_with(
        &self,
        now_secs: i64,
        mut read: impl FnMut() -> Read,
        mut pause: impl FnMut(Duration),
    ) -> Result<Loaded, HarnessError> {
        for attempt in 1..=READ_ATTEMPTS {
            match read() {
                Read::Found(settings) => {
                    return Ok(Loaded { settings: *settings, recovered: None });
                }
                Read::Missing => return Ok(Loaded { settings: defaults(), recovered: None }),
                Read::Corrupt => break,
                Read::Io if attempt < READ_ATTEMPTS => pause(READ_BACKOFF * attempt),
                Read::Io => {}
            }
        }
        let aside = self.quarantine(now_secs).map_err(|_| unreadable())?;
        Ok(Loaded { settings: defaults(), recovered: Some(aside) })
    }

    /// Writes `settings` atomically, then runs `commit` before the write lock is released, so
    /// the file and the in-memory state change in the same order for concurrent callers.
    pub(crate) fn save_then(
        &self,
        settings: &Settings,
        commit: impl FnOnce(),
    ) -> Result<(), HarnessError> {
        let bytes = serde_json::to_vec_pretty(settings).map_err(|_| write_failed())?;
        let _writer = lock(&self.write);
        atomic_write(&self.path, &bytes).map_err(|_| write_failed())?;
        commit();
        Ok(())
    }

    fn read(&self) -> Read {
        match read_json_capped::<Value>(&self.path, MAX_SETTINGS_BYTES) {
            Ok(value) => parse(value),
            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => Read::Missing,
                io::ErrorKind::InvalidData | io::ErrorKind::FileTooLarge => Read::Corrupt,
                _ => Read::Io,
            },
        }
    }

    /// `settings.json` becomes `settings.corrupt-<epoch>.json`, with `-<n>` before the extension
    /// when that name is taken; an earlier copy is never replaced.
    fn quarantine(&self, now_secs: i64) -> io::Result<String> {
        let stem = self.path.file_stem().and_then(|s| s.to_str()).unwrap_or("settings");
        let extension = self.path.extension().and_then(|s| s.to_str());
        for n in 0..QUARANTINE_NAMES {
            let suffix = if n == 0 { String::new() } else { format!("-{n}") };
            let name = match extension {
                Some(ext) => format!("{stem}.corrupt-{now_secs}{suffix}.{ext}"),
                None => format!("{stem}.corrupt-{now_secs}{suffix}"),
            };
            let target = self.path.with_file_name(&name);
            if !target.exists() {
                fs::rename(&self.path, &target)?;
                return Ok(name);
            }
        }
        Err(io::Error::from(io::ErrorKind::AlreadyExists))
    }
}

fn parse(value: Value) -> Read {
    if !value.is_object() {
        return Read::Corrupt;
    }
    match serde_json::from_value::<Settings>(value) {
        Ok(mut settings) => {
            settings.normalize();
            Read::Found(Box::new(settings))
        }
        Err(_) => Read::Corrupt,
    }
}

#[cfg(test)]
mod retry_tests;
#[cfg(test)]
mod tests;
