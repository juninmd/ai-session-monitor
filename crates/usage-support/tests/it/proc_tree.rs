//! `run_capped`: deadlines, process-tree kills and cancellation.

use std::time::{Duration, Instant};

use usage_model::ErrorCode;
use usage_support::run_capped;

use crate::common::peer::peer;

#[tokio::test]
async fn running_past_the_deadline_is_a_retryable_timeout_with_no_params() {
    let mut spec = peer(&["sleep", "60000"]);
    spec.timeout = Duration::from_millis(300);
    let started = Instant::now();
    let error = run_capped(spec).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::Timeout);
    assert!(error.retryable && error.params.is_empty());
    assert!(started.elapsed() < Duration::from_secs(5), "{:?}", started.elapsed());
}

#[tokio::test]
async fn the_deadline_also_applies_to_a_child_that_floods_its_output() {
    let mut spec = peer(&["flood"]);
    spec.timeout = Duration::from_millis(1500);
    spec.max_stdout = 4096;
    let started = Instant::now();
    let error = run_capped(spec).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::Timeout);
    assert!(started.elapsed() < Duration::from_secs(10), "{:?}", started.elapsed());
}

#[tokio::test]
async fn a_zero_timeout_is_a_timeout_and_not_a_hang() {
    let mut spec = peer(&["sleep", "60000"]);
    spec.timeout = Duration::ZERO;
    let started = Instant::now();
    assert_eq!(run_capped(spec).await.unwrap_err().code, ErrorCode::Timeout);
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[tokio::test]
async fn descendants_holding_the_pipes_open_do_not_hold_the_result() {
    let mut spec = peer(&["orphan-holding-pipe", "4000"]);
    spec.timeout = Duration::from_secs(20);
    let started = Instant::now();
    let out = run_capped(spec).await.unwrap();
    assert_eq!(out.status_code, Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).starts_with("child:"));
    assert!(started.elapsed() < Duration::from_millis(3000), "{:?}", started.elapsed());
}

#[cfg(windows)]
mod windows {
    use std::ffi::OsString;

    use usage_support::CommandSpec;

    use super::*;
    use crate::common::peer::{is_alive, peer_path, read_pid, wait_for_liveness};

    #[tokio::test]
    async fn a_timeout_ends_the_whole_process_tree() {
        let dir = tempfile::tempdir().unwrap();
        let pid_file = dir.path().join("child.pid");
        let mut spec = peer(&["grandchild", pid_file.to_str().unwrap()]);
        spec.timeout = Duration::from_secs(3);
        let run = tokio::spawn(run_capped(spec));

        let pid = read_pid(&pid_file).await;
        assert!(is_alive(pid).await, "the grandchild must be running before the deadline");
        let error = run.await.unwrap().unwrap_err();
        assert_eq!(error.code, ErrorCode::Timeout);
        assert!(wait_for_liveness(pid, false).await, "the grandchild survived the tree kill");
    }

    #[tokio::test]
    async fn dropping_the_future_ends_the_whole_process_tree() {
        let dir = tempfile::tempdir().unwrap();
        let pid_file = dir.path().join("child.pid");
        let mut spec = peer(&["grandchild", pid_file.to_str().unwrap()]);
        spec.timeout = Duration::from_secs(60);
        let run = tokio::spawn(run_capped(spec));

        let pid = read_pid(&pid_file).await;
        assert!(is_alive(pid).await, "the grandchild must be running before the cancel");
        run.abort();
        assert!(run.await.unwrap_err().is_cancelled());
        assert!(wait_for_liveness(pid, false).await, "the grandchild outlived its caller");
    }

    /// A `.cmd` shim in the style of npm's: it forwards `%*` to the real program.
    fn shim(dir: &std::path::Path) -> std::path::PathBuf {
        let script = dir.join("shim for peer.cmd");
        let body = format!("@echo off\r\n\"{}\" args %*\r\n", peer_path());
        std::fs::write(&script, body).unwrap();
        script
    }

    fn shim_spec(script: &std::path::Path, args: &[&str]) -> CommandSpec {
        let mut spec = CommandSpec::new(script);
        spec.args = args.iter().map(OsString::from).collect();
        spec
    }

    async fn through_shim(script: &std::path::Path, args: &[&str]) -> String {
        let out = run_capped(shim_spec(script, args)).await.unwrap();
        String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n")
    }

    #[tokio::test]
    async fn cmd_shims_receive_their_arguments_as_data() {
        let dir = tempfile::tempdir().unwrap();
        let script = shim(dir.path());
        let out = through_shim(&script, &["hello world", "plain", "日本"]).await;
        assert_eq!(out, "[hello world]\n[plain]\n[日本]\n");
    }

    #[tokio::test]
    async fn hostile_arguments_cannot_inject_commands_through_a_cmd_shim() {
        let dir = tempfile::tempdir().unwrap();
        let script = shim(dir.path());
        let marker = dir.path().join("pwned.txt");
        let injections = [
            format!("x\" & echo pwned > \"{}\" & \"", marker.display()),
            format!("x & echo pwned > \"{}\"", marker.display()),
            format!("x | echo pwned > \"{}\"", marker.display()),
            format!("x > \"{}\"", marker.display()),
            format!("x\"^&echo pwned>\"{}\"", marker.display()),
        ];
        for evil in &injections {
            let out = through_shim(&script, &[evil]).await;
            assert!(!marker.exists(), "an argument was executed as a command: {evil}");
            assert_eq!(out, format!("[{evil}]\n"), "the argument must arrive unchanged");
        }
    }

    #[tokio::test]
    async fn arguments_that_cannot_be_escaped_for_cmd_are_refused_and_never_run() {
        let dir = tempfile::tempdir().unwrap();
        let script = shim(dir.path());
        let marker = dir.path().join("pwned.txt");
        let newline = format!("x\r\necho pwned > \"{}\"", marker.display());
        let error = run_capped(shim_spec(&script, &[&newline])).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::SpawnFailed);
        assert!(!marker.exists());
    }

    #[tokio::test]
    async fn percent_signs_are_not_expanded_by_a_cmd_shim() {
        let dir = tempfile::tempdir().unwrap();
        let script = shim(dir.path());
        let args = ["%USERNAME%", "100%", "%cd%", "!var!", "^caret"];
        let expected: String = args.iter().map(|arg| format!("[{arg}]\n")).collect();
        assert_eq!(through_shim(&script, &args).await, expected);
    }
}
