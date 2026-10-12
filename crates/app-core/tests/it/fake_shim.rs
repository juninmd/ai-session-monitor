//! A `ClaudeShim` that records its calls and answers from a script.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use app_core::ClaudeShim;
use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, HarnessError, ShimState, ShimStatus};

use crate::collector::lock;

#[derive(Clone, Debug)]
pub struct ShimCall {
    pub op: &'static str,
    pub claude: ClaudeSettings,
    pub data_dir: PathBuf,
    pub home_dir: PathBuf,
    pub now_secs: i64,
    pub exe: Option<PathBuf>,
}

pub struct FakeShim {
    log: Mutex<Vec<ShimCall>>,
    answer: Result<ShimStatus, HarnessError>,
}

pub fn installed() -> ShimStatus {
    ShimStatus { state: ShimState::Installed, chained: false, settings_path: None }
}

impl FakeShim {
    pub fn answering(answer: Result<ShimStatus, HarnessError>) -> Arc<Self> {
        Arc::new(Self { log: Mutex::new(Vec::new()), answer })
    }

    pub fn calls(&self) -> Vec<ShimCall> {
        lock(&self.log).clone()
    }

    fn record(&self, op: &'static str, claude: &ClaudeSettings, ctx: &Ctx, exe: Option<&Path>) {
        lock(&self.log).push(ShimCall {
            op,
            claude: claude.clone(),
            data_dir: ctx.data_dir.clone(),
            home_dir: ctx.home_dir.clone(),
            now_secs: ctx.now_secs,
            exe: exe.map(Path::to_path_buf),
        });
    }
}

impl ClaudeShim for FakeShim {
    fn status(&self, settings: &ClaudeSettings, ctx: &Ctx) -> ShimStatus {
        self.record("status", settings, ctx, None);
        let unreadable = ShimStatus { state: ShimState::Unreadable, chained: false, ..installed() };
        self.answer.clone().unwrap_or(unreadable)
    }

    fn install(
        &self,
        settings: &ClaudeSettings,
        ctx: &Ctx,
        shim_exe: &Path,
    ) -> Result<ShimStatus, HarnessError> {
        self.record("install", settings, ctx, Some(shim_exe));
        self.answer.clone()
    }

    fn uninstall(&self, settings: &ClaudeSettings, ctx: &Ctx) -> Result<ShimStatus, HarnessError> {
        self.record("uninstall", settings, ctx, None);
        self.answer.clone()
    }

    fn refresh_copy(
        &self,
        settings: &ClaudeSettings,
        ctx: &Ctx,
        shim_exe: &Path,
    ) -> Result<(), HarnessError> {
        self.record("refresh_copy", settings, ctx, Some(shim_exe));
        self.answer.clone().map(|_| ())
    }
}
