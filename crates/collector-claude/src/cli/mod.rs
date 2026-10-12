//! The preferred source of the per-model weekly windows (Opus, Sonnet, Fable, ...): the Claude
//! Code CLI's own `claude -p "/usage"`, which prints the account's limits as local text without a
//! model call (cost 0). The CLI authenticates itself, so this path never reads a token. It runs
//! under the same opt-in as the OAuth source, which stays as the fallback when the CLI is absent
//! or fails.
//!
//! Each run starts a process (about five seconds of CPU, the user's global hooks may fire), so the
//! answer is cached for ten minutes, a failure for thirty, and runs never overlap.
//!
//! `Cli` is the seam: production runs the fixed binary ([`SystemCli`]); tests hand the collector a
//! scripted double or a throwaway script in a temp dir.

mod cache;
mod parse;
mod system;

use std::future::Future;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;
use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, HarnessError, HarnessView, UsageWindow};

pub(crate) use system::SystemCli;

use crate::oauth::{blocking, bounded, weekly};
use crate::paths::Paths;

/// `view.meta` value of `extra_source` when the windows come from this source.
pub(crate) const META_VALUE: &str = "cli_usage";
const WEEKLY_ID: &str = "weekly";

/// Why there is no answer from the CLI.
#[derive(Clone, Debug)]
pub(crate) enum CliError {
    /// Not installed at a fixed location: the normal state of a machine without the CLI.
    Absent,
    /// Installed, but it did not give a usable answer.
    Failed(HarnessError),
}

pub(crate) trait Cli: Send + Sync {
    /// The parsed JSON envelope `claude -p "/usage" --output-format json` printed.
    /// `config_dir` becomes `CLAUDE_CONFIG_DIR` for the child when the user configured one.
    fn usage(
        &self,
        ctx: &Ctx,
        config_dir: Option<&Path>,
    ) -> impl Future<Output = Result<Value, CliError>> + Send;
}

/// One per-model weekly line. The reset text is localized, so only whether it equals the
/// "all models" line's text is kept: then the statusLine's epoch applies.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModelRow {
    pub label: String,
    pub used_pct: Option<f64>,
    pub same_reset: bool,
}

/// Two polls (a tick and a manual refresh) must not start two processes.
static RUN_LOCK: Mutex<()> = Mutex::const_new(());

/// Adds the per-model windows to a usable statusLine view. `true` when the CLI answered (even with
/// no per-model line); `false` when the caller should fall back to the OAuth source.
pub(crate) async fn add_extras<C: Cli>(
    view: &mut HarnessView,
    settings: &ClaudeSettings,
    ctx: &Ctx,
    cli: &C,
) -> bool {
    let Some(paths) = Paths::resolve(settings, ctx) else { return false };
    let _one_at_a_time = RUN_LOCK.lock().await;
    let Some(Ok(rows)) = rows(&paths, settings, ctx, cli).await else { return false };
    let epoch = view.windows.iter().find(|w| w.id == WEEKLY_ID).and_then(|w| w.resets_at);
    let windows = windows(&rows, epoch);
    if !windows.is_empty() {
        view.windows.extend(windows);
        view.meta.insert("extra_source".to_owned(), META_VALUE.to_owned());
    }
    true
}

/// `None` when there is nothing to use and nothing to cache (CLI absent).
async fn rows<C: Cli>(
    paths: &Paths,
    settings: &ClaudeSettings,
    ctx: &Ctx,
    cli: &C,
) -> Option<Result<Vec<ModelRow>, HarnessError>> {
    let (path, now) = (paths.cli_cache.clone(), ctx.now_secs);
    if let Some(hit) = blocking(move || cache::fresh(&path, now)).await.flatten() {
        return Some(hit);
    }
    let config_dir = settings.config_dir.as_deref().map(str::trim).filter(|d| !d.is_empty());
    let outcome = match cli.usage(ctx, config_dir.map(Path::new)).await {
        Err(CliError::Absent) => return None,
        Err(CliError::Failed(error)) => Err(error),
        Ok(envelope) => parse::envelope(&envelope),
    };
    let (path, stored) = (paths.cli_cache.clone(), outcome.clone());
    blocking(move || cache::store(&path, now, &stored)).await;
    Some(outcome)
}

fn windows(rows: &[ModelRow], weekly_epoch: Option<i64>) -> Vec<UsageWindow> {
    let mut found: Vec<UsageWindow> = Vec::new();
    for row in rows {
        let resets_at = if row.same_reset { weekly_epoch } else { None };
        let window = weekly(&row.label, row.used_pct, resets_at);
        match found.iter_mut().find(|f| f.id == window.id) {
            Some(existing) if window.used_pct > existing.used_pct => *existing = window,
            Some(_) => {}
            None => found.push(window),
        }
    }
    bounded(found)
}

#[cfg(test)]
mod tests;
