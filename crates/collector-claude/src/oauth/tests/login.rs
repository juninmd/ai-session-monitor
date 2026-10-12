//! Problems with the stored login are found locally, before any request, and are not throttled:
//! signing in again has to show up at the very next poll.

use super::*;
use crate::testenv::{credentials_json, usage_no_model_caps};

fn server() -> FakeServer {
    FakeServer::json(&usage_no_model_caps())
}

#[tokio::test]
async fn no_login_file_means_not_signed_in() {
    let env = installed_env();
    let server = server();
    assert_partial(&poll(&env, &server, NOW).await, "not_signed_in");
    server.assert_untouched();
}

#[tokio::test]
async fn an_expired_login_is_reported_without_a_request() {
    let env = installed_env();
    env.write_credentials((NOW - 1) * 1000);
    let server = server();
    assert_partial(&poll(&env, &server, NOW).await, "oauth_expired");
    server.assert_untouched();
}

#[tokio::test]
async fn the_expiry_instant_is_expired_and_a_millisecond_later_is_not() {
    let env = installed_env();
    let server = server();
    env.write_credentials(NOW * 1000);
    assert_partial(&poll(&env, &server, NOW).await, "oauth_expired");
    server.assert_untouched();

    env.write_credentials(NOW * 1000 + 1);
    let view = poll(&env, &server, NOW).await;
    assert_eq!(server.connections(), 1);
    assert!(view.notices.is_empty(), "{view:?}");
}

#[tokio::test]
async fn an_expiry_of_zero_counts_as_expired() {
    let env = installed_env();
    env.write_credentials(0);
    let server = server();
    assert_partial(&poll(&env, &server, NOW).await, "oauth_expired");
    server.assert_untouched();
}

#[tokio::test]
async fn a_login_without_an_expiry_is_tried_and_the_server_decides() {
    let env = installed_env();
    env.write(
        &env.credentials(),
        format!(r#"{{"claudeAiOauth":{{"accessToken":"{FAKE_TOKEN}"}}}}"#),
    );
    let server = FakeServer::start(|_| Reply::status(401, &[]));
    assert_partial(&poll(&env, &server, NOW).await, "unauthorized");
    assert_eq!(server.connections(), 1);
}

#[tokio::test]
async fn unusable_login_files_are_mapped_and_never_reach_the_network() {
    let oversized = " ".repeat(64 * 1024 + 1);
    let cases = [
        ("not json", "endpoint_changed"),
        (oversized.as_str(), "endpoint_changed"),
        ("[]", "not_signed_in"),
        ("{}", "not_signed_in"),
        (r#"{"claudeAiOauth":null}"#, "not_signed_in"),
        (r#"{"claudeAiOauth":{"accessToken":""}}"#, "not_signed_in"),
        (r#"{"claudeAiOauth":{"accessToken":"short"}}"#, "not_signed_in"),
        (r#"{"claudeAiOauth":{"accessToken":"has spaces in the token"}}"#, "not_signed_in"),
        (r#"{"claudeAiOauth":{"accessToken":12345678}}"#, "not_signed_in"),
    ];
    for (body, reason) in cases {
        let env = installed_env();
        env.write(&env.credentials(), body);
        let server = server();
        assert_partial(&poll(&env, &server, NOW).await, reason);
        server.assert_untouched();
    }
}

#[tokio::test]
async fn a_login_path_that_is_a_directory_is_an_internal_error() {
    let env = installed_env();
    std::fs::create_dir_all(env.credentials()).unwrap();
    let server = server();
    assert_partial(&poll(&env, &server, NOW).await, "internal");
    server.assert_untouched();
}

#[tokio::test]
async fn a_login_file_saved_with_a_bom_is_read() {
    let env = installed_env();
    env.write(&env.credentials(), format!("\u{feff}{}", credentials_json((NOW + 60) * 1000)));
    let server = server();
    let view = poll(&env, &server, NOW).await;
    assert_eq!(server.connections(), 1);
    assert_eq!(view.plan.as_deref(), Some("Max 5x"));
}

#[tokio::test]
async fn a_renewed_login_is_used_at_the_very_next_poll() {
    let env = installed_env();
    let server = server();
    env.write_credentials((NOW - 10) * 1000);
    assert_partial(&poll(&env, &server, NOW).await, "oauth_expired");

    env.write_credentials((NOW + 3600) * 1000);
    let view = poll(&env, &server, NOW).await;
    assert!(view.notices.is_empty(), "{view:?}");
    assert_eq!(server.connections(), 1, "the expiry was not remembered");
}
