//! End-to-end: a chained command that leaves a background job holding its stdout open. Claude Code
//! takes the line a moment after the command exits, whatever still holds the pipe, so the shim
//! must not make such a script worse than it is without the shim. The job is a copy of the
//! `shim_chain_peer` test binary (see its `--straggler-ms`), so no shell syntax is involved.
//!
//! Timing is judged from the command's own exit (`--stamp`) to the shim's, never from the start:
//! how long a shell takes to launch varies by seconds on a busy machine.

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "tests assert by panicking")]

mod common;

use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use common::Harness;

/// Far beyond the 5 s chain budget: only the release (or this cap) ends the job.
const JOB_CAP_MS: u64 = 20_000;
/// The shim settles 500 ms after the command exits; four times that is slack for a busy machine.
const GRACE: Duration = Duration::from_secs(2);

/// A test must not end while the job still runs: it holds the harness' pipes and the copied
/// program open, and would outlive the test binary.
fn appears_within(path: &Path, limit: Duration) -> bool {
    let until = Instant::now() + limit;
    while Instant::now() < until && !path.exists() {
        std::thread::sleep(Duration::from_millis(20));
    }
    path.exists()
}

/// How long after the command's own exit the shim itself was gone.
fn lag_after_command_exit(harness: &Harness, shim_exit: SystemTime) -> Duration {
    let stamp = std::fs::read_to_string(harness.dir.path().join("peer.exit")).unwrap();
    let command_exit = UNIX_EPOCH + Duration::from_millis(stamp.trim().parse().unwrap());
    shim_exit.duration_since(command_exit).unwrap_or_default()
}

#[test]
fn a_background_job_holding_stdout_open_does_not_hold_the_line_back() {
    let harness = Harness::new();
    let peer = harness.install_peer();
    let release = harness.dir.path().join("release");
    let command = format!(
        "{peer} --echo --stamp peer.exit --marker job.ended --until release \
         --straggler-ms {JOB_CAP_MS}"
    );
    let mut shim_exit = SystemTime::now();

    // The job lives until the shim is gone, so a shim that waits for the pipe can only give up
    // at its own 5 s budget.
    let outcome = harness.run_chained_then(&command, b"{}", || {
        shim_exit = SystemTime::now();
        std::fs::write(&release, b"go").unwrap();
    });

    assert_eq!(outcome.code, Some(0));
    assert_eq!(outcome.text().trim_end(), "PEER:{}", "the finished command's line was dropped");
    let lag = lag_after_command_exit(&harness, shim_exit);
    assert!(lag < GRACE, "the shim lingered {lag:?} after the command exited");
    assert!(
        appears_within(&harness.dir.path().join("job.ended"), Duration::from_secs(10)),
        "the job was killed, but it is the user's to finish"
    );
}

#[test]
fn a_background_job_outliving_the_chain_budget_on_its_own_does_not_blank_the_line() {
    let harness = Harness::new();
    let peer = harness.install_peer();
    // No release: the job sleeps past the 5 s budget by itself, as a slow cache refresh would.
    let command = format!("{peer} --echo --stamp peer.exit --marker job.ended --straggler-ms 6000");
    let mut shim_exit = SystemTime::now();

    let outcome = harness.run_chained_then(&command, b"{}", || shim_exit = SystemTime::now());

    assert_eq!(outcome.code, Some(0));
    assert_eq!(outcome.text().trim_end(), "PEER:{}", "the finished command's line was dropped");
    let lag = lag_after_command_exit(&harness, shim_exit);
    assert!(lag < GRACE, "the shim lingered {lag:?} after the command exited");
    assert!(appears_within(&harness.dir.path().join("job.ended"), Duration::from_secs(10)));
}

#[test]
fn a_command_without_a_background_job_is_not_held_after_it_exits() {
    let harness = Harness::new();
    let command = format!("{} --echo --stamp peer.exit", harness.install_peer());
    let mut shim_exit = SystemTime::now();

    let outcome = harness.run_chained_then(&command, b"{}", || shim_exit = SystemTime::now());

    assert_eq!(outcome.text().trim_end(), "PEER:{}");
    // Generous on purpose; that no grace period applies at all is pinned in collect.rs.
    let lag = lag_after_command_exit(&harness, shim_exit);
    assert!(lag < GRACE, "the shim lingered {lag:?} after the command exited");
}
