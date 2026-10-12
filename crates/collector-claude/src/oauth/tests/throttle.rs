//! The 300 s throttle. Collectors are stateless, so the last attempt lives in a small file next
//! to the shim's data; the clock is the `Ctx`, which makes every edge testable.

use super::*;
use crate::testenv::usage_limits;

#[tokio::test]
async fn a_good_answer_is_reused_for_five_minutes() {
    let env = ready_env();
    let server = FakeServer::json(&usage_limits());
    let first = poll(&env, &server, NOW).await;
    for later in [NOW + 1, NOW + 150, NOW + 299] {
        let again = poll(&env, &server, later).await;
        assert_eq!(
            (again.windows, again.plan),
            (first.windows.clone(), first.plan.clone()),
            "at +{}",
            later - NOW
        );
    }
    assert_eq!(server.connections(), 1);

    poll(&env, &server, NOW + 300).await;
    assert_eq!(server.connections(), 2, "the interval is over");
}

#[tokio::test]
async fn a_failure_is_remembered_so_a_broken_endpoint_is_not_hammered() {
    let env = ready_env();
    let server = FakeServer::start(|_| Reply::status(503, &[]));
    assert_partial(&poll(&env, &server, NOW).await, "server_error");
    assert_partial(&poll(&env, &server, NOW + 120).await, "server_error");
    assert_partial(&poll(&env, &server, NOW + 299).await, "server_error");
    assert_eq!(server.connections(), 1);

    poll(&env, &server, NOW + 300).await;
    assert_eq!(server.connections(), 2);
}

#[tokio::test]
async fn retry_after_stretches_the_wait() {
    let env = ready_env();
    let server = FakeServer::start(|_| Reply::status(429, &[("Retry-After", "1200")]));
    assert_partial(&poll(&env, &server, NOW).await, "rate_limited");
    assert_partial(&poll(&env, &server, NOW + 1199).await, "rate_limited");
    assert_eq!(server.connections(), 1);
    poll(&env, &server, NOW + 1200).await;
    assert_eq!(server.connections(), 2);
}

#[tokio::test]
async fn a_short_retry_after_never_shortens_the_wait() {
    let env = ready_env();
    let server = FakeServer::start(|_| Reply::status(429, &[("Retry-After", "5")]));
    poll(&env, &server, NOW).await;
    poll(&env, &server, NOW + 299).await;
    assert_eq!(server.connections(), 1);
}

#[tokio::test]
async fn a_huge_retry_after_is_capped_at_an_hour() {
    let env = ready_env();
    let server = FakeServer::start(|_| Reply::status(429, &[("Retry-After", "99999999")]));
    poll(&env, &server, NOW).await;
    poll(&env, &server, NOW + 3599).await;
    assert_eq!(server.connections(), 1);
    poll(&env, &server, NOW + 3600).await;
    assert_eq!(server.connections(), 2);
}

#[tokio::test]
async fn an_endpoint_that_recovers_shows_up_after_the_wait() {
    let env = ready_env();
    let failing = FakeServer::start(|_| Reply::status(500, &[]));
    assert_partial(&poll(&env, &failing, NOW).await, "server_error");

    let healthy = FakeServer::json(&usage_limits());
    let view = poll(&env, &healthy, NOW + 300).await;
    assert_eq!(view.windows.len(), 4);
    assert!(view.notices.is_empty());
}

#[tokio::test]
async fn a_cache_stamped_in_the_future_does_not_block_polling() {
    let env = ready_env();
    let server = FakeServer::json(&usage_limits());
    poll(&env, &server, NOW + 1000).await;
    // The clock was set back: the stamp proves nothing about how long ago the request was.
    poll(&env, &server, NOW).await;
    assert_eq!(server.connections(), 2);
}

#[tokio::test]
async fn a_damaged_cache_file_is_replaced_by_a_fresh_attempt() {
    let env = ready_env();
    env.write(&env.oauth_cache(), "{ torn");
    let server = FakeServer::json(&usage_limits());
    poll(&env, &server, NOW).await;
    assert_eq!(server.connections(), 1);
    poll(&env, &server, NOW + 10).await;
    assert_eq!(server.connections(), 1, "the rewritten cache is honoured");
}

#[tokio::test]
async fn the_cache_file_holds_the_outcome_and_no_secret() {
    let env = ready_env();
    let server = FakeServer::json(&usage_limits());
    poll(&env, &server, NOW).await;
    let text = std::fs::read_to_string(env.oauth_cache()).unwrap();
    assert!(!text.contains(FAKE_TOKEN) && !text.contains("fake-refresh"), "{text}");
    let doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!((doc["v"].as_i64(), doc["attemptedAt"].as_i64()), (Some(1), Some(NOW)));
}

#[tokio::test]
async fn an_unwritable_cache_only_costs_an_extra_request() {
    let env = ready_env();
    // A directory where the file belongs: the throttle cannot be stored, but the poll still works.
    std::fs::create_dir_all(env.oauth_cache()).unwrap();
    let server = FakeServer::json(&usage_limits());
    let view = poll(&env, &server, NOW).await;
    assert_eq!(view.windows.len(), 4);
    poll(&env, &server, NOW + 1).await;
    assert_eq!(server.connections(), 2, "nothing was remembered");
}
