//! Every way the app-server conversation can fail, and the error code each one becomes. With no
//! rollout files around, the fallback has nothing to offer and the original error shows.

use std::time::{Duration, Instant};

use usage_model::settings::CodexSettings;
use usage_model::{ErrorCode, ViewState};

use crate::common::process::wait_until_gone;
use crate::common::{PATIENT, Rig, assert_failure};

async fn failing(mode: &str, code: ErrorCode, retryable: bool) -> usage_model::HarnessError {
    let rig = Rig::new(mode);
    let view = rig.collect().await;
    assert_failure(&view, code, retryable).clone()
}

fn param<'a>(error: &'a usage_model::HarnessError, key: &str) -> Option<&'a str> {
    error.params.get(key).map(String::as_str)
}

#[tokio::test]
async fn a_missing_binary_is_binary_not_found_and_names_no_path() {
    let rig = Rig::without_binary();
    let view = rig.collect().await;
    let error = assert_failure(&view, ErrorCode::BinaryNotFound, false);
    assert!(error.params.is_empty(), "{:?}", error.params);
}

#[tokio::test]
async fn a_relative_path_or_an_unknown_name_is_binary_not_found() {
    let rig = Rig::new("ok");
    let name = rig.exe.file_name().unwrap().to_string_lossy().into_owned();
    // A relative path would mean "the current directory"; a bare name goes to PATH, where this one
    // is nowhere. Both name a program that does exist next to the fake, and neither may start it.
    let separator = std::path::MAIN_SEPARATOR;
    let unknown = "no-such-program-for-ai-session-monitor-tests".to_owned();
    for wanted in [format!(".{separator}{name}"), format!("sub{separator}{name}"), unknown] {
        let settings = CodexSettings { binary_path: Some(wanted.clone()), ..rig.settings() };
        let view = rig.collect_with(&settings).await;
        assert_failure(&view, ErrorCode::BinaryNotFound, false);
        assert!(rig.pid().is_none(), "{wanted}: nothing may have been started");
    }
}

#[cfg(windows)]
#[tokio::test]
async fn a_file_that_is_not_a_program_is_a_spawn_failure() {
    let rig = Rig::without_binary();
    std::fs::write(&rig.exe, b"this is text, not a program").unwrap();
    let view = rig.collect().await;
    assert_failure(&view, ErrorCode::SpawnFailed, false);
}

#[tokio::test]
async fn a_signed_out_account_is_not_signed_in_whichever_call_says_so() {
    for mode in ["not_signed_in", "not_signed_in_rpc"] {
        failing(mode, ErrorCode::NotSignedIn, false).await;
    }
}

#[tokio::test]
async fn backend_statuses_keep_only_their_number() {
    let cases = [
        ("backend_401", ErrorCode::NotSignedIn, false, "401"),
        ("backend_403", ErrorCode::Unauthorized, false, "403"),
        ("backend_429", ErrorCode::RateLimited, true, "429"),
        ("backend_500", ErrorCode::ServerError, true, "500"),
        ("backend_503", ErrorCode::ServerError, true, "503"),
        ("backend_418", ErrorCode::BadResponse, false, "418"),
    ];
    for (mode, code, retryable, status) in cases {
        let error = failing(mode, code, retryable).await;
        assert_eq!(param(&error, "httpStatus"), Some(status), "{mode}");
        assert_eq!(param(&error, "phase"), Some("rate_limits"), "{mode}");
        let text = format!("{:?}", error.params);
        assert!(!text.contains("sentinel") && !text.contains("invalid"), "{mode}: {text}");
    }
}

#[tokio::test]
async fn a_backend_that_cannot_be_reached_is_a_retryable_network_error() {
    let error = failing("offline", ErrorCode::Network, true).await;
    assert_eq!(param(&error, "phase"), Some("rate_limits"));
}

#[tokio::test]
async fn a_refused_handshake_is_not_retryable() {
    for mode in ["bad_handshake", "bad_handshake_result"] {
        let error = failing(mode, ErrorCode::HandshakeFailed, false).await;
        assert_eq!(param(&error, "phase"), Some("initialize"), "{mode}");
    }
}

#[tokio::test]
async fn lines_that_are_not_the_protocol_are_a_protocol_error() {
    for mode in ["garbage_line", "huge_line"] {
        let error = failing(mode, ErrorCode::ProtocolError, false).await;
        assert_eq!(param(&error, "phase"), Some("initialize"), "{mode}");
    }
}

#[tokio::test]
async fn a_server_that_dies_is_a_retryable_handshake_failure() {
    let early = failing("exits_early", ErrorCode::HandshakeFailed, true).await;
    assert_eq!(param(&early, "phase"), Some("initialize"));
    let midway = failing("exits_midway", ErrorCode::HandshakeFailed, true).await;
    assert_eq!(param(&midway, "phase"), Some("rate_limits"));
}

#[tokio::test]
async fn an_answer_of_the_wrong_shape_is_the_endpoint_changing() {
    let error = failing("bad_shape", ErrorCode::EndpointChanged, false).await;
    assert_eq!(param(&error, "phase"), Some("rate_limits"));
}

#[tokio::test]
async fn silence_ends_in_a_retryable_timeout_and_the_process_is_gone() {
    for mode in ["hang", "hang_after_init"] {
        let rig = Rig::new(mode);
        let started = Instant::now();
        let view = rig.collect_within(Duration::from_secs(1)).await;
        assert_failure(&view, ErrorCode::Timeout, true);
        assert!(started.elapsed() < Duration::from_secs(20), "{mode}: {:?}", started.elapsed());
        let pid = rig.pid().expect("the fake started");
        assert!(wait_until_gone(pid).await, "{mode}: the app-server was left running");
    }
}

#[tokio::test]
async fn the_budget_covers_the_whole_conversation_not_each_call() {
    // Three replies of 600 ms each: no single one is slow, together they overrun 1.5 s.
    let rig = Rig::new("slow");
    let view = rig.collect_within(Duration::from_millis(1_500)).await;
    assert_failure(&view, ErrorCode::Timeout, true);

    let patient = Rig::new("slow");
    assert_eq!(patient.collect_within(PATIENT).await.state, ViewState::Ready);
}

#[tokio::test]
async fn a_zero_budget_times_out_without_hanging() {
    let rig = Rig::new("ok");
    let started = Instant::now();
    let view = rig.collect_within(Duration::ZERO).await;
    assert_failure(&view, ErrorCode::Timeout, true);
    assert!(started.elapsed() < Duration::from_secs(20));
}
