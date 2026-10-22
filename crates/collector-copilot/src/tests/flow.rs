//! Whole polls: the gates that run before any I/O, the token order, the view a good answer
//! becomes, and the tokens staying out of every view.

use usage_model::settings::CopilotSettings;
use usage_model::{ErrorCode, HarnessId, HarnessView, ViewState};
use usage_support::Secret;

use super::fake_server::{FakeServer, Reply};
use super::{
    APPS_TOKEN, FIRST_OF_FEBRUARY, HOSTS_TOKEN, NOW, TOKEN, fixture, poll, put, secret, settings,
};
use crate::{Plan, plan};

const BLANK_TOKENS: [&str; 5] = ["", " ", "\t\r\n", "\u{a0}\u{2003}", "   \n   "];

fn serving(name: &'static str) -> FakeServer {
    FakeServer::always(move || Reply::json(200, &fixture(name)))
}

fn home_with_files() -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    put(home.path(), "apps.json", &fixture("apps.json"));
    put(home.path(), "hosts.json", &fixture("hosts.json"));
    home
}

fn authorization(server: &FakeServer) -> String {
    let requests = server.requests();
    assert_eq!(requests.len(), 1, "exactly one request per poll");
    requests[0].header("authorization").unwrap().to_owned()
}

#[test]
fn a_disabled_harness_is_never_read_whatever_else_is_set() {
    let token = secret();
    for token in [None, Some(&token)] {
        for local_consent in [false, true] {
            let off = CopilotSettings { enabled: false, local_consent };
            match plan(&off, token) {
                Plan::Done(view) => assert_eq!(*view, HarnessView::disabled(HarnessId::Copilot)),
                _ => panic!("a disabled harness must not read"),
            }
        }
    }
}

#[test]
fn no_token_and_no_consent_needs_setup_and_a_blank_paste_counts_as_none() {
    let expected = HarnessView::needs_setup(HarnessId::Copilot);
    let blanks: Vec<Secret> = BLANK_TOKENS.iter().map(|blank| Secret::new(*blank)).collect();
    for token in std::iter::once(None).chain(blanks.iter().map(Some)) {
        match plan(&settings(false), token) {
            Plan::Done(view) => assert_eq!(*view, expected, "{token:?}"),
            _ => panic!("{token:?} must not read"),
        }
    }
}

#[test]
fn a_pasted_token_wins_and_is_trimmed_and_consent_without_one_reads_locally() {
    let padded = Secret::new(format!("  {TOKEN}\n"));
    assert!(
        matches!(plan(&settings(false), Some(&padded)), Plan::Pasted(t) if t.expose() == TOKEN)
    );
    assert!(matches!(plan(&settings(true), Some(&padded)), Plan::Pasted(_)));
    let blank = Secret::new("  ");
    assert!(matches!(plan(&settings(true), Some(&blank)), Plan::Local));
    assert!(matches!(plan(&settings(true), None), Plan::Local));
}

#[tokio::test]
async fn gated_polls_open_no_socket_and_read_no_file() {
    let server = serving("individual_pro.json");
    let home = home_with_files();
    let off = CopilotSettings { enabled: false, local_consent: true };
    assert_eq!(poll(&off, Some(&secret()), home.path(), &server).await.state, ViewState::Disabled);
    // Files are present but consent is not: they are not a token source.
    assert_eq!(
        poll(&settings(false), None, home.path(), &server).await.state,
        ViewState::NeedsSetup
    );
    let blank = Secret::new("  ");
    assert_eq!(
        poll(&settings(false), Some(&blank), home.path(), &server).await.state,
        ViewState::NeedsSetup
    );
    server.assert_untouched();
}

#[tokio::test]
async fn a_pasted_token_is_sent_without_touching_the_local_files() {
    let server = serving("individual_pro.json");
    let home = home_with_files();
    let view = poll(&settings(true), Some(&secret()), home.path(), &server).await;
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(authorization(&server), format!("token {TOKEN}"));
    assert_eq!(view.meta["token_source"], "pasted");
}

#[tokio::test]
async fn with_consent_the_local_token_is_sent_apps_json_first() {
    let server = serving("individual_pro.json");
    let home = home_with_files();
    let view = poll(&settings(true), None, home.path(), &server).await;
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(authorization(&server), format!("token {APPS_TOKEN}"));
    assert_eq!(view.meta["token_source"], "local");
}

#[tokio::test]
async fn a_blank_paste_with_consent_falls_back_to_the_local_token() {
    let server = serving("individual_pro.json");
    let home = tempfile::tempdir().unwrap();
    put(home.path(), "hosts.json", &fixture("hosts.json"));
    let blank = Secret::new("\n");
    poll(&settings(true), Some(&blank), home.path(), &server).await;
    assert_eq!(authorization(&server), format!("token {HOSTS_TOKEN}"));
}

#[tokio::test]
async fn consent_with_no_sign_in_is_an_error_without_a_socket() {
    let server = serving("individual_pro.json");
    let home = tempfile::tempdir().unwrap();
    let view = poll(&settings(true), None, home.path(), &server).await;
    assert_eq!(view.state, ViewState::Error);
    assert_eq!(view.error.unwrap().code, ErrorCode::NotSignedIn);
    server.assert_untouched();
}

#[tokio::test]
async fn a_good_answer_is_a_ready_unofficial_view_stamped_with_the_poll_time() {
    let server = serving("individual_pro.json");
    let home = tempfile::tempdir().unwrap();
    let view = poll(&settings(false), Some(&secret()), home.path(), &server).await;
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(view.id, HarnessId::Copilot);
    let source = view.source.unwrap();
    assert_eq!((source.id.as_str(), source.official), ("copilot_internal_user", false));
    assert_eq!(view.read_at, Some(NOW));
    assert!(!view.stale);
    assert_eq!(view.plan.as_deref(), Some("Individual Pro"));
    assert_eq!(view.windows.len(), 1);
    assert_eq!(view.windows[0].resets_at, Some(FIRST_OF_FEBRUARY));
    assert_eq!(view.facts.len(), 2);
    assert!(view.notices.is_empty() && view.error.is_none());
    assert!(!view.meta.contains_key("reset_derived"));
}

#[tokio::test]
async fn a_derived_reset_is_flagged_in_meta_and_by_a_notice() {
    let server = serving("no_reset.json");
    let home = tempfile::tempdir().unwrap();
    let view = poll(&settings(false), Some(&secret()), home.path(), &server).await;
    assert_eq!(view.meta["reset_derived"], "true");
    let codes: Vec<&str> = view.notices.iter().map(|n| n.code.as_str()).collect();
    assert_eq!(codes, ["copilot_reset_derived"]);
    assert!(crate::NOTICE_CODES.contains(&codes[0]));
    assert!(view.notices[0].params.is_empty());
}

#[tokio::test]
async fn an_unlimited_only_account_is_ready_with_facts_and_no_windows() {
    let server = serving("free_unlimited_chat.json");
    let home = tempfile::tempdir().unwrap();
    let view = poll(&settings(false), Some(&secret()), home.path(), &server).await;
    assert_eq!(view.state, ViewState::Ready);
    assert!(view.windows.is_empty() && !view.stale);
    assert_eq!(view.facts.len(), 2);
}

#[tokio::test]
async fn a_failed_read_is_an_error_view_that_keeps_every_token_out() {
    let home = home_with_files();
    for (status, code) in [
        (401, ErrorCode::Unauthorized),
        (403, ErrorCode::Unauthorized),
        (404, ErrorCode::EndpointChanged),
        (429, ErrorCode::RateLimited),
        (500, ErrorCode::ServerError),
    ] {
        let server = FakeServer::always(move || Reply::json(status, &fixture("error_auth.json")));
        let view = poll(&settings(true), Some(&secret()), home.path(), &server).await;
        assert_eq!(view.state, ViewState::Error, "{status}");
        assert_eq!(view.error.as_ref().unwrap().code, code, "{status}");
        assert!(view.windows.is_empty() && view.facts.is_empty());
        assert!(!serde_json::to_string(&view).unwrap().contains("fabricated"), "no response text");
    }
}

#[tokio::test]
async fn a_body_that_is_not_the_quota_document_is_endpoint_changed() {
    let server = serving("hostile.json");
    let home = tempfile::tempdir().unwrap();
    let view = poll(&settings(false), Some(&secret()), home.path(), &server).await;
    assert_eq!(view.error.unwrap().code, ErrorCode::EndpointChanged);
}

#[tokio::test]
async fn every_good_fixture_gives_a_ready_view_with_no_token_in_it() {
    let home = home_with_files();
    for name in [
        "individual_pro.json",
        "free_finite.json",
        "business.json",
        "overage.json",
        "no_reset.json",
        "free_unlimited_chat.json",
    ] {
        let server = serving(name);
        let view = poll(&settings(true), None, home.path(), &server).await;
        assert_eq!(view.state, ViewState::Ready, "{name}");
    }
}
