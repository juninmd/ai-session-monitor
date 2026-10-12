//! Shared support: the loopback fake proxy, throwaway certificates, and thin helpers that build
//! settings, run one poll inside a sandbox and inspect the resulting view.

mod reply;
mod seen;
mod server;
pub mod tls;

use std::collections::BTreeMap;
use std::fs;

use collector_litellm::collect;
use tempfile::TempDir;
use usage_model::settings::{LitellmScope, LitellmSettings};
use usage_model::{Ctx, HarnessError, HarnessId, HarnessView, Source, UsageWindow, ViewState};
use usage_support::Secret;

pub use reply::Reply;
pub use server::Fake;

/// 2026-10-11T12:00:00Z, the clock every test runs on.
pub const NOW: i64 = 1_791_720_000;
/// A fabricated credential; the leak checks look for exactly this text.
pub const SECRET: &str = "sk-fixture-7f3a9c1e5b2d4a60";
/// Planted in every fixture field that must never reach a view.
const DECOY: &str = "decoy";

/// Settings for the key scope with a fast timeout; tests override what they exercise.
pub fn settings(base: &str) -> LitellmSettings {
    LitellmSettings {
        base_url: Some(base.to_owned()),
        timeout_secs: 5,
        ..LitellmSettings::default()
    }
}

pub fn scoped(base: &str, scope: LitellmScope, id: &str) -> LitellmSettings {
    LitellmSettings { scope, entity_id: Some(id.to_owned()), ..settings(base) }
}

/// One poll with the fabricated key at the fixed clock.
pub async fn run(settings: &LitellmSettings) -> HarnessView {
    run_with(settings, Some(&Secret::new(SECRET)), NOW).await
}

/// One poll inside a sandbox. Whatever the outcome, the app's data and home directories must
/// still be empty and the view must carry neither the key nor any decoy planted in a fixture.
pub async fn run_with(settings: &LitellmSettings, key: Option<&Secret>, now: i64) -> HarnessView {
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let ctx = Ctx::new(now, data.path(), home.path());
    let view = collect(settings, key, &ctx).await;
    for (dir, what) in [(&data, "data"), (&home, "home")] {
        let leftovers = fs::read_dir(dir.path()).unwrap().count();
        assert_eq!(leftovers, 0, "the collector wrote into the {what} directory");
    }
    assert_eq!(view.id, HarnessId::Litellm);
    assert_clean(&view, key.map_or("", Secret::expose));
    view
}

/// Neither the credential nor fixture decoys may appear anywhere a view can carry text.
pub fn assert_clean(view: &HarnessView, key: &str) {
    // Trimmed: a proxy sees the key without edge spaces, and the padded form contains it anyway.
    let key = key.trim().to_lowercase();
    let shown = [serde_json::to_string(view).unwrap(), format!("{view:?}")];
    for text in shown.map(|text| text.to_lowercase()) {
        assert!(key.is_empty() || !text.contains(&key), "the key reached a view");
        assert!(!text.contains(DECOY), "server-side private data reached a view: {text}");
    }
}

/// The view of a poll that reached the proxy and read a record.
pub fn assert_ready(view: &HarnessView) {
    assert_eq!(view.state, ViewState::Ready, "{view:?}");
    assert_eq!(view.source, Some(Source { id: "key_info".to_owned(), official: true }));
    assert!(view.plan.is_none() && view.error.is_none());
}

/// Params an error may carry: short identifiers and numbers, never free text.
const SAFE_PARAMS: [&str; 6] =
    ["host", "httpStatus", "retryAfterSecs", "redirect", "reason", "field"];

/// The error of a failed poll; a failed poll shows nothing else and explains itself safely.
pub fn error_of(view: &HarnessView) -> &HarnessError {
    assert_eq!(view.state, ViewState::Error, "{view:?}");
    assert!(view.windows.is_empty() && view.facts.is_empty() && view.notices.is_empty());
    assert!(view.source.is_none() && !view.stale);
    let error = view.error.as_ref().expect("an error view carries its error");
    for (key, value) in &error.params {
        assert!(SAFE_PARAMS.contains(&key.as_str()), "unexpected error param {key}");
        assert!(value.chars().count() <= 64, "error param {key} is free text");
    }
    error
}

pub fn param<'a>(error: &'a HarnessError, key: &str) -> Option<&'a str> {
    error.params.get(key).map(String::as_str)
}

pub fn window<'a>(view: &'a HarnessView, id: &str) -> &'a UsageWindow {
    view.windows.iter().find(|w| w.id == id).unwrap_or_else(|| panic!("no window {id}: {view:?}"))
}

pub fn ids(view: &HarnessView) -> Vec<&str> {
    view.windows.iter().map(|w| w.id.as_str()).collect()
}

pub fn codes(view: &HarnessView) -> Vec<&str> {
    view.notices.iter().map(|n| n.code.as_str()).collect()
}

pub fn meta(view: &HarnessView) -> BTreeMap<&str, &str> {
    view.meta.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect()
}
