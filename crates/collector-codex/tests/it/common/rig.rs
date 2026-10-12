//! One fake `codex app-server` in a directory of its own, plus a fabricated home directory.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use serde_json::Value;
use tempfile::TempDir;
use usage_model::settings::CodexSettings;
use usage_model::{Ctx, HarnessView};

use super::process::read_pid;

/// 2026-10-10T20:00:00Z, the clock every test runs on.
pub const NOW: i64 = 1_791_662_400;
/// The fake's reset times: `NOW + 8100` s and `NOW + 349200` s.
pub const FIVE_HOUR_RESET: i64 = 1_791_670_500;
pub const WEEKLY_RESET: i64 = 1_792_011_600;
/// Room for a slow first start of a freshly copied executable.
pub const PATIENT: Duration = Duration::from_secs(60);

/// Always `.exe`, also off Windows: the binary lookup only accepts names that look runnable, and
/// on Unix the extension is ignored when the file is started.
const FAKE_NAME: &str = "fake_app_server.exe";

pub struct Rig {
    /// Holds the fake executable, its `fake-mode` file and whatever it writes.
    pub bin: TempDir,
    pub exe: PathBuf,
    /// Stands in for the user's home directory.
    pub home: TempDir,
}

impl Rig {
    pub fn new(mode: &str) -> Self {
        Self::in_dir(mode, tempfile::tempdir().unwrap())
    }

    /// For paths that contain spaces and non-ASCII characters.
    pub fn named(mode: &str, prefix: &str) -> Self {
        Self::in_dir(mode, tempfile::Builder::new().prefix(prefix).tempdir().unwrap())
    }

    /// No program at all: `binary_path` names a file that does not exist.
    pub fn without_binary() -> Self {
        let bin = tempfile::tempdir().unwrap();
        let exe = bin.path().join("absent.exe");
        Self { bin, exe, home: tempfile::tempdir().unwrap() }
    }

    fn in_dir(mode: &str, bin: TempDir) -> Self {
        let exe = bin.path().join(FAKE_NAME);
        fs::copy(env!("CARGO_BIN_EXE_fake_app_server"), &exe).unwrap();
        fs::write(bin.path().join("fake-mode"), mode).unwrap();
        // A first run can be held up by a virus scanner; take that delay here, not in a budget.
        let mut warm_up = Command::new(&exe);
        warm_up.arg("warm-up").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        let _ = warm_up.status();
        Self { bin, exe, home: tempfile::tempdir().unwrap() }
    }

    pub fn settings(&self) -> CodexSettings {
        let binary_path = Some(self.exe.to_string_lossy().into_owned());
        CodexSettings { enabled: true, consent: true, binary_path }
    }

    pub fn ctx_at(&self, now_secs: i64) -> Ctx {
        Ctx::new(now_secs, self.home.path().join("data"), self.home.path())
    }

    pub async fn collect(&self) -> HarnessView {
        self.collect_within(PATIENT).await
    }

    pub async fn collect_within(&self, budget: Duration) -> HarnessView {
        collector_codex::collect_with_budget(&self.settings(), &self.ctx_at(NOW), budget).await
    }

    pub async fn collect_at(&self, now_secs: i64) -> HarnessView {
        collector_codex::collect_with_budget(&self.settings(), &self.ctx_at(now_secs), PATIENT)
            .await
    }

    pub async fn collect_with(&self, settings: &CodexSettings) -> HarnessView {
        collector_codex::collect_with_budget(settings, &self.ctx_at(NOW), PATIENT).await
    }

    /// The fake's own pid, once it has started.
    pub fn pid(&self) -> Option<u32> {
        read_pid(&self.bin.path().join("fake-pid"))
    }

    /// The pid of the process the `grandchild` mode starts.
    pub fn child_pid(&self) -> Option<u32> {
        read_pid(&self.bin.path().join("fake-child-pid"))
    }

    /// Every JSON message the client sent, in order.
    pub fn received(&self) -> Vec<Value> {
        let log = fs::read_to_string(self.bin.path().join("fake-received.log"));
        log.unwrap_or_default().lines().filter_map(|line| serde_json::from_str(line).ok()).collect()
    }
}
