//! Whole polls: the gates that run before any I/O, the views a good answer becomes, and the key
//! staying out of every view.

use usage_model::settings::OpenrouterSettings;
use usage_model::{AmountUnit, ErrorCode, HarnessId, HarnessView, ViewState, WindowKind};
use usage_support::Secret;

use super::fake_server::{FakeServer, Reply};
use super::{NOW, enabled, fixture, poll, routed, secret};
use crate::{Plan, plan};

const BLANK_KEYS: [&str; 5] = ["", " ", "\t\r\n", "\u{a0}\u{2003}", "   \n   "];

fn facts_of(view: &HarnessView) -> Vec<(&str, f64)> {
    view.facts.iter().map(|f| (f.id.as_str(), f.number.unwrap())).collect()
}

#[test]
fn a_disabled_harness_is_never_read_whatever_else_is_set() {
    let key = secret();
    for key in [None, Some(&key)] {
        match plan(&OpenrouterSettings { enabled: false }, key) {
            Plan::Done(view) => assert_eq!(*view, HarnessView::disabled(HarnessId::Openrouter)),
            Plan::Read(_) => panic!("a disabled harness must not read"),
        }
    }
}

#[test]
fn no_key_or_a_blank_one_needs_setup() {
    let expected = HarnessView::needs_setup(HarnessId::Openrouter);
    let blanks: Vec<Secret> = BLANK_KEYS.iter().map(|blank| Secret::new(*blank)).collect();
    for key in std::iter::once(None).chain(blanks.iter().map(Some)) {
        match plan(&enabled(), key) {
            Plan::Done(view) => assert_eq!(*view, expected, "{key:?}"),
            Plan::Read(_) => panic!("{key:?} must not read"),
        }
    }
}

#[tokio::test]
async fn gated_polls_open_no_socket() {
    let server = routed((200, "key_limited.json"), (200, "credits_ok.json"));
    let off = OpenrouterSettings { enabled: false };
    assert_eq!(poll(&off, Some(&secret()), &server).await.state, ViewState::Disabled);
    assert_eq!(poll(&enabled(), None, &server).await.state, ViewState::NeedsSetup);
    assert_eq!(
        poll(&enabled(), Some(&Secret::new("  ")), &server).await.state,
        ViewState::NeedsSetup
    );
    server.assert_untouched();
}

#[tokio::test]
async fn a_limited_key_is_an_official_percent_window_in_dollars_plus_facts() {
    let server = routed((200, "key_limited.json"), (200, "credits_ok.json"));
    let view = poll(&enabled(), Some(&secret()), &server).await;
    assert_eq!(
        (view.state, view.id, view.read_at),
        (ViewState::Ready, HarnessId::Openrouter, Some(NOW))
    );
    let source = view.source.as_ref().unwrap();
    assert_eq!((source.id.as_str(), source.official), ("key_credits", true));
    let window = &view.windows[0];
    assert_eq!((window.id.as_str(), window.kind), ("key_limit", WindowKind::Budget));
    assert_eq!(window.used_pct, Some(25.0));
    assert_eq!(window.resets_at, None, "the reset moment is not reported, so it stays unknown");
    let amount = window.amount.as_ref().unwrap();
    assert_eq!((amount.used, amount.limit, amount.unit), (2.5, Some(10.0), AmountUnit::Usd));
    assert_eq!(
        facts_of(&view),
        [
            ("balance", 74.75),
            ("usage_daily", 0.25),
            ("usage_weekly", 1.0),
            ("usage_monthly", 2.5),
            ("usage_total", 2.5)
        ]
    );
    assert!(view.facts.iter().all(|f| f.unit.as_deref() == Some("usd")));
}

#[tokio::test]
async fn an_unlimited_key_has_facts_and_no_made_up_percent() {
    let server = routed((200, "key_unlimited.json"), (403, "credits_forbidden.json"));
    let view = poll(&enabled(), Some(&secret()), &server).await;
    assert_eq!(view.state, ViewState::Ready);
    assert!(view.windows.is_empty());
    let ids: Vec<&str> = view.facts.iter().map(|f| f.id.as_str()).collect();
    assert_eq!(
        ids,
        ["usage_daily", "usage_weekly", "usage_monthly", "usage_total"],
        "no balance without credits"
    );
}

#[tokio::test]
async fn an_overspent_key_shows_the_overage_unclamped() {
    let server = routed((200, "key_overspent.json"), (403, "credits_forbidden.json"));
    let window = poll(&enabled(), Some(&secret()), &server).await.windows.remove(0);
    assert_eq!(window.used_pct, Some(130.0));
    assert_eq!(window.amount.unwrap().used, 6.5);
}

#[tokio::test]
async fn failures_become_error_views_without_the_key_or_the_response_text() {
    for (status, code) in [
        (401, ErrorCode::Unauthorized),
        (402, ErrorCode::BadResponse),
        (403, ErrorCode::Unauthorized),
        (404, ErrorCode::EndpointChanged),
        (429, ErrorCode::RateLimited),
        (500, ErrorCode::ServerError),
    ] {
        let server = FakeServer::always(move || Reply::json(status, &fixture("error_auth.json")));
        let view = poll(&enabled(), Some(&secret()), &server).await;
        assert_eq!(view.state, ViewState::Error, "{status}");
        assert_eq!(view.error.as_ref().map(|e| e.code), Some(code), "{status}");
        assert!(view.windows.is_empty() && view.facts.is_empty());
        assert!(!serde_json::to_string(&view).unwrap().contains("fabricated"));
    }
}
