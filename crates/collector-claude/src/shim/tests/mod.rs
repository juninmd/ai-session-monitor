//! The settings.json management against temp dirs, through the public functions. The races use
//! the crate-private `Hooks` seam; everything else is what a caller of the crate can do.

mod data_wipe;
mod install_flow;
mod profiles;
mod races;
mod refresh;
mod safety;
mod status;
mod uninstall_flow;

use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use std::path::Path;

use serde_json::Value;
use usage_model::{HarnessError, ShimStatus};

use crate::paths::EXE_NAME;
use crate::testenv::{Env, files_under};

fn install(env: &Env) -> Result<ShimStatus, HarnessError> {
    super::install_shim(&env.settings(), &env.ctx(), &env.shim_source())
}

fn uninstall(env: &Env) -> Result<ShimStatus, HarnessError> {
    super::uninstall_shim(&env.settings(), &env.ctx())
}

fn status(env: &Env) -> ShimStatus {
    super::shim_status(&env.settings(), &env.ctx())
}

fn parse(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

/// The settings.json as JSON.
fn json(env: &Env) -> Value {
    read_json(&env.settings_json())
}

fn status_line(env: &Env) -> Value {
    json(env)["statusLine"].clone()
}

fn command_of(env: &Env) -> String {
    status_line(env)["command"].as_str().unwrap().to_owned()
}

fn keys(value: &Value) -> Vec<String> {
    value.as_object().unwrap().keys().cloned().collect()
}

fn reason(error: &HarnessError) -> &str {
    error.params["reason"].as_str()
}

type Snapshot = BTreeMap<String, (usize, u64)>;

/// Every file under the temp root as (size, checksum): "nothing changed anywhere" in one
/// comparison whose failure message stays readable.
fn snapshot(env: &Env) -> Snapshot {
    files_under(env.root())
        .into_iter()
        .map(|name| {
            let bytes = std::fs::read(env.root().join(&name)).unwrap();
            let mut hasher = DefaultHasher::new();
            bytes.hash(&mut hasher);
            (name, (bytes.len(), hasher.finish()))
        })
        .collect()
}

fn assert_unchanged(env: &Env, before: &Snapshot, what: &str) {
    let now = snapshot(env);
    let names: BTreeSet<&String> = now.keys().chain(before.keys()).collect();
    let differing: Vec<_> =
        names.into_iter().filter(|name| now.get(*name) != before.get(*name)).collect();
    assert!(differing.is_empty(), "{what}: files that differ: {differing:?}");
}

/// What `install` writes for a home-relative data dir (the default test layout), with the shim
/// recording to the same place Claude Code's collector reads.
fn command(chained: bool) -> String {
    let base = "~/AppData/Local/ai-session-monitor";
    let mut command = format!("{base}/bin/{EXE_NAME} --out {base}/claude/rate_limits.json");
    if chained {
        command.push_str(&format!(" --chain {base}/claude/chain.json"));
    }
    command
}
