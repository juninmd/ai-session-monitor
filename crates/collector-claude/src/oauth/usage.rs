//! `GET /api/oauth/usage`: the request, and the per-model weekly windows in its response.
//!
//! Two response shapes are understood, both from community reports (single origin, unverified
//! against a live account here): the older flat keys `seven_day_opus` / `seven_day_sonnet`
//! (`{utilization, resets_at}`, null when the account has no such cap) and the newer `limits[]`
//! whose `weekly_scoped` entries carry `{percent, resets_at, scope: {model: {display_name}}}`.
//! Percentages are read as 0..=100 and `resets_at` as RFC 3339. `five_hour` and `seven_day` are
//! ignored on purpose: the statusLine is the source for those.

use serde_json::Value;
use usage_model::{ErrorCode, HarnessError, UsageWindow, WindowKind};
use usage_support::{Auth, ClientOptions, RequestSpec, SafeClient, Secret, parse_rfc3339_secs};

use super::Endpoint;
use crate::datafile::WEEKLY_MINUTES;
use crate::lenient::{clean_text, percent, plausible, slug};

/// Claude Code sends the same header; the endpoint rejects requests without it.
const BETA_HEADER: &str = "oauth-2025-04-20";
const MAX_BODY_BYTES: usize = 1024 * 1024;
const MAX_EXTRA_WINDOWS: usize = 8;
const MAX_LIMIT_ENTRIES: usize = 32;
const MAX_LABEL_CHARS: usize = 40;
const KNOWN_KEYS: [&str; 6] =
    ["five_hour", "seven_day", "seven_day_opus", "seven_day_sonnet", "limits", "extra_usage"];

pub(crate) async fn fetch(endpoint: &Endpoint, token: &Secret) -> Result<Value, HarnessError> {
    let client = SafeClient::new(ClientOptions {
        timeout: endpoint.timeout,
        max_body_bytes: MAX_BODY_BYTES,
        allow_loopback_http: endpoint.allow_loopback_http,
        ..ClientOptions::default()
    })?;
    let request = RequestSpec::new(endpoint.base.as_str())
        .segment("api")
        .segment("oauth")
        .segment("usage")
        .header("anthropic-beta", BETA_HEADER)
        .auth(Auth::new(token));
    client.get_json(&request).await
}

/// The extra weekly windows of a response; empty when the account has none. A document with none
/// of the known keys is not this endpoint's shape any more.
pub(crate) fn parse(body: &Value) -> Result<Vec<UsageWindow>, HarnessError> {
    let object = body
        .as_object()
        .filter(|object| KNOWN_KEYS.iter().any(|key| object.contains_key(*key)))
        .ok_or_else(|| HarnessError::new(ErrorCode::EndpointChanged, false))?;
    let mut found: Vec<UsageWindow> = Vec::new();
    if let Some(Value::Array(limits)) = object.get("limits") {
        for entry in limits.iter().take(MAX_LIMIT_ENTRIES) {
            if let Some(window) = scoped(entry) {
                keep_the_higher(&mut found, window);
            }
        }
    }
    for (key, family) in [("seven_day_opus", "opus"), ("seven_day_sonnet", "sonnet")] {
        let flat = object.get(key).and_then(|raw| flat(family, raw));
        if let Some(window) = flat.filter(|w| !found.iter().any(|f| f.id == w.id)) {
            found.push(window);
        }
    }
    found.truncate(MAX_EXTRA_WINDOWS);
    Ok(found)
}

/// A `weekly_scoped` entry of `limits[]`.
fn scoped(entry: &Value) -> Option<UsageWindow> {
    let entry = entry.as_object()?;
    if entry.get("kind")?.as_str()? != "weekly_scoped" {
        return None;
    }
    let name = entry.get("scope")?.get("model")?.get("display_name")?.as_str()?;
    let label = clean_text(name, MAX_LABEL_CHARS)?;
    Some(weekly(&label, percent(entry.get("percent"), 100.0), resets(entry.get("resets_at"))))
}

/// A flat `seven_day_<family>` key.
fn flat(family: &str, raw: &Value) -> Option<UsageWindow> {
    let object = raw.as_object()?;
    Some(weekly(family, percent(object.get("utilization"), 100.0), resets(object.get("resets_at"))))
}

pub(crate) fn weekly(name: &str, used_pct: Option<f64>, resets_at: Option<i64>) -> UsageWindow {
    let lower = name.to_lowercase();
    let family = if lower.contains("opus") {
        "opus".to_owned()
    } else if lower.contains("sonnet") {
        "sonnet".to_owned()
    } else {
        slug(name)
    };
    let mut window = UsageWindow::new(format!("{family}:weekly"), WindowKind::Weekly);
    // The UI names Opus and Sonnet itself; any other model keeps the provider's display name.
    window.label = (family != "opus" && family != "sonnet").then(|| name.to_owned());
    window.group = Some(family);
    window.window_minutes = Some(WEEKLY_MINUTES);
    window.used_pct = used_pct;
    window.resets_at = resets_at;
    window
}

/// Two entries for one family: the more consumed one is the cap that binds.
fn keep_the_higher(found: &mut Vec<UsageWindow>, window: UsageWindow) {
    match found.iter_mut().find(|f| f.id == window.id) {
        Some(existing) if window.used_pct.unwrap_or(-1.0) > existing.used_pct.unwrap_or(-1.0) => {
            *existing = window;
        }
        Some(_) => {}
        None => found.push(window),
    }
}

fn resets(value: Option<&Value>) -> Option<i64> {
    parse_rfc3339_secs(value?.as_str()?).and_then(plausible)
}

/// Bounds a window list that did not come straight from `parse` (the throttle cache is a file
/// anybody with access to the data directory could edit).
pub(crate) fn bounded(windows: Vec<UsageWindow>) -> Vec<UsageWindow> {
    windows
        .into_iter()
        .filter_map(|w| {
            let id = clean_text(&w.id, 48)?;
            let mut safe = UsageWindow::new(id, w.kind);
            safe.group = w.group.as_deref().and_then(|g| clean_text(g, 48));
            safe.label = w.label.as_deref().and_then(|l| clean_text(l, MAX_LABEL_CHARS));
            safe.window_minutes = w.window_minutes.filter(|m| (1..=WEEKLY_MINUTES).contains(m));
            safe.used_pct = percent(w.used_pct.map(Value::from).as_ref(), 100.0);
            safe.resets_at = w.resets_at.and_then(plausible);
            Some(safe)
        })
        .take(MAX_EXTRA_WINDOWS)
        .collect()
}

#[cfg(test)]
mod tests;
