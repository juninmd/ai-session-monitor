//! Turns what was read into the `HarnessView` the UI renders.

use std::collections::BTreeMap;

use usage_model::status::is_expired;
use usage_model::{
    Ctx, ErrorCode, Fact, HarnessId, HarnessView, Notice, Severity, Source, ViewState,
};

use crate::app_server::Reading;
use crate::limits::Limits;
use crate::rollout::Found;

pub(crate) const ROLLOUT_FALLBACK: &str = "codex_rollout_fallback";
pub(crate) const NO_LIMITS: &str = "codex_no_limits";

fn ready(source: &str, official: bool, read_at: Option<i64>) -> HarnessView {
    let mut view = HarnessView::with_state(HarnessId::Codex, ViewState::Ready);
    view.source = Some(Source { id: source.to_owned(), official });
    view.read_at = read_at;
    view
}

fn fill(view: &mut HarnessView, limits: Limits, now_secs: i64) {
    // Every window already past its reset: the numbers stay, dimmed, until fresh ones arrive.
    view.stale = !limits.windows.is_empty()
        && limits.windows.iter().all(|window| is_expired(window, now_secs));
    view.plan = limits.plan;
    view.windows = limits.windows;
    if let Some(balance) = limits.credits {
        view.facts.push(Fact {
            id: "credits_balance".to_owned(),
            number: Some(balance),
            text: None,
            unit: Some("credits".to_owned()),
        });
    }
}

fn notice(code: &str, severity: Severity, params: BTreeMap<String, String>) -> Notice {
    Notice { code: code.to_owned(), severity, params }
}

/// Data from the app-server was produced by this very call, so it is as old as the poll.
pub(crate) fn from_app_server(reading: Reading, ctx: &Ctx) -> HarnessView {
    let mut view = ready("app_server", true, Some(ctx.now_secs));
    match reading {
        Reading::Limits(limits) => fill(&mut view, limits, ctx.now_secs),
        Reading::NoLimits => {
            view.notices.push(notice(NO_LIMITS, Severity::Warn, BTreeMap::new()));
        }
    }
    view
}

/// "Last seen" data from the newest rollout, dated by the event itself. `reason` says why the
/// app-server was not used.
pub(crate) fn from_rollout(found: Found, reason: ErrorCode, ctx: &Ctx) -> HarnessView {
    let mut view = ready("rollout", false, found.read_at);
    fill(&mut view, found.limits, ctx.now_secs);
    let name = serde_json::to_value(reason)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default();
    let params = BTreeMap::from([("reason".to_owned(), name)]);
    view.notices.push(notice(ROLLOUT_FALLBACK, Severity::Info, params));
    view
}

#[cfg(test)]
mod tests {
    use usage_model::{UsageWindow, WindowKind};

    use super::*;

    const NOW: i64 = 1_791_662_400;

    fn ctx() -> Ctx {
        Ctx::new(NOW, "data", "home")
    }

    fn window(id: &str, resets_at: Option<i64>) -> UsageWindow {
        let mut window = UsageWindow::new(id, WindowKind::Other);
        window.used_pct = Some(10.0);
        window.resets_at = resets_at;
        window
    }

    fn limits(windows: Vec<UsageWindow>) -> Limits {
        Limits { windows, plan: Some("Plus".into()), credits: Some(3.5) }
    }

    #[test]
    fn app_server_data_is_official_fresh_and_dated_by_the_poll() {
        let view =
            from_app_server(Reading::Limits(limits(vec![window("a", Some(NOW + 60))])), &ctx());
        assert_eq!(view.id, HarnessId::Codex);
        assert_eq!(view.state, ViewState::Ready);
        assert_eq!(view.source, Some(Source { id: "app_server".into(), official: true }));
        assert_eq!((view.read_at, view.stale), (Some(NOW), false));
        assert_eq!(view.plan.as_deref(), Some("Plus"));
        assert_eq!(view.facts[0].number, Some(3.5));
        assert!(view.notices.is_empty() && view.error.is_none() && view.meta.is_empty());
    }

    #[test]
    fn stale_means_every_window_has_reset_and_the_windows_are_kept() {
        let at = |a: i64, b: i64| {
            from_app_server(
                Reading::Limits(limits(vec![window("a", Some(a)), window("b", Some(b))])),
                &ctx(),
            )
        };
        assert!(!at(NOW + 1, NOW + 2).stale);
        assert!(!at(NOW, NOW + 2).stale, "one window still running");
        let all_reset = at(NOW, NOW - 5);
        assert!(all_reset.stale, "the reset instant itself counts as reset");
        assert_eq!(all_reset.windows.len(), 2);
    }

    #[test]
    fn a_window_with_no_reset_time_is_never_called_reset() {
        let view = from_app_server(
            Reading::Limits(limits(vec![window("a", Some(NOW - 1)), window("b", None)])),
            &ctx(),
        );
        assert!(!view.stale);
        let none = from_app_server(Reading::Limits(limits(vec![])), &ctx());
        assert!(!none.stale && none.windows.is_empty(), "nothing to be stale about");
    }

    #[test]
    fn api_key_mode_is_ready_with_no_windows_and_a_warning() {
        let view = from_app_server(Reading::NoLimits, &ctx());
        assert_eq!(view.state, ViewState::Ready);
        assert!(view.windows.is_empty() && view.plan.is_none() && view.facts.is_empty());
        assert_eq!(view.read_at, Some(NOW));
        let [only] = view.notices.as_slice() else { panic!("{:?}", view.notices) };
        assert_eq!((only.code.as_str(), only.severity), (NO_LIMITS, Severity::Warn));
    }

    #[test]
    fn rollout_data_is_unofficial_dated_by_its_event_and_explains_itself() {
        let found =
            Found { limits: limits(vec![window("a", Some(NOW + 60))]), read_at: Some(NOW - 90) };
        let view = from_rollout(found, ErrorCode::BinaryNotFound, &ctx());
        assert_eq!(view.source, Some(Source { id: "rollout".into(), official: false }));
        assert_eq!(view.read_at, Some(NOW - 90));
        let [only] = view.notices.as_slice() else { panic!("{:?}", view.notices) };
        assert_eq!((only.code.as_str(), only.severity), (ROLLOUT_FALLBACK, Severity::Info));
        assert_eq!(only.params.get("reason").map(String::as_str), Some("binary_not_found"));
    }

    #[test]
    fn a_rollout_whose_windows_all_reset_is_stale() {
        let found = Found { limits: limits(vec![window("a", Some(NOW - 10))]), read_at: None };
        let view = from_rollout(found, ErrorCode::Timeout, &ctx());
        assert!(view.stale);
        assert_eq!(view.read_at, None);
    }

    #[test]
    fn notice_codes_are_all_declared() {
        assert_eq!(crate::NOTICE_CODES, [ROLLOUT_FALLBACK, NO_LIMITS]);
    }
}
