//! `run_capped`: output, caps, stdin, exit codes, arguments and start failures.

use std::ffi::OsString;
use std::time::{Duration, Instant};

use usage_model::ErrorCode;
use usage_support::{CommandSpec, ProcOutput, run_capped};

use crate::common::peer::{peer, peer_path};

async fn run(args: &[&str]) -> ProcOutput {
    run_capped(peer(args)).await.unwrap()
}

/// Bytes that are never all the same, so a slice taken from the wrong place is noticed.
fn pattern(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 251) as u8).collect()
}

#[tokio::test]
async fn stdout_and_stderr_are_captured_separately_with_the_exit_code() {
    let out = run(&["print-both", "10", "5"]).await;
    assert_eq!(out.status_code, Some(0));
    assert_eq!(out.stdout, vec![b'x'; 10]);
    assert_eq!(out.stderr, vec![b'x'; 5]);
    assert!(!out.truncated);
}

#[tokio::test]
async fn a_nonzero_exit_is_data_not_an_error() {
    for code in [1, 3, 255] {
        let out = run(&["exit", &code.to_string()]).await;
        assert_eq!(out.status_code, Some(code));
        assert!(out.stdout.is_empty() && out.stderr.is_empty() && !out.truncated);
    }
}

#[tokio::test]
async fn output_beyond_the_cap_is_dropped_and_the_head_is_kept() {
    let input = pattern(100_000);
    let mut spec = peer(&["cat"]);
    spec.stdin = Some(input.clone());
    spec.max_stdout = 1000;
    let out = run_capped(spec).await.unwrap();
    assert_eq!(out.stdout, input[..1000]);
    assert!(out.truncated);
    assert_eq!(out.status_code, Some(0));
}

#[tokio::test]
async fn the_cap_boundary_is_exact() {
    for (size, cap, truncated) in
        [(1000, 1000, false), (1001, 1000, true), (999, 1000, false), (1, 0, true), (0, 0, false)]
    {
        let mut spec = peer(&["print", &size.to_string()]);
        spec.max_stdout = cap;
        let out = run_capped(spec).await.unwrap();
        assert_eq!(out.stdout.len(), size.min(cap), "{size}/{cap}");
        assert_eq!(out.truncated, truncated, "{size}/{cap}");
    }
}

#[tokio::test]
async fn each_stream_has_its_own_cap_and_either_one_marks_truncation() {
    let mut spec = peer(&["print-both", "5000000", "5000000"]);
    (spec.max_stdout, spec.max_stderr) = (1000, 500);
    let out = run_capped(spec).await.unwrap();
    assert_eq!((out.stdout.len(), out.stderr.len()), (1000, 500));
    assert!(out.truncated);

    let mut spec = peer(&["print-both", "10", "600"]);
    (spec.max_stdout, spec.max_stderr) = (1000, 500);
    let out = run_capped(spec).await.unwrap();
    assert_eq!((out.stdout.len(), out.stderr.len()), (10, 500));
    assert!(out.truncated, "stderr alone going over the cap still counts");
}

#[tokio::test]
async fn caps_and_timeouts_beyond_the_limits_are_clamped_not_trusted() {
    let mut spec = peer(&["print", "10"]);
    (spec.max_stdout, spec.max_stderr, spec.timeout) = (usize::MAX, usize::MAX, Duration::MAX);
    assert_eq!(run_capped(spec).await.unwrap().stdout.len(), 10);
}

#[tokio::test]
async fn stdin_is_delivered_and_then_closed() {
    let mut spec = peer(&["cat"]);
    spec.stdin = Some("olá\nmundo 🔑\n".as_bytes().to_vec());
    let out = run_capped(spec).await.unwrap();
    assert_eq!(out.stdout, "olá\nmundo 🔑\n".as_bytes());
}

#[tokio::test]
async fn large_input_and_large_output_at_once_do_not_deadlock() {
    let input = pattern(8 << 20);
    let mut spec = peer(&["cat"]);
    spec.stdin = Some(input.clone());
    spec.max_stdout = 16 << 20;
    let out = run_capped(spec).await.unwrap();
    assert_eq!(out.stdout.len(), input.len());
    assert!(out.stdout == input && !out.truncated);
}

#[tokio::test]
async fn without_stdin_the_child_sees_end_of_input_at_once() {
    let started = Instant::now();
    let out = run(&["cat"]).await;
    assert_eq!(out.status_code, Some(0));
    assert!(out.stdout.is_empty());
    assert!(started.elapsed() < Duration::from_secs(10));
}

#[tokio::test]
async fn a_child_that_never_reads_its_stdin_is_not_an_error() {
    let mut spec = peer(&["exit", "0"]);
    spec.stdin = Some(pattern(5 << 20));
    let out = run_capped(spec).await.unwrap();
    assert_eq!(out.status_code, Some(0));
}

#[tokio::test]
async fn a_chatty_child_with_small_caps_finishes_instead_of_blocking_on_its_pipes() {
    let mut spec = peer(&["print-both", "6000000", "6000000"]);
    (spec.max_stdout, spec.max_stderr) = (16, 16);
    let started = Instant::now();
    let out = run_capped(spec).await.unwrap();
    assert!(out.truncated && out.status_code == Some(0));
    assert!(started.elapsed() < Duration::from_secs(15));
}

#[tokio::test]
async fn arguments_travel_as_an_array_and_arrive_untouched() {
    let tricky = [
        "a b",
        "\"quoted\"",
        r"back\slash\",
        r"trailing\\",
        "",
        "日本",
        "100%",
        "^caret&amp|pipe>redir<in",
        "a;b",
        "--flag=value with spaces",
        "$(not-a-shell)",
        "`tick`",
        "*",
    ];
    let mut args = vec!["args"];
    args.extend(tricky);
    let out = run(&args).await;
    let expected: String = tricky.iter().map(|arg| format!("[{arg}]\n")).collect();
    assert_eq!(String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n"), expected);
}

#[tokio::test]
async fn a_program_path_with_spaces_and_unicode_runs() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("pasta com espaço e ção");
    std::fs::create_dir(&home).unwrap();
    let copy = home.join("peer copy.exe");
    std::fs::copy(peer_path(), &copy).unwrap();
    let mut spec = CommandSpec::new(&copy);
    spec.args = vec!["args".into(), "ok".into()];
    let out = run_capped(spec).await.unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "[ok]");
}

#[tokio::test]
async fn the_working_directory_is_honoured() {
    let dir = tempfile::tempdir().unwrap();
    let cwd = dir.path().join("diretório-ção");
    std::fs::create_dir(&cwd).unwrap();
    let mut spec = peer(&["cwd"]);
    spec.cwd = Some(cwd);
    let out = run_capped(spec).await.unwrap();
    let printed = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    assert!(printed.ends_with("diretório-ção"), "{printed}");
}

#[tokio::test]
async fn a_missing_binary_is_binary_not_found_without_leaking_its_path() {
    let dir = tempfile::tempdir().unwrap();
    let absolute = CommandSpec::new(dir.path().join("missing.exe"));
    let bare = CommandSpec::new("ai-sm-no-such-tool-0b1c2d3e");
    for spec in [absolute, bare] {
        let error = run_capped(spec).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::BinaryNotFound);
        assert!(!error.retryable);
        assert!(error.params.is_empty());
    }
}

#[tokio::test]
async fn other_start_failures_are_spawn_failed_and_never_panic() {
    let dir = tempfile::tempdir().unwrap();
    let text = dir.path().join("notes.txt");
    std::fs::write(&text, "not a program").unwrap();
    let missing_cwd = {
        let mut spec = peer(&["exit", "0"]);
        spec.cwd = Some(dir.path().join("no-such-dir"));
        spec
    };
    let not_programs = [
        CommandSpec::new(dir.path()),
        CommandSpec::new(&text),
        CommandSpec::new(OsString::from("a\0b")),
        missing_cwd,
    ];
    for spec in not_programs {
        let error = run_capped(spec).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::SpawnFailed);
        assert!(!error.retryable && error.params.is_empty());
    }
    let error = run_capped(CommandSpec::new("")).await.unwrap_err();
    assert!(matches!(error.code, ErrorCode::BinaryNotFound | ErrorCode::SpawnFailed));
}

#[tokio::test]
async fn parallel_runs_do_not_mix_their_streams() {
    let tasks: Vec<_> = (0..8usize)
        .map(|n| {
            let input = pattern(50_000 + n);
            let mut spec = peer(&["cat"]);
            spec.stdin = Some(input.clone());
            spec.max_stdout = 1 << 20;
            tokio::spawn(async move { (input, run_capped(spec).await.unwrap()) })
        })
        .collect();
    for task in tasks {
        let (input, out) = task.await.unwrap();
        assert!(out.stdout == input);
    }
}

#[test]
fn debug_output_shows_sizes_and_never_arguments_stdin_or_output() {
    let mut spec = CommandSpec::new(r"C:\tools\codex.cmd");
    spec.args = vec!["--api-key".into(), "sk-live-fabricated-1".into()];
    spec.stdin = Some(b"password=hunter2".to_vec());
    let text = format!("{spec:?} {spec:#?}");
    for hidden in ["sk-live", "api-key", "hunter2", r"C:\tools"] {
        assert!(!text.contains(hidden), "{hidden} leaked: {text}");
    }
    assert!(text.contains("codex.cmd"));

    let output = ProcOutput {
        status_code: Some(0),
        stdout: b"token=abc123xyz".to_vec(),
        stderr: b"secret-stderr".to_vec(),
        truncated: false,
    };
    let text = format!("{output:?} {output:#?}");
    assert!(!text.contains("abc123") && !text.contains("secret-stderr"), "{text}");
}
