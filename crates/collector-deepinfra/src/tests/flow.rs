//! Whole polls: the gates that run before any I/O, the views a good answer becomes, and the key
//! staying out of every view.

use usage_model::settings::DeepinfraSettings;
use usage_model::{AmountUnit, ErrorCode, HarnessId, HarnessView, ViewState, WindowKind};
use usage_support::Secret;

use super::{KEY, NOW, enabled, poll, routed, secret};
use crate::{Plan, plan};

const BLANK_KEYS: [&str; 5] = ["", " ", "\t\r\n", "\u{a0}\u{2003}", "   \n   "];

fn facts_of(view: &HarnessView) -> Vec<(&str, f64)> {
    view.facts.iter().map(|f| (f.id.as_str(), f.number.unwrap())).collect()
}

#[test]
fn a_disabled_harness_is_never_read_whatever_else_is_set() {
    let key = secret();
    for key in [None, Some(&key)] {
        match plan(&DeepinfraSettings { enabled: false }, key) {
            Plan::Done(view) => assert_eq!(*view, HarnessView::disabled(HarnessId::Deepinfra)),
            Plan::Read(_) => panic!("a disabled harness must not read"),
        }
    }
}

#[test]
fn no_key_or_a_blank_one_needs_setup() {
    let expected = HarnessView::needs_setup(HarnessId::Deepinfra);
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
    let server = routed((200, "checklist_limited.json"), (200, "usage_ok.json"));
    let off = DeepinfraSettings { enabled: false };
    assert_eq!(poll(&off, Some(&secret()), &server).await.state, ViewState::Disabled);
    assert_eq!(poll(&enabled(), None, &server).await.state, ViewState::NeedsSetup);
    assert_eq!(
        poll(&enabled(), Some(&Secret::new("  ")), &server).await.state,
        ViewState::NeedsSetup
    );
    server.assert_untouched();
}

#[tokio::test]
async fn a_limited_account_is_a_percent_window_in_dollars_plus_facts() {
    let server = routed((200, "checklist_limited.json"), (200, "usage_ok.json"));
    let view = poll(&enabled(), Some(&secret()), &server).await;
    assert_eq!(
        (view.state, view.id, view.read_at),
        (ViewState::Ready, HarnessId::Deepinfra, Some(NOW))
    );
    let source = view.source.as_ref().unwrap();
    assert_eq!((source.id.as_str(), source.official), ("payment_checklist", true));
    let window = &view.windows[0];
    assert_eq!((window.id.as_str(), window.kind), ("spend_limit", WindowKind::Budget));
    assert_eq!((window.used_pct, window.resets_at), (Some(25.0), None));
    let amount = window.amount.as_ref().unwrap();
    assert_eq!((amount.used, amount.limit, amount.unit), (2.5, Some(10.0), AmountUnit::Usd));
    assert_eq!(facts_of(&view), [("balance", 12.5), ("usage_recent", 2.5), ("usage_month", 12.34)]);
    assert!(view.facts.iter().all(|f| f.unit.as_deref() == Some("usd")));
}

#[tokio::test]
async fn without_a_limit_there_is_no_window_and_a_failed_usage_call_only_drops_the_month() {
    let server = routed((200, "checklist_unlimited.json"), (500, "error_auth.json"));
    let view = poll(&enabled(), Some(&secret()), &server).await;
    assert_eq!(view.state, ViewState::Ready);
    assert!(view.windows.is_empty());
    assert_eq!(facts_of(&view), [("balance", 3.25), ("usage_recent", 0.5)]);
}

#[tokio::test]
async fn a_positive_stripe_balance_is_money_owed_not_a_negative_balance() {
    let server = routed((200, "checklist_owing.json"), (404, "error_auth.json"));
    let view = poll(&enabled(), Some(&secret()), &server).await;
    assert_eq!(facts_of(&view), [("owed", 4.0), ("usage_recent", 1.0)]);
}

#[tokio::test]
async fn failures_become_error_views_without_the_key_or_the_response_text() {
    for (status, code) in [
        (401, ErrorCode::Unauthorized),
        (403, ErrorCode::Unauthorized),
        (404, ErrorCode::EndpointChanged),
        (429, ErrorCode::RateLimited),
        (500, ErrorCode::ServerError),
    ] {
        let server = routed((status, "error_auth.json"), (200, "usage_ok.json"));
        let view = poll(&enabled(), Some(&secret()), &server).await;
        assert_eq!(view.state, ViewState::Error, "{status}");
        assert_eq!(view.error.as_ref().map(|e| e.code), Some(code), "{status}");
        assert!(view.facts.is_empty() && view.windows.is_empty());
        assert!(!serde_json::to_string(&view).unwrap().contains("fabricated"));
    }
}

#[tokio::test]
async fn the_key_reaches_only_the_authorization_header_of_the_two_requests() {
    let server = routed((200, "checklist_limited.json"), (200, "usage_ok.json"));
    poll(&enabled(), Some(&secret()), &server).await;
    let requests = server.requests();
    let targets: Vec<&str> = requests.iter().map(|r| r.target.as_str()).collect();
    assert_eq!(targets, ["/payment/checklist", "/payment/usage?from=current"]);
    for request in &requests {
        assert_eq!(request.method, "GET");
        assert_eq!(request.header("authorization"), Some(format!("Bearer {KEY}").as_str()));
        for (name, value) in &request.headers {
            assert_eq!(value.contains(KEY), name.eq_ignore_ascii_case("authorization"), "{name}");
        }
    }
}
