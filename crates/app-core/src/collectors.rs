//! The seams between the scheduler and the outside. `app-core` knows only these traits; the shell
//! (composition root) implements them with the real collector crates, tests with fakes.

use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, HarnessError, HarnessId, HarnessView, Settings, ShimStatus};

use crate::secrets::SecretStore;

pub type CollectFuture<'a> = Pin<Box<dyn Future<Output = HarnessView> + Send + 'a>>;

pub trait Collector: Send + Sync {
    fn id(&self) -> HarnessId;

    /// One read of the harness. Never panics, never blocks the runtime; failures come back as a
    /// `HarnessView` in the `Error` state.
    fn collect<'a>(
        &'a self,
        settings: &'a Settings,
        secrets: &'a dyn SecretStore,
        ctx: &'a Ctx,
    ) -> CollectFuture<'a>;
}

/// Claude Code settings.json management (statusLine shim). Synchronous: small local files.
pub trait ClaudeShim: Send + Sync {
    fn status(&self, settings: &ClaudeSettings, ctx: &Ctx) -> ShimStatus;
    fn install(
        &self,
        settings: &ClaudeSettings,
        ctx: &Ctx,
        shim_exe: &Path,
    ) -> Result<ShimStatus, HarnessError>;
    fn uninstall(&self, settings: &ClaudeSettings, ctx: &Ctx) -> Result<ShimStatus, HarnessError>;

    /// Brings the installed copy of the shim up to `shim_exe`, the bundled build, and touches
    /// nothing else. Default: there is no copy to refresh.
    fn refresh_copy(
        &self,
        _settings: &ClaudeSettings,
        _ctx: &Ctx,
        _shim_exe: &Path,
    ) -> Result<(), HarnessError> {
        Ok(())
    }
}

/// Everything `AppService` needs from the collector side.
pub struct Adapters {
    /// One per harness, in tab order.
    pub collectors: Vec<Arc<dyn Collector>>,
    pub claude_shim: Arc<dyn ClaudeShim>,
}
