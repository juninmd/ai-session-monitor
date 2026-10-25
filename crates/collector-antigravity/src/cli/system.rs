//! Runs the installed CLI: fixed path under the user's profile, argv only, bounded.

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use serde_json::Value;
use usage_model::{Ctx, ErrorCode, HarnessError};
use usage_support::{CommandSpec, neutral_dir, run_capped};

use super::{Cli, CliError};

/// `%LOCALAPPDATA%\agy\bin`, relative to the profile directory the context carries.
const INSTALL_DIR: [&str; 5] = ["AppData", "Local", "agy", "bin", "agy.exe"];
/// The CLI starts its embedded server first; a few seconds is normal.
const TIMEOUT: Duration = Duration::from_secs(15);
/// The real answer is a few KiB; anything near this is not a quota report.
const MAX_STDOUT: usize = 256 << 10;
const MAX_STDERR: usize = 16 << 10;
const ARGS: [&str; 4] = ["-p", "/usage", "--output-format", "json"];

#[derive(Debug)]
pub(crate) struct SystemCli {
    path: Vec<&'static str>,
    timeout: Duration,
}

impl SystemCli {
    pub(crate) fn new() -> Self {
        Self { path: INSTALL_DIR.to_vec(), timeout: TIMEOUT }
    }

    /// Tests run their own throwaway program, never a real install.
    #[cfg(test)]
    pub(crate) fn with(file: &'static str, timeout: Duration) -> Self {
        Self { path: vec!["AppData", "Local", "agy", "bin", file], timeout }
    }
}

impl Cli for SystemCli {
    async fn usage(&self, ctx: &Ctx) -> Result<Value, CliError> {
        let program: PathBuf =
            self.path.iter().fold(ctx.home_dir.clone(), |dir, part| dir.join(part));
        if !program.is_file() {
            return Err(CliError::Absent);
        }
        let mut spec = CommandSpec::new(program);
        spec.args = ARGS.map(OsString::from).to_vec();
        // A neutral directory: the CLI must not pick up a project's settings or write into one.
        let cwd = neutral_dir(&ctx.data_dir)
            .map_err(|_| CliError::Failed(HarnessError::new(ErrorCode::SpawnFailed, false)))?;
        spec.cwd = Some(cwd);
        spec.timeout = self.timeout;
        spec.max_stdout = MAX_STDOUT;
        spec.max_stderr = MAX_STDERR;
        let output = run_capped(spec).await.map_err(|error| match error.code {
            ErrorCode::BinaryNotFound => CliError::Absent,
            _ => CliError::Failed(error),
        })?;
        let failed = |code| CliError::Failed(HarnessError::new(code, true));
        if output.truncated {
            return Err(failed(ErrorCode::ResponseTooLarge));
        }
        if output.status_code != Some(0) {
            return Err(failed(ErrorCode::BadResponse));
        }
        serde_json::from_slice(&output.stdout).map_err(|_| failed(ErrorCode::BadResponse))
    }
}
