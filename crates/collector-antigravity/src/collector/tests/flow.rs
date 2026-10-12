//! A poll that works: what is asked, and what the view says about the answers.

use serde_json::{Value, json};
use usage_model::{HarnessId, ViewState, WindowKind};

use super::*;
use crate::discovery::Scheme;
use crate::fake::Request;

const BASE: &str = "/exa.language_server_pb.LanguageServerService";

fn body_of(request: &Request) -> Value {
    serde_json::from_str(&request.body).expect("request bodies are JSON")
}

#[tokio::test]
async fn a_full_poll_asks_status_then_summary_the_way_the_app_does() {
    let server = serve("status_ide", Some("summary_full")).await;
    let view = poll(vec![server.target(TOKEN)]).await;

    assert_eq!((view.id, view.state), (HarnessId::Antigravity, ViewState::Ready));
    assert_eq!(
        rows(&view),
        [
            ("gemini:five_hour", Some(54.0), Some(1_791_641_520)),
            ("gemini:weekly", Some(29.0), Some(1_792_053_939)),
            ("claude_gpt:five_hour", Some(39.0), Some(1_791_649_800)),
            ("claude_gpt:weekly", Some(12.0), Some(1_792_111_194)),
        ]
    );
    assert_eq!(view.plan.as_deref(), Some("Google AI Pro"));
    assert_eq!(view.read_at, Some(NOW));
    let source = view.source.as_ref().expect("source");
    assert_eq!((source.id.as_str(), source.official), ("language_server", false));
    assert!(!view.stale && view.error.is_none() && view.notices.is_empty());

    let requests = server.requests();
    let methods: Vec<&str> = requests.iter().map(Request::method).collect();
    assert_eq!(methods, ["GetUserStatus", "RetrieveUserQuotaSummary"]);
    for request in &requests {
        assert_eq!(request.path, format!("{BASE}/{}", request.method()));
        assert_eq!(request.header("x-codeium-csrf-token"), Some(TOKEN));
        assert_eq!(request.header("connect-protocol-version"), Some("1"));
        assert!(
            request.header("content-type").is_some_and(|kind| kind.starts_with("application/json"))
        );
        assert!(request.header("host").is_some_and(|host| host.starts_with("127.0.0.1:")));
    }
    let metadata = &body_of(&requests[0])["metadata"];
    assert_eq!((&metadata["ideName"], &metadata["locale"]), (&json!("antigravity"), &json!("en")));
    assert_eq!(body_of(&requests[1]), json!({ "forceRefresh": true }));
}

#[tokio::test]
async fn summary_windows_carry_the_lengths_the_plans_fix() {
    let server = serve("status_hub", Some("summary_full")).await;
    let view = poll(vec![server.target(TOKEN)]).await;
    let shape: Vec<_> =
        view.windows.iter().map(|w| (w.kind, w.group.as_deref(), w.window_minutes)).collect();
    assert_eq!(
        shape,
        [
            (WindowKind::FiveHour, Some("gemini"), Some(300)),
            (WindowKind::Weekly, Some("gemini"), Some(10_080)),
            (WindowKind::FiveHour, Some("claude_gpt"), Some(300)),
            (WindowKind::Weekly, Some("claude_gpt"), Some(10_080)),
        ]
    );
}

#[tokio::test]
async fn an_older_ide_without_the_summary_shows_the_session_windows_of_its_status() {
    let server = serve("status_ide", None).await;
    let view = poll(vec![server.target(TOKEN)]).await;
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(
        rows(&view),
        [
            ("gemini:five_hour", Some(30.0), Some(1_791_641_520)),
            ("claude_gpt:five_hour", Some(50.0), Some(1_791_649_800)),
        ]
    );
    // The length is a guess here, so it stays unsaid and cannot steer the pace estimate.
    assert!(view.windows.iter().all(|w| w.kind == WindowKind::FiveHour));
    assert!(view.windows.iter().all(|w| w.window_minutes.is_none()));
    assert_eq!(view.plan.as_deref(), Some("Google AI Pro"));
    assert!(view.error.is_none());
    assert_eq!(server.requests().len(), 2, "the 404 is expected and not retried");
}

#[tokio::test]
async fn a_weekly_only_plan_lists_weekly_windows_and_the_summary_outranks_the_status() {
    let server = serve("status_free", Some("summary_free")).await;
    let view = poll(vec![server.target(TOKEN)]).await;
    // The status alone would say 20 % for Gemini; the summary states 65 % and wins.
    assert_eq!(
        rows(&view),
        [
            ("gemini:weekly", Some(65.0), Some(1_792_195_200)),
            ("claude_gpt:weekly", Some(0.0), Some(1_792_195_200)),
        ]
    );
    assert_eq!(view.plan.as_deref(), Some("Free"));
}

#[tokio::test]
async fn a_newer_hub_gets_its_windows_from_the_summary_and_its_plan_from_the_status() {
    let server = serve("status_hub", Some("summary_hub")).await;
    let view = poll(vec![server.target(TOKEN)]).await;
    let used: Vec<Option<f64>> = view.windows.iter().map(|w| w.used_pct).collect();
    assert_eq!(used, [Some(51.0), Some(10.0), Some(75.0), Some(40.0)]);
    assert_eq!(view.plan.as_deref(), Some("Pro"));
}

#[tokio::test]
async fn a_plan_with_no_quota_to_show_is_no_data_not_an_error() {
    let server = serve("status_hub", None).await;
    let view = poll(vec![server.target(TOKEN)]).await;
    assert_eq!(view.state, ViewState::NoData);
    assert_eq!(view.plan.as_deref(), Some("Pro"));
    assert!(view.windows.is_empty() && view.error.is_none() && !view.stale);
    assert_eq!(view.read_at, Some(NOW));
    assert!(view.source.is_some());
}

#[tokio::test]
async fn unknown_shares_stay_unknown_and_are_flagged_once_per_pool() {
    let server = serve("status_hub", Some("summary_prose")).await;
    let view = poll(vec![server.target(TOKEN)]).await;
    // Reset times come from the prose ("in 3 hours, 30 minutes"); the disabled bucket is left out.
    assert_eq!(
        rows(&view),
        [
            ("gemini:five_hour", Some(50.0), Some(NOW + 3 * 3_600 + 1_800)),
            ("gemini:weekly", None, Some(NOW + 5 * 86_400 + 11 * 3_600)),
            ("claude_gpt:weekly", Some(60.0), Some(1_792_111_194)),
        ]
    );
    assert_eq!(view.notices.len(), 1);
    let notice = &view.notices[0];
    assert_eq!(notice.code, "antigravity_pool_unknown");
    assert_eq!(notice.params.get("pool").map(String::as_str), Some("gemini"));
    assert_eq!(view.state, ViewState::Ready);
}

#[tokio::test]
async fn windows_that_already_reset_are_kept_and_marked_stale() {
    let past = r#"{"response":{"groups":[{"displayName":"Gemini Models","buckets":[
        {"bucketId":"gemini-5h","remainingFraction":0.4,"resetTime":"2026-10-10T10:00:00Z"}]}]}}"#;
    let server = Script::new(TOKEN, text("status_hub"), Some(past)).serve_http().await;
    let view = poll(vec![server.target(TOKEN)]).await;
    assert_eq!(view.state, ViewState::Ready);
    assert!(view.stale);
    assert_eq!(rows(&view), [("gemini:five_hour", Some(60.0), Some(1_791_626_400))]);
}

#[tokio::test]
async fn the_clock_of_the_poll_dates_the_data_and_the_prose() {
    let server = serve("status_hub", Some("summary_prose")).await;
    let later = NOW + 123;
    let discovery = StaticDiscovery::new(Ok(vec![server.target(TOKEN)]));
    let ctx =
        Ctx::new(later, "data-dir-the-collector-never-uses", "home-dir-the-collector-never-uses");
    let view = collect_with(&on(), &ctx, &discovery, &config()).await;
    assert_eq!(view.read_at, Some(later));
    assert_eq!(view.windows[0].resets_at, Some(later + 3 * 3_600 + 1_800));
}

#[tokio::test]
async fn https_with_a_self_signed_certificate_works_and_carries_the_token() {
    let script = Script::new(TOKEN, text("status_ide"), Some(text("summary_full")));
    let server = script.serve_https().await;
    let target = server.target(TOKEN);
    assert_eq!(target.scheme, Scheme::Https);
    let view = poll(vec![target]).await;
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(view.windows.len(), 4);
    assert!(server.requests().iter().all(|r| r.header("x-codeium-csrf-token") == Some(TOKEN)));
}

#[tokio::test]
async fn the_extension_server_fallback_speaks_plain_http_with_its_own_token() {
    let script =
        Script::new("extension-token-0002", text("status_free"), Some(text("summary_free")));
    let server = script.serve_http().await;
    let target = server.target("extension-token-0002");
    assert_eq!(target.scheme, Scheme::Http);
    let view = poll(vec![target]).await;
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(view.windows.len(), 2);
}
