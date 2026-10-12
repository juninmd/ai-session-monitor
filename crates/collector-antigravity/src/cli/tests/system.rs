//! The real runner against throwaway `.cmd` programs in a temp profile: argv, fixed location, caps,
//! timeout and silence about what the program printed. Windows only, like the CLI.
#![cfg(windows)]

use std::path::Path;
use std::time::{Duration, Instant};

use super::*;

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cli_usage.json");

/// A temp profile whose `AppData\Local\agy\bin` holds `name` with `body`.
fn profile(name: &str, body: &str) -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    let bin = home.path().join("AppData").join("Local").join("agy").join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::write(bin.join(name), body).unwrap();
    home
}

fn ctx_in(home: &Path) -> Ctx {
    Ctx::new(NOW, home, home)
}

/// Prints the fixture only for exactly the four arguments the collector must send.
fn picky_script() -> String {
    // `type` rejects a path that mixes both separators.
    let fixture = FIXTURE.replace('/', "\\");
    format!(
        "@echo off\r\nif not \"%~1\"==\"-p\" exit /b 3\r\nif not \"%~2\"==\"/usage\" exit /b 3\r\n\
         if not \"%~3\"==\"--output-format\" exit /b 3\r\nif not \"%~4\"==\"json\" exit /b 3\r\n\
         if not \"%~5\"==\"\" exit /b 3\r\ntype \"{fixture}\"\r\n"
    )
}

#[tokio::test]
async fn the_installed_program_is_run_with_exactly_the_usage_arguments() {
    let home = profile("agy.cmd", &picky_script());
    let cli = SystemCli::with("agy.cmd", Duration::from_secs(10));
    let output = cli.usage(&ctx_in(home.path())).await.expect("runs");
    let read = parse_usage(&output, NOW).expect("parses");
    assert_eq!(read.windows.len(), 4);
}

#[tokio::test]
async fn no_install_means_absent_and_the_path_is_never_searched() {
    let home = tempfile::tempdir().unwrap();
    let cli = SystemCli::with("agy.cmd", Duration::from_secs(10));
    assert!(matches!(cli.usage(&ctx_in(home.path())).await, Err(CliError::Absent)));
}

#[tokio::test]
async fn a_program_that_hangs_is_killed_at_the_deadline() {
    let home = profile("agy.cmd", "@echo off\r\nping -n 60 127.0.0.1 >nul\r\n");
    let cli = SystemCli::with("agy.cmd", Duration::from_millis(800));
    let started = Instant::now();
    let result = cli.usage(&ctx_in(home.path())).await;
    assert!(matches!(result, Err(CliError::Failed(e)) if e.code == ErrorCode::Timeout));
    assert!(started.elapsed() < Duration::from_secs(10));
}

#[tokio::test]
async fn garbage_and_failures_say_nothing_about_what_was_printed() {
    for body in [
        "@echo off\r\necho SECRET-GARBAGE ya29.token\r\n",
        "@echo off\r\necho SECRET-GARBAGE 1>&2\r\nexit /b 1\r\n",
        "@echo off\r\n",
    ] {
        let home = profile("agy.cmd", body);
        let cli = SystemCli::with("agy.cmd", Duration::from_secs(10));
        let ctx = ctx_in(home.path());
        let view = collect_with(&on(), &ctx, &closed_ide(), &cli, &Config::default()).await;
        assert_eq!(error_code(&view), Some(ErrorCode::BadResponse), "{body}");
        let shown = serde_json::to_string(&view).unwrap();
        assert!(!shown.contains("SECRET") && !shown.contains("ya29"), "{shown}");
    }
}

#[tokio::test]
async fn valid_output_with_a_failing_exit_code_is_an_error_not_a_view() {
    let fixture = FIXTURE.replace('/', "\\");
    let body = format!(
        "@echo off
type \"{fixture}\"
exit /b 1
"
    );
    let home = profile("agy.cmd", &body);
    let cli = SystemCli::with("agy.cmd", Duration::from_secs(10));
    let view =
        collect_with(&on(), &ctx_in(home.path()), &closed_ide(), &cli, &Config::default()).await;
    assert_eq!(error_code(&view), Some(ErrorCode::BadResponse));
}

#[tokio::test]
async fn output_past_the_cap_is_refused_not_parsed() {
    let big = "x".repeat(300);
    let body = format!("@echo off\r\nfor /L %%i in (1,1,1200) do @echo {big}\r\n");
    let home = profile("agy.cmd", &body);
    let cli = SystemCli::with("agy.cmd", Duration::from_secs(20));
    let result = cli.usage(&ctx_in(home.path())).await;
    assert!(matches!(result, Err(CliError::Failed(e)) if e.code == ErrorCode::ResponseTooLarge));
}
