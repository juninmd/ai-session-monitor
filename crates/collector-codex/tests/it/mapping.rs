//! What the app-server says, as the view shows it.

use std::time::Duration;

use serde_json::json;
use usage_model::{
    Fact, HarnessView, Notice, Severity, Source, UsageWindow, ViewState, WindowKind,
};

use crate::common::{FIVE_HOUR_RESET, NOW, Rig, WEEKLY_RESET};

fn window(
    id: &str,
    kind: WindowKind,
    minutes: u32,
    used: Option<f64>,
    resets: Option<i64>,
) -> UsageWindow {
    let mut window = UsageWindow::new(id, kind);
    window.window_minutes = Some(minutes);
    window.used_pct = used;
    window.resets_at = resets;
    window
}

fn five_hour(used: f64) -> UsageWindow {
    window("five_hour", WindowKind::FiveHour, 300, Some(used), Some(FIVE_HOUR_RESET))
}

fn weekly(used: f64) -> UsageWindow {
    window("weekly", WindowKind::Weekly, 10_080, Some(used), Some(WEEKLY_RESET))
}

fn app_server() -> Option<Source> {
    Some(Source { id: "app_server".into(), official: true })
}

/// What every healthy read has in common, whatever the plan.
fn assert_fresh_official_read(view: &HarnessView) {
    assert_eq!(view.state, ViewState::Ready, "{view:?}");
    assert_eq!(view.source, app_server());
    assert_eq!((view.read_at, view.stale), (Some(NOW), false));
    assert!(view.error.is_none() && view.notices.is_empty() && view.meta.is_empty());
}

#[tokio::test]
async fn a_plus_account_maps_both_windows_the_plan_and_the_credit_balance() {
    let view = Rig::new("ok").collect().await;
    assert_fresh_official_read(&view);
    assert_eq!(view.plan.as_deref(), Some("Plus"));
    assert_eq!(view.windows, [five_hour(42.0), weekly(17.0)]);
    let credits = Fact {
        id: "credits_balance".into(),
        number: Some(12.5),
        text: None,
        unit: Some("credits".into()),
    };
    assert_eq!(view.facts, [credits]);
}

#[tokio::test]
async fn a_plan_with_only_the_five_hour_window_shows_just_that_one() {
    let view = Rig::new("plus_only").collect().await;
    assert_fresh_official_read(&view);
    assert_eq!(view.windows, [five_hour(42.0)]);
    assert!(view.facts.is_empty(), "no credits were reported");
}

#[tokio::test]
async fn a_plan_without_a_five_hour_window_shows_the_weekly_one_alone() {
    let view = Rig::new("pro_no_primary").collect().await;
    assert_fresh_official_read(&view);
    assert_eq!(view.plan.as_deref(), Some("Pro"));
    assert_eq!(view.windows, [weekly(17.0)], "the missing primary must not become a 0 % window");
}

#[tokio::test]
async fn an_api_key_login_has_no_windows_and_says_so() {
    for mode in ["api_key", "api_key_rpc", "no_auth_provider"] {
        let view = Rig::new(mode).collect().await;
        assert_eq!(view.state, ViewState::Ready, "{mode}: {view:?}");
        assert_eq!(view.source, app_server(), "{mode}");
        assert!(view.windows.is_empty() && view.facts.is_empty() && view.plan.is_none(), "{mode}");
        assert!(view.error.is_none() && !view.stale, "{mode}");
        let expected = Notice {
            code: "codex_no_limits".into(),
            severity: Severity::Warn,
            params: Default::default(),
        };
        assert_eq!(view.notices, [expected], "{mode}");
    }
}

#[tokio::test]
async fn broken_numbers_are_unknown_not_clamped_into_plausible_ones() {
    let view = Rig::new("bad_numbers").collect().await;
    assert_eq!(view.state, ViewState::Ready, "{view:?}");
    // -5 %, 100000 % and a millisecond timestamp: the lengths survive, the bad values do not.
    let week = window("weekly", WindowKind::Weekly, 10_080, None, None);
    assert_eq!(
        view.windows,
        [window("five_hour", WindowKind::FiveHour, 300, None, Some(FIVE_HOUR_RESET)), week]
    );
    assert!(view.facts.is_empty(), "a balance of NaN is not a balance");
}

#[tokio::test]
async fn noise_from_the_server_is_ignored_and_its_requests_are_refused() {
    let rig = Rig::new("noisy");
    let view = rig.collect().await;
    // The notification in the noise says 99 %: it must not be taken for the answer.
    assert_fresh_official_read(&view);
    assert_eq!(view.windows, [five_hour(42.0), weekly(17.0)]);

    let refusal = rig.received().into_iter().find(|message| message["id"] == json!("srv-1"));
    let refusal = refusal.expect("the server's own request was never answered");
    assert_eq!(refusal["error"]["code"], json!(-32601));
    assert!(refusal.get("result").is_none());
}

#[tokio::test]
async fn the_conversation_follows_the_documented_handshake() {
    let rig = Rig::new("ok");
    rig.collect().await;
    let sent = rig.received();
    let shape: Vec<_> =
        sent.iter().map(|m| (m["method"].as_str().unwrap_or(""), m["id"].as_i64())).collect();
    let expected = [
        ("initialize", Some(1)),
        ("initialized", None),
        ("account/read", Some(2)),
        ("account/rateLimits/read", Some(3)),
    ];
    assert_eq!(shape, expected);
    assert!(sent.iter().all(|m| m.get("jsonrpc").is_none()), "the wire omits the version member");

    let info = &sent[0]["params"]["clientInfo"];
    assert!(
        ["name", "title", "version"]
            .iter()
            .all(|k| info[k].as_str().is_some_and(|v| !v.is_empty()))
    );
    assert_eq!(
        sent[2]["params"],
        json!({"refreshToken": false}),
        "never ask the server to refresh"
    );
    assert!(sent[3].get("params").is_none_or(|params| params.is_null()));
}

#[tokio::test]
async fn the_public_entry_point_uses_the_documented_budget() {
    assert_eq!(collector_codex::APP_SERVER_BUDGET, Duration::from_secs(15));
    let rig = Rig::new("ok");
    let view = collector_codex::collect(&rig.settings(), &rig.ctx_at(NOW)).await;
    assert_fresh_official_read(&view);
    assert_eq!(view.windows.len(), 2);
}

#[test]
fn the_entry_point_future_can_be_boxed_and_sent_between_threads() {
    // The app wires it as `Pin<Box<dyn Future + Send>>`; building the future runs nothing.
    fn assert_send<T: Send>(_: &T) {}
    let settings = usage_model::settings::CodexSettings::default();
    let ctx = usage_model::Ctx::new(NOW, "data", "home");
    let future = collector_codex::collect(&settings, &ctx);
    assert_send(&future);
}
