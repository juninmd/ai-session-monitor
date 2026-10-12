//! The CLI source with fabricated output only; no test ever runs a real `claude`, reads a real
//! profile or sends a token anywhere.

mod flow;
mod gates;
mod parse;
mod system;

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};
use usage_model::settings::ClaudeSettings;
use usage_model::{Ctx, HarnessView};

use super::{Cli, CliError};
use crate::oauth::Endpoint;
use crate::testenv::{Env, ok_both};

/// A fabricated answer shaped like the real one: header, limits block, blank line, then the
/// section whose names (and any look-alike line) must never reach a view.
pub(super) const REPORT: &str = "You are currently using your subscription to power your Claude \
Code usage\n\nCurrent session: 1% used · resets Oct 12, 1:40pm (America/Sao_Paulo)\n\
Current week (all models): 78% used · resets Oct 17, 8pm (America/Sao_Paulo)\n\
Current week (Fable): 0% used · resets Oct 17, 8pm (America/Sao_Paulo)\n\
Current week (Opus): 12.5% used · resets Oct 17, 8pm (America/Sao_Paulo)\n\
Current week (Sonnet): 34% used · resets Oct 18, 9pm (America/Sao_Paulo)\n\n\
What's contributing to your limits usage?\n\
Approximate, based on local sessions on this machine.\n\
Last 24h · 80 requests · plugin acme-secret-plugin · mcp acme-secret-server\n\
Current week (Injected): 99% used · resets Oct 17, 8pm (America/Sao_Paulo)\n";

pub(super) fn envelope(result: &str) -> Value {
    json!({
        "type": "result", "subtype": "success", "is_error": false, "num_turns": 0,
        "total_cost_usd": 0, "result": result, "local_command": "usage"
    })
}

/// Counts calls and remembers the config dir it was asked with.
pub(super) struct ScriptedCli {
    reply: Result<Value, CliError>,
    calls: AtomicUsize,
    config_dir: Mutex<Option<PathBuf>>,
}

impl ScriptedCli {
    pub(super) fn new(reply: Result<Value, CliError>) -> Self {
        Self { reply, calls: AtomicUsize::new(0), config_dir: Mutex::new(None) }
    }
    pub(super) fn says(result: &str) -> Self {
        Self::new(Ok(envelope(result)))
    }
    pub(super) fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
    pub(super) fn config_dir(&self) -> Option<PathBuf> {
        self.config_dir.lock().unwrap().clone()
    }
}

impl Cli for ScriptedCli {
    async fn usage(&self, _: &Ctx, config_dir: Option<&Path>) -> Result<Value, CliError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        *self.config_dir.lock().unwrap() = config_dir.map(Path::to_path_buf);
        self.reply.clone()
    }
}

/// Consented, shim installed, a fresh recording on disk (weekly reset at `WEEKLY_RESET`).
pub(super) fn installed_env() -> Env {
    let env = Env::new();
    crate::install_shim(&env.settings(), &env.ctx(), &env.shim_source()).unwrap();
    env.write_rate_limits(&ok_both());
    env
}

/// `resets_at` of the statusLine weekly window in the `ok_both` fixture.
pub(super) const WEEKLY_RESET: i64 = 1_800_400_000;

pub(super) fn on(env: &Env) -> ClaudeSettings {
    ClaudeSettings { oauth_usage: true, ..env.settings() }
}

/// An OAuth endpoint nobody listens on: tests that expect the CLI to answer must never need it.
pub(super) fn unused_endpoint() -> Endpoint {
    Endpoint {
        base: "http://127.0.0.1:9".to_owned(),
        allow_loopback_http: true,
        timeout: std::time::Duration::from_secs(2),
    }
}

pub(super) async fn poll(env: &Env, cli: &impl Cli, now: i64) -> HarnessView {
    poll_with(&on(env), env, cli, now).await
}

pub(super) async fn poll_with(
    settings: &ClaudeSettings,
    env: &Env,
    cli: &impl Cli,
    now: i64,
) -> HarnessView {
    crate::collect_with_cli(settings, &env.ctx_at(now), &unused_endpoint(), cli).await
}

pub(super) fn ids(view: &HarnessView) -> Vec<&str> {
    view.windows.iter().map(|w| w.id.as_str()).collect()
}
