//! A good answer: what is added to the statusLine view, and exactly what was asked of the server.

use usage_model::{ViewState, WindowKind};

use super::*;
use crate::testenv::{credentials_json, usage_flat, usage_limits, usage_no_model_caps};

fn ids(view: &HarnessView) -> Vec<&str> {
    view.windows.iter().map(|w| w.id.as_str()).collect()
}

#[tokio::test]
async fn per_model_windows_and_the_plan_join_the_statusline_windows() {
    let env = ready_env();
    let server = FakeServer::json(&usage_limits());
    let view = poll(&env, &server, NOW).await;

    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(ids(&view), ["five_hour", "weekly", "opus:weekly", "sonnet:weekly"]);
    assert_eq!(view.plan.as_deref(), Some("Max 5x"));
    assert_eq!(view.meta["extra_source"], "oauth_usage");
    assert!(view.notices.is_empty() && view.error.is_none() && !view.stale);

    let opus = &view.windows[2];
    assert_eq!(
        (opus.kind, opus.group.as_deref(), opus.label.as_deref()),
        (WindowKind::Weekly, Some("opus"), None)
    );
    assert_eq!(
        (opus.used_pct, opus.resets_at, opus.window_minutes),
        (Some(61.0), Some(1_800_259_200), Some(10_080))
    );
    let sonnet = &view.windows[3];
    assert_eq!((sonnet.group.as_deref(), sonnet.used_pct), (Some("sonnet"), Some(18.5)));
}

#[tokio::test]
async fn the_statusline_part_is_not_touched_by_the_extras() {
    let env = ready_env();
    let server = FakeServer::json(&usage_limits());
    let without = crate::collect_with(&env.settings(), &env.ctx(), &endpoint(&server)).await;
    server.assert_untouched();
    let with = poll(&env, &server, NOW).await;

    assert_eq!(&with.windows[..2], &without.windows[..]);
    assert_eq!((with.read_at, with.source.clone()), (without.read_at, without.source.clone()));
    assert_eq!(server.connections(), 1);
}

#[tokio::test]
async fn the_older_flat_keys_are_understood_too() {
    let env = ready_env();
    let server = FakeServer::json(&usage_flat());
    let view = poll(&env, &server, NOW).await;
    assert_eq!(
        ids(&view),
        ["five_hour", "weekly", "opus:weekly"],
        "a null seven_day_sonnet is no cap"
    );
    assert_eq!(view.windows[2].used_pct, Some(61.0));
    assert_eq!(view.meta["extra_source"], "oauth_usage");
}

#[tokio::test]
async fn an_account_without_model_caps_gets_only_its_plan() {
    let env = ready_env();
    let server = FakeServer::json(&usage_no_model_caps());
    let view = poll(&env, &server, NOW).await;
    assert_eq!(ids(&view), ["five_hour", "weekly"]);
    assert_eq!(view.plan.as_deref(), Some("Max 5x"));
    assert!(!view.meta.contains_key("extra_source"), "nothing came from the unofficial source");
    assert!(view.notices.is_empty());
}

#[tokio::test]
async fn an_unknown_subscription_is_not_shown_as_a_plan() {
    let env = ready_env();
    let odd = credentials_json((NOW + 3600) * 1000).replace("\"max\"", "\"<b>vip</b>\"");
    env.write(&env.credentials(), odd);
    let server = FakeServer::json(&usage_limits());
    let view = poll(&env, &server, NOW).await;
    assert_eq!(view.plan, None);
    assert_eq!(view.windows.len(), 4, "the windows do not depend on the plan");
}

#[tokio::test]
async fn the_request_asks_for_one_document_with_the_login_only() {
    let env = ready_env();
    let server = FakeServer::json(&usage_no_model_caps());
    poll(&env, &server, NOW).await;

    let requests = server.requests();
    let [request] = requests.as_slice() else { panic!("expected one request, got {requests:?}") };
    assert_eq!((request.method.as_str(), request.target.as_str()), ("GET", "/api/oauth/usage"));
    assert_eq!(request.header("authorization"), Some(format!("Bearer {FAKE_TOKEN}").as_str()));
    assert_eq!(request.header("anthropic-beta"), Some("oauth-2025-04-20"));
    assert_eq!(request.header("accept"), Some("application/json"));
    assert!(
        request.header("user-agent").is_some_and(|agent| agent.starts_with("ai-session-monitor/"))
    );
    assert!(request.header("cookie").is_none() && request.header("proxy-authorization").is_none());
}

#[tokio::test]
async fn extras_do_not_replace_a_plan_that_is_already_known() {
    let env = ready_env();
    let server = FakeServer::json(&usage_limits());
    let mut view = HarnessView::with_state(usage_model::HarnessId::Claude, ViewState::Ready);
    view.plan = Some("Pro".to_owned());
    super::super::add_extras(&mut view, &oauth_on(&env), &env.ctx(), &endpoint(&server)).await;
    assert_eq!(view.plan.as_deref(), Some("Pro"));
    assert_eq!(view.windows.len(), 2, "the model windows are still added");
}

#[tokio::test]
async fn an_unusable_config_dir_adds_nothing_and_asks_nothing() {
    let env = ready_env();
    let server = FakeServer::json(&usage_limits());
    let relative = ClaudeSettings { config_dir: Some("relative".into()), ..oauth_on(&env) };
    let mut view = HarnessView::with_state(usage_model::HarnessId::Claude, ViewState::Ready);
    super::super::add_extras(&mut view, &relative, &env.ctx(), &endpoint(&server)).await;
    assert!(view.windows.is_empty() && view.notices.is_empty());
    server.assert_untouched();
}
