//! The real runner: fixed locations, argv, caps, timeout. The `.cmd` programs are throwaway
//! stand-ins in a temp profile; the real `claude` is never started.

use std::path::Path;
use std::time::Duration;

use usage_model::ErrorCode;

use super::*;
use crate::cli::system::SystemCli;
use crate::testenv::NOW;

fn touch(home: &Path, parts: &[&str]) {
    let file = parts.iter().fold(home.to_path_buf(), |dir, part| dir.join(part));
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, "stand-in").unwrap();
}

#[test]
fn only_the_three_known_locations_under_home_are_looked_at_in_order() {
    let home = tempfile::tempdir().unwrap();
    let cli = SystemCli::new();
    assert_eq!(cli.program(home.path()), None);

    // A launcher anywhere else (PATH-like directories, the profile root) is not a candidate.
    touch(home.path(), &["claude.exe"]);
    touch(home.path(), &["bin", "claude.exe"]);
    touch(home.path(), &["AppData", "Local", "claude.exe"]);
    assert_eq!(cli.program(home.path()), None);

    let winget: &[&str] = &["AppData", "Local", "Microsoft", "WinGet", "Links", "claude.exe"];
    let npm: &[&str] = &["AppData", "Roaming", "npm", "claude.cmd"];
    let native: &[&str] = &[".local", "bin", "claude.exe"];
    for (parts, next) in [(winget, "winget"), (npm, "npm"), (native, "native")] {
        touch(home.path(), parts);
        let found = cli.program(home.path()).unwrap();
        assert_eq!(found, parts.iter().fold(home.path().to_path_buf(), |d, p| d.join(p)), "{next}");
    }
}

#[tokio::test]
async fn no_install_means_absent() {
    let home = tempfile::tempdir().unwrap();
    let ctx = Ctx::new(NOW, home.path(), home.path());
    let result = SystemCli::new().usage(&ctx, None).await;
    assert!(matches!(result, Err(CliError::Absent)));
}

#[cfg(windows)]
mod windows {
    use std::time::Instant;

    use super::*;

    const LOCATION: [&str; 3] = [".local", "bin", "claude.cmd"];

    fn profile(body: &str) -> tempfile::TempDir {
        let home = tempfile::tempdir().unwrap();
        touch(home.path(), &LOCATION);
        std::fs::write(home.path().join(".local").join("bin").join("claude.cmd"), body).unwrap();
        home
    }

    fn cli(timeout: Duration) -> SystemCli {
        SystemCli::with(&LOCATION, timeout)
    }

    async fn run(home: &Path, timeout: Duration) -> Result<serde_json::Value, CliError> {
        let ctx = Ctx::new(NOW, home, home);
        cli(timeout).usage(&ctx, None).await
    }

    fn code(result: Result<serde_json::Value, CliError>) -> ErrorCode {
        match result {
            Err(CliError::Failed(error)) => error.code,
            other => panic!("expected a failure, got {other:?}"),
        }
    }

    /// Prints a fabricated answer only for exactly the argv and environment the collector owes.
    fn picky_script(answer: &Path, config_dir: &str) -> String {
        let answer = answer.display();
        format!(
            "@echo off\r\nif not \"%~1\"==\"-p\" exit /b 3\r\nif not \"%~2\"==\"/usage\" exit /b 3\r\n\
             if not \"%~3\"==\"--output-format\" exit /b 3\r\nif not \"%~4\"==\"json\" exit /b 3\r\n\
             if not \"%~5\"==\"--max-budget-usd\" exit /b 3\r\nif not \"%~6\"==\"0.02\" exit /b 3\r\n\
             if not \"%~7\"==\"--no-session-persistence\" exit /b 3\r\nif not \"%~8\"==\"\" exit /b 3\r\n\
             if not \"%CLAUDE_CONFIG_DIR%\"==\"{config_dir}\" exit /b 4\r\ntype \"{answer}\"\r\n"
        )
    }

    #[tokio::test]
    async fn the_installed_program_gets_exactly_the_usage_arguments_and_the_config_dir() {
        let answers = tempfile::tempdir().unwrap();
        let answer = answers.path().join("answer.json");
        std::fs::write(&answer, envelope(REPORT).to_string()).unwrap();
        let config = answers.path().join("outro perfil");
        let home = profile(&picky_script(&answer, &config.display().to_string()));
        let ctx = Ctx::new(NOW, home.path(), home.path());

        let body = cli(Duration::from_secs(10)).usage(&ctx, Some(&config)).await.expect("runs");
        let rows = crate::cli::parse::envelope(&body).expect("parses");
        assert_eq!(rows.len(), 3);

        let wrong = cli(Duration::from_secs(10)).usage(&ctx, None).await;
        assert_eq!(code(wrong), ErrorCode::BadResponse, "no setting, no CLAUDE_CONFIG_DIR");
    }

    #[tokio::test]
    async fn a_program_that_hangs_is_killed_at_the_deadline() {
        let home = profile("@echo off\r\nping -n 60 127.0.0.1 >nul\r\n");
        let started = Instant::now();
        let result = run(home.path(), Duration::from_millis(800)).await;
        assert_eq!(code(result), ErrorCode::Timeout);
        assert!(started.elapsed() < Duration::from_secs(10));
    }

    #[tokio::test]
    async fn failures_and_garbage_say_nothing_about_what_was_printed() {
        for (body, expected) in [
            ("@echo off\r\necho SECRET-GARBAGE ya29.token\r\n", ErrorCode::BadResponse),
            ("@echo off\r\necho SECRET-GARBAGE 1>&2\r\nexit /b 1\r\n", ErrorCode::BadResponse),
            ("@echo off\r\n", ErrorCode::BadResponse),
        ] {
            let home = profile(body);
            let error = match run(home.path(), Duration::from_secs(10)).await {
                Err(CliError::Failed(error)) => error,
                other => panic!("expected a failure, got {other:?}"),
            };
            assert_eq!(error.code, expected, "{body}");
            let shown = serde_json::to_string(&error).unwrap();
            assert!(!shown.contains("SECRET") && !shown.contains("ya29"), "{shown}");
        }
    }

    #[tokio::test]
    async fn a_valid_answer_with_a_failing_exit_code_is_an_error() {
        let answers = tempfile::tempdir().unwrap();
        let answer = answers.path().join("answer.json");
        std::fs::write(&answer, envelope(REPORT).to_string()).unwrap();
        let home = profile(&format!("@echo off\r\ntype \"{}\"\r\nexit /b 1\r\n", answer.display()));
        assert_eq!(code(run(home.path(), Duration::from_secs(10)).await), ErrorCode::BadResponse);
    }

    #[tokio::test]
    async fn output_past_the_cap_is_refused_not_parsed() {
        let big = "x".repeat(300);
        let home = profile(&format!("@echo off\r\nfor /L %%i in (1,1,1200) do @echo {big}\r\n"));
        let result = run(home.path(), Duration::from_secs(20)).await;
        assert_eq!(code(result), ErrorCode::ResponseTooLarge);
    }

    #[tokio::test]
    async fn the_whole_poll_through_the_real_runner() {
        let env = installed_env();
        let answer = env.root().join("answer.json");
        std::fs::write(&answer, envelope(REPORT).to_string()).unwrap();
        let program = env.home.join(".local").join("bin").join("claude.cmd");
        std::fs::create_dir_all(program.parent().unwrap()).unwrap();
        std::fs::write(&program, format!("@echo off\r\ntype \"{}\"\r\n", answer.display()))
            .unwrap();

        let runner = cli(Duration::from_secs(10));
        let view = poll(&env, &runner, NOW).await;
        assert_eq!(
            ids(&view),
            ["five_hour", "weekly", "fable:weekly", "opus:weekly", "sonnet:weekly"]
        );
    }
}
