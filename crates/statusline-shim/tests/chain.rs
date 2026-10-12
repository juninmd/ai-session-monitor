//! End-to-end through the built binary: chaining to the user's previous status line command.
//! The "previous command" is the `shim_chain_peer` test binary, named relatively so no shell
//! quoting is involved; whichever shell the shim picks on this machine runs it.

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "tests assert by panicking")]

mod common;

use std::process::Command;
use std::time::Duration;

use common::{Harness, payload_with_limits, run};

const SHIM: &str = env!("CARGO_BIN_EXE_aism-statusline-shim");

#[test]
fn the_chained_output_is_forwarded_and_it_gets_the_same_stdin() {
    let harness = Harness::new();
    let peer = harness.install_peer();
    let payload = payload_with_limits().to_string();

    let outcome = harness.run_chained(&format!("{peer} --echo"), payload.as_bytes());

    assert_eq!(outcome.code, Some(0));
    assert_eq!(outcome.text().trim_end(), format!("PEER:{payload}"));
    assert!(harness.read_out().is_some(), "recording still happens when chaining");
}

#[test]
fn unicode_survives_the_round_trip_through_the_chain() {
    let harness = Harness::new();
    let peer = harness.install_peer();
    let payload = r#"{"workspace":{"current_dir":"C:/Users/João Pereira/projeção ✓"}}"#;

    let outcome = harness.run_chained(&format!("{peer} --echo"), payload.as_bytes());

    assert_eq!(outcome.stdout, format!("PEER:{payload}").into_bytes());
}

#[test]
fn chaining_works_without_an_out_file() {
    let harness = Harness::new();
    let peer = harness.install_peer();
    let chain = harness.write_chain(&format!("{peer} --len"));
    let mut command = Command::new(SHIM);
    command.current_dir(harness.dir.path()).arg("--chain").arg(chain);

    let outcome = run(command, b"12345");

    assert_eq!(outcome.text(), "LEN=5");
    assert!(harness.read_out().is_none());
}

#[test]
fn a_closed_stdin_reaches_the_chain_as_an_empty_input() {
    let harness = Harness::new();
    let peer = harness.install_peer();
    let mut command = harness.command();
    command.arg("--chain").arg(harness.write_chain(&format!("{peer} --len")));
    command.stdin(std::process::Stdio::null());

    let output = command.output().unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"LEN=0");
    assert!(harness.read_out().is_none(), "nothing to record");
}

#[test]
fn a_failing_chain_prints_nothing_and_the_shim_still_exits_zero() {
    let harness = Harness::new();
    let peer = harness.install_peer();

    let outcome = harness.run_chained(&format!("{peer} --echo --exit 3"), b"{}");

    assert_eq!(outcome.code, Some(0));
    assert!(outcome.stdout.is_empty(), "Claude Code blanks the line on a non-zero exit");
    assert!(outcome.stderr.is_empty());
}

#[test]
fn a_command_that_does_not_exist_is_harmless() {
    let harness = Harness::new();

    let outcome = harness.run_chained("definitely-not-a-real-command-xyz --flag", b"{}");

    assert_eq!(outcome.code, Some(0));
    assert!(outcome.stdout.is_empty() && outcome.stderr.is_empty());
}

#[test]
fn the_chained_stderr_never_reaches_the_status_line() {
    let harness = Harness::new();
    let peer = harness.install_peer();

    let outcome = harness.run_chained(&format!("{peer} --err oops --len"), b"abc");

    assert_eq!(outcome.text(), "LEN=3");
    assert!(outcome.stderr.is_empty());
}

#[test]
fn a_missing_or_broken_chain_file_prints_nothing() {
    let harness = Harness::new();
    let dir = harness.dir.path();
    std::fs::write(dir.join("broken.json"), "{ not json").unwrap();
    for chain in [dir.join("absent.json"), dir.join("broken.json")] {
        let mut command = harness.command();
        command.arg("--chain").arg(chain);
        let outcome = run(command, payload_with_limits().to_string().as_bytes());
        assert_eq!(outcome.code, Some(0));
        assert!(outcome.stdout.is_empty() && outcome.stderr.is_empty());
    }
}

#[test]
fn an_oversized_payload_is_not_recorded_but_still_reaches_the_chain_whole() {
    let harness = Harness::new();
    let peer = harness.install_peer();
    let mut huge = payload_with_limits().to_string().into_bytes();
    huge.resize(3 * 1024 * 1024, b' ');

    let outcome = harness.run_chained(&format!("{peer} --len"), &huge);

    assert_eq!(outcome.code, Some(0));
    assert_eq!(outcome.text(), format!("LEN={}", 3 * 1024 * 1024));
    assert!(harness.read_out().is_none());
}

#[test]
fn output_far_beyond_a_status_line_is_dropped() {
    let harness = Harness::new();
    let peer = harness.install_peer();

    let outcome = harness.run_chained(&format!("{peer} --big 200000"), b"{}");

    assert_eq!(outcome.code, Some(0));
    assert!(outcome.stdout.is_empty());
}

#[test]
fn output_is_forwarded_up_to_the_cap_and_dropped_one_byte_beyond() {
    let harness = Harness::new();
    let peer = harness.install_peer();

    let at_cap = harness.run_chained(&format!("{peer} --big 65536"), b"{}");
    let beyond = harness.run_chained(&format!("{peer} --big 65537"), b"{}");

    assert_eq!(at_cap.stdout.len(), 64 * 1024);
    assert!(beyond.stdout.is_empty(), "a status line this long is not one");
}

#[test]
fn a_command_still_running_past_the_output_cap_is_ended_at_once() {
    let harness = Harness::new();
    let peer = harness.install_peer();
    let marker = harness.dir.path().join("survived.marker");

    // It ignores the pipe the shim closes at the cap and would finish at 4 s, inside the 5 s
    // budget, unless the cap ends it first. A shell that takes over a second to start hides a
    // regression here instead of failing.
    let outcome = harness.run_chained(
        &format!("{peer} --big 200000 --sleep-ms 4000 --marker survived.marker"),
        b"{}",
    );

    assert_eq!(outcome.code, Some(0));
    assert!(outcome.stdout.is_empty());
    std::thread::sleep(Duration::from_millis(6500).saturating_sub(outcome.elapsed));
    assert!(!marker.exists(), "the command was left running past the cap");
}

#[test]
fn a_hung_chain_is_killed_after_five_seconds_and_does_not_outlive_the_shim() {
    let harness = Harness::new();
    let peer = harness.install_peer();
    let marker = harness.dir.path().join("survived.marker");

    let outcome =
        harness.run_chained(&format!("{peer} --sleep-ms 7000 --marker survived.marker"), b"{}");

    assert_eq!(outcome.code, Some(0));
    assert!(outcome.stdout.is_empty());
    assert!(
        outcome.elapsed >= Duration::from_secs(5),
        "waited the full budget: {:?}",
        outcome.elapsed
    );
    assert!(outcome.elapsed < Duration::from_secs(9), "but not much longer: {:?}", outcome.elapsed);
    // The peer would write the marker at 7 s; give it until 9 s to prove it is really gone.
    std::thread::sleep(Duration::from_millis(9000).saturating_sub(outcome.elapsed));
    assert!(!marker.exists(), "the hung command was left running");
}

#[test]
fn the_chained_command_sees_an_incremented_depth_guard() {
    let harness = Harness::new();
    let peer = harness.install_peer();

    let outcome = harness.run_chained(&format!("{peer} --env AISM_SHIM_DEPTH"), b"{}");

    assert_eq!(outcome.text(), "AISM_SHIM_DEPTH=1");
}

#[test]
fn a_chain_that_points_back_at_the_shim_terminates() {
    let harness = Harness::new();
    let local = format!("shim-copy{}", std::env::consts::EXE_SUFFIX);
    std::fs::copy(SHIM, harness.dir.path().join(&local)).unwrap();
    // chain.json names itself: shim -> shim -> shim -> ... without the guard.
    let chain = harness.dir.path().join("chain.json");
    std::fs::write(&chain, format!(r#"{{"command":"./{local} --chain chain.json"}}"#)).unwrap();
    let mut command = Command::new(harness.dir.path().join(&local));
    command.current_dir(harness.dir.path()).arg("--chain").arg(&chain);

    let outcome = run(command, b"{}");

    assert_eq!(outcome.code, Some(0));
    assert!(outcome.stdout.is_empty());
    assert!(outcome.elapsed < Duration::from_secs(20), "{:?}", outcome.elapsed);
}

/// With Git for Windows hidden from the shim, the documented fallback shell runs the command.
#[cfg(windows)]
#[test]
fn powershell_runs_the_chain_when_git_bash_is_not_available() {
    let harness = Harness::new();
    let peer = harness.install_peer();
    let chain = harness.write_chain(&format!("{peer} --echo"));
    let nothing = harness.dir.path().join("no-git-here");
    std::fs::create_dir_all(&nothing).unwrap();
    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_owned());
    let mut command = harness.command();
    command
        .arg("--chain")
        .arg(chain)
        .env_remove("CLAUDE_CODE_GIT_BASH_PATH")
        .env("PATH", format!(r"{system_root}\System32\WindowsPowerShell\v1.0"))
        .env("ProgramFiles", &nothing)
        .env("ProgramW6432", &nothing)
        .env("ProgramFiles(x86)", &nothing)
        .env("LOCALAPPDATA", &nothing);

    let outcome = run(command, br#"{"k":"v"}"#);

    assert_eq!(outcome.code, Some(0));
    assert_eq!(outcome.text().trim_end(), r#"PEER:{"k":"v"}"#);
}
