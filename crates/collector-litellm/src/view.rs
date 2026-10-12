//! Assembles the `Ready` view of one poll.

use std::collections::BTreeMap;

use usage_model::{
    ErrorCode, HarnessError, HarnessId, HarnessView, Notice, Severity, Source, UsageWindow,
    ViewState,
};

use crate::budget::{self, Windows};
use crate::plan::Plan;
use crate::record::Kind;
use crate::sanitize::display_text;
use crate::walk::Resolved;

/// Every code this crate can put in `Notice::code`.
pub(crate) const NO_BUDGET: &str = "litellm_no_budget";
pub(crate) const LEVEL_UNREADABLE: &str = "litellm_level_unreadable";
pub(crate) const WINDOWS_TRUNCATED: &str = "litellm_windows_truncated";

pub(crate) fn ready(plan: &Plan<'_>, resolved: &Resolved, now_secs: i64) -> HarnessView {
    let (kind, level) = match &resolved.followed {
        Some((kind, level)) => (*kind, level),
        None => (plan.scope, &resolved.primary),
    };
    let secret = plan.secret.expose();
    let Windows { list, dropped } = budget::windows(level, secret);

    let mut view = HarnessView::with_state(HarnessId::Litellm, ViewState::Ready);
    view.source = Some(Source { id: "key_info".to_owned(), official: true });
    // The response was produced during this poll.
    view.read_at = Some(now_secs);
    view.stale = all_reset(&list, now_secs);
    view.facts = budget::facts(&resolved.primary);
    view.notices = notices(&list, dropped, &resolved.unreadable);
    view.meta = meta(plan, kind, resolved, secret);
    view.windows = list;
    view
}

/// Every window already reset: the proxy sweeps resets about every ten minutes, so the spend
/// shown may still be the old period's until the next poll.
fn all_reset(windows: &[UsageWindow], now_secs: i64) -> bool {
    !windows.is_empty() && windows.iter().all(|w| w.resets_at.is_some_and(|at| at <= now_secs))
}

fn meta(
    plan: &Plan<'_>,
    level: Kind,
    resolved: &Resolved,
    secret: &str,
) -> BTreeMap<String, String> {
    let mut meta = BTreeMap::from([
        ("host".to_owned(), plan.host.clone()),
        ("scope".to_owned(), plan.scope.as_str().to_owned()),
        ("level".to_owned(), level.as_str().to_owned()),
    ]);
    // The alias of the record the user configured, whichever level supplied the windows.
    if let Some(alias) = resolved.primary.alias.as_deref().and_then(|a| display_text(a, secret)) {
        meta.insert("alias".to_owned(), alias);
    }
    meta
}

fn notices(
    windows: &[UsageWindow],
    dropped: usize,
    unreadable: &[(Kind, HarnessError)],
) -> Vec<Notice> {
    let mut out = Vec::new();
    // Null is not unlimited, but "no cap" may only be said when every level was actually read.
    if windows.is_empty() && unreadable.is_empty() {
        out.push(notice(NO_BUDGET, Severity::Info, &[]));
    }
    for (kind, error) in unreadable {
        let reason = code_name(error.code);
        out.push(notice(
            LEVEL_UNREADABLE,
            Severity::Warn,
            &[("level", kind.as_str()), ("reason", &reason)],
        ));
    }
    if dropped > 0 {
        out.push(notice(WINDOWS_TRUNCATED, Severity::Info, &[("dropped", &dropped.to_string())]));
    }
    out
}

fn notice(code: &str, severity: Severity, params: &[(&str, &str)]) -> Notice {
    Notice {
        code: code.to_owned(),
        severity,
        params: params.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect(),
    }
}

/// The snake_case wire name of an error code, as the UI knows it.
fn code_name(code: ErrorCode) -> String {
    serde_json::to_value(code)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unknown".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(reset: Option<i64>) -> UsageWindow {
        UsageWindow { resets_at: reset, ..UsageWindow::new("w", usage_model::WindowKind::Budget) }
    }

    #[test]
    fn stale_only_when_every_window_already_reset() {
        let now = 1_000;
        assert!(!all_reset(&[], now), "nothing to be stale about");
        assert!(!all_reset(&[window(Some(now + 1))], now));
        assert!(all_reset(&[window(Some(now))], now), "a reset at this very second has happened");
        assert!(all_reset(&[window(Some(now - 1))], now));
        assert!(!all_reset(&[window(Some(now - 1)), window(Some(now + 60))], now));
        assert!(!all_reset(&[window(None)], now), "a budget that never resets never goes stale");
        assert!(!all_reset(&[window(Some(now - 1)), window(None)], now));
    }

    #[test]
    fn error_codes_are_reported_by_their_wire_names() {
        assert_eq!(code_name(ErrorCode::Unauthorized), "unauthorized");
        assert_eq!(code_name(ErrorCode::EndpointChanged), "endpoint_changed");
        assert_eq!(code_name(ErrorCode::Timeout), "timeout");
    }

    #[test]
    fn an_unreadable_level_never_comes_with_a_claim_of_no_budget() {
        let err = HarnessError::new(ErrorCode::Unauthorized, false);
        let got = notices(&[], 0, &[(Kind::User, err)]);
        let codes: Vec<_> = got.iter().map(|n| n.code.as_str()).collect();
        assert_eq!(codes, [LEVEL_UNREADABLE]);
        assert_eq!(got[0].severity, Severity::Warn);
        assert_eq!(got[0].params.get("level").map(String::as_str), Some("user"));
        assert_eq!(got[0].params.get("reason").map(String::as_str), Some("unauthorized"));
    }

    #[test]
    fn no_windows_and_nothing_unreadable_says_no_budget() {
        let got = notices(&[], 0, &[]);
        assert_eq!(got.len(), 1);
        assert_eq!((got[0].code.as_str(), got[0].severity), (NO_BUDGET, Severity::Info));
        assert!(got[0].params.is_empty());
    }

    #[test]
    fn a_cut_list_reports_how_many_windows_were_left_out() {
        let got = notices(&[window(None)], 7, &[]);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].code, WINDOWS_TRUNCATED);
        assert_eq!(got[0].params.get("dropped").map(String::as_str), Some("7"));
    }
}
