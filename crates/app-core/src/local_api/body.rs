//! The JSON of `GET /v1/limits`. Built from the same views the webview gets, narrowed to what is
//! worth showing in a status bar: ids, states, percentages, reset moments and numeric facts. Errors,
//! notices, `meta`, window labels and groups are not here, so nothing a collector put there (a
//! host, a path, a provider's label) can leave the app through this port.

use serde::Serialize;
use usage_model::status::{Status, available_pct, is_expired, status_of};
use usage_model::{HarnessId, HarnessView, Snapshot, UsageWindow, ViewState, WindowKind};

use crate::i18n::harness_name;

pub(super) const VERSION: u8 = 1;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Limits {
    version: u8,
    generated_at: String,
    harnesses: Vec<HarnessOut>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HarnessOut {
    id: HarnessId,
    name: &'static str,
    state: ViewState,
    /// `null` while no source has answered yet.
    official: Option<bool>,
    plan: Option<String>,
    stale: bool,
    windows: Vec<WindowOut>,
    facts: Vec<FactOut>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WindowOut {
    id: String,
    kind: WindowKind,
    /// `null` when unknown, and for a window that already reset.
    used_pct: Option<f64>,
    available_pct: Option<f64>,
    /// RFC 3339 (UTC), `null` when the source gave none.
    resets_at: Option<String>,
    /// `ok`, `warn`, `crit` or `out`; `null` for an unknown or expired window.
    status: Option<&'static str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FactOut {
    id: String,
    number: Option<f64>,
    text: Option<String>,
    unit: Option<String>,
}

/// The response body. Disabled harnesses are left out; every other one is listed, in tab order,
/// with its state so a consumer can tell "needs setup" from "no data".
pub(super) fn limits(snapshot: &Snapshot, warn_at_pct: u8) -> Vec<u8> {
    let now = snapshot.generated_at;
    let body = Limits {
        version: VERSION,
        generated_at: rfc3339(now),
        harnesses: snapshot
            .views
            .iter()
            .filter(|view| view.state != ViewState::Disabled)
            .map(|view| harness(view, now, f64::from(warn_at_pct)))
            .collect(),
    };
    // Plain data with string keys cannot fail to serialize; an empty object is never a valid
    // answer, so a failure surfaces as a body the consumer rejects.
    serde_json::to_vec(&body).unwrap_or_default()
}

fn harness(view: &HarnessView, now: i64, warn_at_pct: f64) -> HarnessOut {
    HarnessOut {
        id: view.id,
        name: harness_name(view.id),
        state: view.state,
        official: view.source.as_ref().map(|source| source.official),
        plan: view.plan.clone(),
        stale: view.stale,
        windows: view.windows.iter().map(|w| window(w, now, warn_at_pct)).collect(),
        facts: view
            .facts
            .iter()
            .map(|fact| FactOut {
                id: fact.id.clone(),
                number: fact.number.filter(|n| n.is_finite()),
                text: fact.text.clone(),
                unit: fact.unit.clone(),
            })
            .collect(),
    }
}

fn window(window: &UsageWindow, now: i64, warn_at_pct: f64) -> WindowOut {
    // A window that reset no longer describes reality: no numbers, no status.
    let expired = is_expired(window, now);
    let used = window.used_pct.filter(|pct| pct.is_finite() && !expired);
    let status = status_of(window, now, warn_at_pct).map(|status| match status {
        Status::Ok => "ok",
        Status::Warn => "warn",
        Status::Crit => "crit",
        Status::Out => "out",
    });
    WindowOut {
        id: window.id.clone(),
        kind: window.kind,
        used_pct: used,
        available_pct: used.and(available_pct(window)),
        resets_at: window.resets_at.map(rfc3339),
        status,
    }
}

/// `2026-10-10T12:00:00Z`. Days-to-civil conversion after Howard Hinnant's date algorithms; the
/// range of `i64` seconds that matter here (a few thousand years either side of 1970) is exact.
pub(super) fn rfc3339(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rest = secs.rem_euclid(86_400);
    let z = days.saturating_add(719_468);
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    let (hour, minute, second) = (rest / 3600, rest % 3600 / 60, rest % 60);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}
