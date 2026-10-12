//! End-to-end through the built binary: what the shim records (and refuses to record).

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "tests assert by panicking")]

mod common;

use std::process::Stdio;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use common::{Harness, payload_with_limits, run};
use serde_json::json;

fn now() -> i64 {
    i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()).unwrap()
}

#[test]
fn records_only_the_rate_limits_object_and_prints_nothing() {
    let harness = Harness::new();
    let before = now();
    let outcome = harness.run(payload_with_limits().to_string().as_bytes());

    assert_eq!(outcome.code, Some(0));
    assert!(outcome.stdout.is_empty() && outcome.stderr.is_empty(), "no chain, no output");
    let doc = harness.read_out().expect("file written");
    assert_eq!(doc["v"], 1);
    let captured = doc["capturedAt"].as_i64().unwrap();
    assert!((before..=now()).contains(&captured), "capturedAt is the capture time");
    assert_eq!(doc["rateLimits"], payload_with_limits()["rate_limits"]);

    let raw = std::fs::read_to_string(harness.out()).unwrap();
    for private in
        ["projeto-secreto", "transcript-marker", "session-marker", "model-marker", "12.34"]
    {
        assert!(!raw.contains(private), "the recorded file must not contain {private}");
    }
}

#[test]
fn a_payload_without_rate_limits_records_null() {
    let harness = Harness::new();
    let outcome = harness.run(br#"{"cwd":"/x","model":{"id":"m"}}"#);

    assert_eq!(outcome.code, Some(0));
    let doc = harness.read_out().expect("file written");
    assert_eq!(doc["rateLimits"], serde_json::Value::Null);
    assert_eq!(doc["v"], 1);
}

#[test]
fn real_limits_survive_the_next_payload_that_has_none() {
    let harness = Harness::new();
    harness.run(payload_with_limits().to_string().as_bytes());
    let before = harness.read_out().unwrap();

    let outcome = harness.run(br#"{"cwd":"/new/session"}"#);

    assert_eq!(outcome.code, Some(0));
    assert_eq!(harness.read_out().unwrap(), before, "session start must not erase the data");
}

#[test]
fn invalid_json_writes_nothing_and_still_exits_zero() {
    let harness = Harness::new();
    for garbage in
        [&b"this is not json"[..], b"", b"{\"rate_limits\":", b"[1,2,3]", b"\xff\xfe\x00"]
    {
        let outcome = harness.run(garbage);
        assert_eq!(outcome.code, Some(0), "{garbage:?}");
        assert!(outcome.stdout.is_empty() && outcome.stderr.is_empty(), "{garbage:?}");
    }
    assert!(harness.read_out().is_none(), "nothing may be recorded for a non-payload");
}

#[test]
fn invalid_json_leaves_an_existing_recording_untouched() {
    let harness = Harness::new();
    harness.run(payload_with_limits().to_string().as_bytes());
    let before = std::fs::read(harness.out()).unwrap();

    harness.run(b"garbage");

    assert_eq!(std::fs::read(harness.out()).unwrap(), before);
}

#[test]
fn oversized_stdin_writes_nothing_and_still_exits_zero() {
    let harness = Harness::new();
    let mut huge = payload_with_limits().to_string().into_bytes();
    huge.resize(3 * 1024 * 1024, b' ');

    let outcome = harness.run(&huge);

    assert_eq!(outcome.code, Some(0));
    assert!(outcome.stdout.is_empty());
    assert!(harness.read_out().is_none(), "an oversized payload is not recorded");
}

#[test]
fn unchanged_content_is_not_rewritten_while_the_file_is_young() {
    let harness = Harness::new();
    let payload = payload_with_limits().to_string();
    harness.run(payload.as_bytes());
    let first = std::fs::metadata(harness.out()).unwrap().modified().unwrap();

    harness.run(payload.as_bytes());

    // A rewrite would move the modification time even within the same second.
    assert_eq!(std::fs::metadata(harness.out()).unwrap().modified().unwrap(), first);
}

#[test]
fn unchanged_content_is_refreshed_once_the_file_is_older_than_five_seconds() {
    let harness = Harness::new();
    let payload = payload_with_limits();
    // Same limits as the next payload, but captured long ago and written 30 s ago.
    let aged = json!({"v": 1, "capturedAt": 1_000_000_000, "rateLimits": payload["rate_limits"]});
    std::fs::create_dir_all(harness.out().parent().unwrap()).unwrap();
    std::fs::write(harness.out(), aged.to_string()).unwrap();
    let file = std::fs::File::options().write(true).open(harness.out()).unwrap();
    file.set_modified(SystemTime::now() - Duration::from_secs(30)).unwrap();
    drop(file);

    harness.run(payload.to_string().as_bytes());

    let doc = harness.read_out().unwrap();
    assert_eq!(doc["rateLimits"], payload["rate_limits"]);
    assert!(doc["capturedAt"].as_i64().unwrap() >= now() - 60, "capturedAt was refreshed");
}

#[test]
fn a_file_stamped_in_the_future_is_rewritten_not_trusted() {
    let harness = Harness::new();
    let payload = payload_with_limits();
    let skewed = json!({"v": 1, "capturedAt": 1_000_000_000, "rateLimits": payload["rate_limits"]});
    std::fs::create_dir_all(harness.out().parent().unwrap()).unwrap();
    std::fs::write(harness.out(), skewed.to_string()).unwrap();
    let file = std::fs::File::options().write(true).open(harness.out()).unwrap();
    file.set_modified(SystemTime::now() + Duration::from_secs(3600)).unwrap();
    drop(file);

    harness.run(payload.to_string().as_bytes());

    assert!(harness.read_out().unwrap()["capturedAt"].as_i64().unwrap() >= now() - 60);
}

#[test]
fn changed_limits_are_written_immediately() {
    let harness = Harness::new();
    harness.run(payload_with_limits().to_string().as_bytes());
    let mut next = payload_with_limits();
    next["rate_limits"]["five_hour"]["used_percentage"] = json!(24.0);

    harness.run(next.to_string().as_bytes());

    assert_eq!(harness.read_out().unwrap()["rateLimits"]["five_hour"]["used_percentage"], 24.0);
}

#[test]
fn unicode_in_values_and_a_unicode_output_path_work() {
    let harness = Harness::new();
    let payload =
        json!({"rate_limits": {"spend_limit": {"used_percentage": 3, "period": "mensal ção 🗓️"}}});

    let outcome = harness.run(payload.to_string().as_bytes());

    assert_eq!(outcome.code, Some(0));
    assert_eq!(harness.read_out().unwrap()["rateLimits"]["spend_limit"]["period"], "mensal ção 🗓️");
}

#[test]
fn a_closed_stdin_is_harmless_and_records_nothing() {
    let harness = Harness::new();
    let mut command = harness.command();
    command.stdin(Stdio::null());

    let output = command.output().unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    assert!(harness.read_out().is_none(), "no input is not a payload");
}

#[test]
fn no_arguments_at_all_is_harmless() {
    let command = std::process::Command::new(env!("CARGO_BIN_EXE_aism-statusline-shim"));
    let outcome = run(command, payload_with_limits().to_string().as_bytes());

    assert_eq!(outcome.code, Some(0));
    assert!(outcome.stdout.is_empty() && outcome.stderr.is_empty());
}

#[test]
fn an_unwritable_output_location_is_harmless() {
    let harness = Harness::new();
    let blocker = harness.dir.path().join("a file, not a directory");
    std::fs::write(&blocker, b"x").unwrap();
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_aism-statusline-shim"));
    command.arg("--out").arg(blocker.join("claude").join("rate_limits.json"));

    let outcome = run(command, payload_with_limits().to_string().as_bytes());

    assert_eq!(outcome.code, Some(0));
    assert!(outcome.stdout.is_empty() && outcome.stderr.is_empty());
}

#[test]
fn a_tilde_in_the_out_path_is_expanded_from_the_home_directory() {
    let harness = Harness::new();
    let home = harness.dir.path().join("home dir");
    std::fs::create_dir_all(&home).unwrap();
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_aism-statusline-shim"));
    command.args(["--out", "~/AppData/claude/rate_limits.json"]);
    command.env("USERPROFILE", &home).env("HOME", &home);

    run(command, payload_with_limits().to_string().as_bytes());

    assert!(home.join("AppData/claude/rate_limits.json").is_file());
}
