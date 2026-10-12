//! The application service: the single object behind every IPC command. The shell calls these
//! methods from its `#[tauri::command]` handlers and implements `EventSink` to push changes to the
//! webview, the tray icon and the desktop notifications.

use std::fmt;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use usage_model::history::MAX_EPOCHS_PER_WINDOW;
use usage_model::{
    HarnessError, HarnessHistory, HarnessId, SecretKind, SecretStatus, Settings, ShimStatus,
    Snapshot,
};

use crate::clock::{Clock, SystemClock};
use crate::collectors::Adapters;
use crate::notify::Notification;
use crate::paths::AppPaths;
use crate::scheduler::Scheduler;
use crate::secrets::SecretStore;
use crate::shared::Shared;
use crate::validate::{exceeds_chars, invalid_input, validate};

mod binding;
use binding::moves_the_key;

/// Longest secret accepted, in characters; real API keys are a tenth of that.
const MAX_SECRET_CHARS: usize = 4096;

/// Outbound events. Called from the scheduler task, so implementations must be quick and
/// non-blocking; they must not call back into `AppService` synchronously.
pub trait EventSink: Send + Sync + 'static {
    fn snapshot_changed(&self, snapshot: &Snapshot);
    fn settings_changed(&self, settings: &Settings);
    fn notify(&self, notification: &Notification);
}

pub struct AppService {
    shared: Arc<Shared>,
}

impl fmt::Debug for AppService {
    /// Shows nothing: the service holds settings and a secret store.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppService").finish_non_exhaustive()
    }
}

impl AppService {
    /// Loads settings from disk. A missing file means defaults; an unusable one is set aside as
    /// `settings.corrupt-<epoch>.json`, defaults apply and `run` announces it with a
    /// notification. A file another process holds is retried for up to 1.5 s before it counts as
    /// unusable. Fails with `SettingsUnreadable` only when the unusable file cannot be set aside.
    /// Does not start polling: spawn `run` on the async runtime for that.
    pub fn new(
        paths: AppPaths,
        secrets: Arc<dyn SecretStore>,
        sink: Arc<dyn EventSink>,
        adapters: Adapters,
    ) -> Result<Arc<Self>, HarnessError> {
        Self::with_clock(paths, secrets, sink, adapters, Arc::new(SystemClock))
    }

    /// `new` with an injected clock, for tests and simulations.
    pub fn with_clock(
        paths: AppPaths,
        secrets: Arc<dyn SecretStore>,
        sink: Arc<dyn EventSink>,
        adapters: Adapters,
        clock: Arc<dyn Clock>,
    ) -> Result<Arc<Self>, HarnessError> {
        let shared = Shared::new(paths, secrets, sink, adapters, clock)?;
        Ok(Arc::new(Self { shared: Arc::new(shared) }))
    }

    /// The scheduler loop. Resolves after `shutdown`. A call made while another loop is running
    /// returns at once. If the loop dies (a panic in the sink, say) or its task is cancelled, the
    /// next call starts a fresh one, so the shell can supervise it by spawning `run` again;
    /// `shutdown` is final, and a run after it returns at once.
    pub async fn run(self: Arc<Self>) {
        let Some(_claim) = self.shared.claim_run() else { return };
        Scheduler::new(Arc::clone(&self.shared)).run().await;
    }

    /// Ends `run` promptly and cancels the collections in flight.
    pub fn shutdown(&self) {
        self.shared.request_stop();
    }

    /// `shutdown`, then waits until the loop has unwound (collections dropped, child processes
    /// killed), at most `grace`. For the shell's exit path, where the process ends right after.
    /// `false` when the wait ran out.
    pub async fn shutdown_and_wait(&self, grace: Duration) -> bool {
        self.shutdown();
        let unwound = async {
            while self.shared.is_running() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        };
        tokio::time::timeout(grace, unwound).await.is_ok()
    }

    /// Current views in tab order; never waits for a collector.
    pub fn snapshot(&self) -> Snapshot {
        let now_secs = self.shared.clock.now_secs();
        Snapshot {
            generated_at: now_secs,
            views: self.shared.with_hub(|hub| hub.views_at(now_secs)),
        }
    }

    /// Non-blocking: asks the scheduler to collect now. Rate-limited per harness; a refused
    /// request is dropped without an error.
    pub fn refresh(&self, harness: Option<HarnessId>) {
        self.shared.request_refresh(harness);
    }

    pub fn settings(&self) -> Settings {
        self.shared.settings().as_ref().clone()
    }

    /// Validates, normalizes, persists atomically, applies (interval, enabled harnesses) and emits
    /// `settings_changed`. Resolves to the normalized settings. A rejected value changes nothing.
    /// Moving the LiteLLM connection (server, header, certificate trust) drops its stored key
    /// first; when that fails, nothing is saved.
    pub fn save_settings(&self, mut settings: Settings) -> Result<Settings, HarnessError> {
        validate(&settings)?;
        settings.normalize();
        // Before the write: a failed write loses the key but never pairs it with the new server.
        if moves_the_key(&self.shared.settings().litellm, &settings.litellm) {
            self.shared.secrets.delete(SecretKind::Litellm)?;
        }
        let saved = Arc::new(settings.clone());
        self.shared.store.save_then(&settings, || self.shared.apply_settings(saved))?;
        // Reads the latest saved settings at delivery time, so racing saves still end on the last.
        self.shared.relay.trigger(|| self.shared.sink.settings_changed(&self.settings()));
        Ok(settings)
    }

    pub fn secret_status(&self) -> Result<SecretStatus, HarnessError> {
        Ok(SecretStatus {
            litellm: self.exists(SecretKind::Litellm)?,
            opencode: self.exists(SecretKind::Opencode)?,
            kimi: self.exists(SecretKind::Kimi)?,
            zai: self.exists(SecretKind::Zai)?,
            openrouter: self.exists(SecretKind::Openrouter)?,
            deepseek: self.exists(SecretKind::Deepseek)?,
        })
    }

    fn exists(&self, kind: SecretKind) -> Result<bool, HarnessError> {
        self.shared.secrets.get(kind).map(|secret| secret.is_some())
    }

    /// The write is done, so a store that cannot be read back must not report it as failed: the
    /// written kind is known, and the other one counts as unset when unreadable.
    fn status_after_write(&self, written: SecretKind, present: bool) -> SecretStatus {
        let has = |kind| if kind == written { present } else { self.exists(kind).unwrap_or(false) };
        SecretStatus {
            litellm: has(SecretKind::Litellm),
            opencode: has(SecretKind::Opencode),
            kimi: has(SecretKind::Kimi),
            zai: has(SecretKind::Zai),
            openrouter: has(SecretKind::Openrouter),
            deepseek: has(SecretKind::Deepseek),
        }
    }

    /// Stores the trimmed value and re-reads that harness. The value is never logged or echoed.
    pub fn set_secret(
        &self,
        kind: SecretKind,
        value: String,
    ) -> Result<SecretStatus, HarnessError> {
        let value = value.trim();
        if value.is_empty() || exceeds_chars(value, MAX_SECRET_CHARS) {
            return Err(invalid_input("value"));
        }
        self.shared.secrets.set(kind, value)?;
        self.shared.request_forced(harness_of(kind));
        Ok(self.status_after_write(kind, true))
    }

    /// Deleting a secret that does not exist is not an error.
    pub fn clear_secret(&self, kind: SecretKind) -> Result<SecretStatus, HarnessError> {
        self.shared.secrets.delete(kind)?;
        self.shared.request_forced(harness_of(kind));
        Ok(self.status_after_write(kind, false))
    }

    /// The newest `limit` (default and maximum 8) past epochs of each window of `harness`. Empty
    /// while the history is switched off; what was stored stays until `clear_history`.
    pub fn history(
        &self,
        harness: HarnessId,
        limit: Option<u8>,
    ) -> Result<HarnessHistory, HarnessError> {
        let limit = limit.unwrap_or(MAX_EPOCHS_PER_WINDOW);
        if limit == 0 || limit > MAX_EPOCHS_PER_WINDOW {
            return Err(invalid_input("limit"));
        }
        if !self.shared.settings().general.history_enabled {
            return Ok(HarnessHistory::empty(harness));
        }
        Ok(self.shared.history.query(harness, limit))
    }

    /// Forgets the stored history, in memory and on disk. Blocking: it touches the disk.
    pub fn clear_history(&self) -> Result<(), HarnessError> {
        self.shared.history.clear(self.shared.clock.now_secs())
    }

    pub fn claude_shim_status(&self) -> ShimStatus {
        self.shared.shim.status(&self.shared.settings().claude, &self.shared.ctx())
    }

    /// `shim_exe` is the bundled statusline-shim executable resolved by the shell. Needs the
    /// user's consent in the settings.
    pub fn install_claude_shim(&self, shim_exe: &Path) -> Result<ShimStatus, HarnessError> {
        let settings = self.shared.settings();
        if !settings.claude.shim_consent {
            return Err(invalid_input("shimConsent"));
        }
        let status = self.shared.shim.install(&settings.claude, &self.shared.ctx(), shim_exe)?;
        self.shared.request_forced(HarnessId::Claude);
        Ok(status)
    }

    /// Startup step for the shell, once per launch: only an install writes the copy of the shim
    /// that Claude Code runs, so an app update would leave it stale. `shim_exe` is the bundled
    /// build, as for `install_claude_shim`. Does nothing, quietly, without the user's consent. A
    /// failure (for example a copy Claude Code is running) changes nothing: the shell logs it and
    /// the next launch tries again.
    pub fn refresh_claude_shim_copy(&self, shim_exe: &Path) -> Result<(), HarnessError> {
        let settings = self.shared.settings();
        if !settings.claude.shim_consent {
            return Ok(());
        }
        self.shared.shim.refresh_copy(&settings.claude, &self.shared.ctx(), shim_exe)
    }

    pub fn uninstall_claude_shim(&self) -> Result<ShimStatus, HarnessError> {
        let settings = self.shared.settings();
        let status = self.shared.shim.uninstall(&settings.claude, &self.shared.ctx())?;
        self.shared.request_forced(HarnessId::Claude);
        Ok(status)
    }
}

fn harness_of(kind: SecretKind) -> HarnessId {
    match kind {
        SecretKind::Litellm => HarnessId::Litellm,
        SecretKind::Opencode => HarnessId::Opencode,
        SecretKind::Kimi => HarnessId::Kimi,
        SecretKind::Zai => HarnessId::Zai,
        SecretKind::Openrouter => HarnessId::Openrouter,
        SecretKind::Deepseek => HarnessId::Deepseek,
    }
}
