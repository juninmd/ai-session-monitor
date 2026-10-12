//! Composition root for the collector side: the adapters `AppService` is built with. With the
//! `collectors` feature they are the real collector crates; without it the shell still builds and
//! runs, every tab simply has nothing to read and installing the Claude shim reports a failure.

#[cfg(not(feature = "collectors"))]
use std::path::Path;
#[cfg(not(feature = "collectors"))]
use std::sync::Arc;

use app_core::Adapters;
#[cfg(not(feature = "collectors"))]
use app_core::ClaudeShim;
#[cfg(not(feature = "collectors"))]
use usage_model::settings::ClaudeSettings;
#[cfg(not(feature = "collectors"))]
use usage_model::{Ctx, ErrorCode, HarnessError, ShimState, ShimStatus};

#[cfg(test)]
mod tests;

/// `claude_dir` is the folder `CLAUDE_CONFIG_DIR` named at startup, if any (see `claude_dir`).
pub fn adapters(claude_dir: Option<String>) -> Adapters {
    #[cfg(feature = "collectors")]
    {
        crate::wiring_collectors::adapters(claude_dir)
    }
    #[cfg(not(feature = "collectors"))]
    {
        let _ = claude_dir; // there is no Claude collector to hand it to
        Adapters { collectors: Vec::new(), claude_shim: Arc::new(UnavailableShim) }
    }
}

/// Stands in for the Claude shim installer when the collectors are compiled out.
#[cfg(not(feature = "collectors"))]
struct UnavailableShim;

#[cfg(not(feature = "collectors"))]
impl ClaudeShim for UnavailableShim {
    fn status(&self, _settings: &ClaudeSettings, _ctx: &Ctx) -> ShimStatus {
        ShimStatus { state: ShimState::NotInstalled, chained: false, settings_path: None }
    }

    fn install(
        &self,
        _settings: &ClaudeSettings,
        _ctx: &Ctx,
        _shim_exe: &Path,
    ) -> Result<ShimStatus, HarnessError> {
        Err(HarnessError::new(ErrorCode::InstallFailed, false))
    }

    fn uninstall(
        &self,
        _settings: &ClaudeSettings,
        _ctx: &Ctx,
    ) -> Result<ShimStatus, HarnessError> {
        Err(HarnessError::new(ErrorCode::InstallFailed, false))
    }
}
