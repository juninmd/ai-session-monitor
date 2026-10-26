//! Runs the installed CLI: fixed paths under the user's profile, argv only, no shell, no PATH.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;
use usage_model::{Ctx, ErrorCode, HarnessError};
use usage_support::{CommandSpec, neutral_dir, run_capped};

use super::{Cli, CliError};

/// Where Claude Code puts its launcher on Windows, relative to the profile directory: the native
/// installer (documented), the default npm global prefix, and the WinGet portable link.
const LOCATIONS: [&[&str]; 3] = [
    &[".local", "bin", "claude.exe"],
    &["AppData", "Roaming", "npm", "claude.cmd"],
    &["AppData", "Local", "Microsoft", "WinGet", "Links", "claude.exe"],
];
/// A run is about five seconds; a hung one is killed with its descendants.
const TIMEOUT: Duration = Duration::from_secs(20);
/// The real answer is about 3 KiB; anything near this is not a limits report.
const MAX_STDOUT: usize = 256 << 10;
const MAX_STDERR: usize = 16 << 10;
/// Cost bound in case a future version treats `/usage` as a prompt; the local command costs 0.
/// `--no-session-persistence` keeps the polls out of the user's session history. Loading only the
/// `project` settings (the neutral working directory has none) keeps the user's global hooks and
/// plugins from firing on every poll, where a hook that opens a terminal would steal the focus.
const ARGS: [&str; 8] = [
    "-p",
    "/usage",
    "--output-format",
    "json",
    "--max-budget-usd",
    "0.02",
    "--setting-sources",
    "project",
];
const NO_SESSION_FLAG: &str = "--no-session-persistence";

#[derive(Debug)]
pub(crate) struct SystemCli {
    locations: Vec<Vec<&'static str>>,
    timeout: Duration,
}

impl SystemCli {
    pub(crate) fn new() -> Self {
        Self { locations: LOCATIONS.iter().map(|l| l.to_vec()).collect(), timeout: TIMEOUT }
    }

    /// Tests run their own throwaway program, never a real install.
    #[cfg(test)]
    pub(crate) fn with(location: &[&'static str], timeout: Duration) -> Self {
        Self { locations: vec![location.to_vec()], timeout }
    }

    pub(crate) fn program(&self, home: &Path) -> Option<PathBuf> {
        self.locations
            .iter()
            .map(|parts| parts.iter().fold(home.to_path_buf(), |dir, part| dir.join(part)))
            .find(|path| path.is_file())
    }
}

impl Cli for SystemCli {
    async fn usage(&self, ctx: &Ctx, config_dir: Option<&Path>) -> Result<Value, CliError> {
        let Some(program) = self.program(&ctx.home_dir) else { return Err(CliError::Absent) };
        let mut spec = CommandSpec::new(program);
        spec.args = ARGS.iter().chain([&NO_SESSION_FLAG]).map(OsString::from).collect();
        if let Some(dir) = config_dir {
            spec.env = vec![("CLAUDE_CONFIG_DIR".into(), dir.as_os_str().to_owned())];
        }
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
