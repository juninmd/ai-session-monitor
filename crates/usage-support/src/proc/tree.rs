//! Children that never open a console window and die together with their descendants.

use std::ffi::{OsStr, OsString};
use std::io;
use std::path::Path;
use std::time::Duration;

use tokio::process::{Child, Command};
use usage_model::{ErrorCode, HarnessError};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// A tree kill that takes longer is abandoned: the direct child is still ended.
#[cfg(windows)]
const KILL_TIMEOUT: Duration = Duration::from_secs(5);
/// How long to wait for an exit status after a kill before giving up on it.
const REAP_TIMEOUT: Duration = Duration::from_secs(2);

/// A command with stdio left to the caller, no console window and the direct child killed on drop.
pub(super) fn command(program: &OsStr, args: &[OsString], cwd: Option<&Path>) -> Command {
    let mut command = Command::new(program);
    command.args(args).kill_on_drop(true);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

/// A missing binary is its own code; any other failure to start is `SpawnFailed`.
pub(super) fn spawn_error(error: &io::Error) -> HarnessError {
    let code = match error.kind() {
        io::ErrorKind::NotFound => ErrorCode::BinaryNotFound,
        _ => ErrorCode::SpawnFailed,
    };
    HarnessError::new(code, false)
}

/// A missing working directory would otherwise be reported as a missing binary.
pub(super) fn check_cwd(cwd: Option<&Path>) -> Result<(), HarnessError> {
    match cwd {
        Some(dir) if !dir.is_dir() => Err(HarnessError::new(ErrorCode::SpawnFailed, false)),
        _ => Ok(()),
    }
}

/// Owns a spawned child. Dropping it while the child is unreaped ends the whole process tree.
pub(super) struct ChildGuard {
    pub(super) child: Child,
}

impl ChildGuard {
    pub(super) fn spawn(command: &mut Command) -> io::Result<Self> {
        command.spawn().map(|child| Self { child })
    }

    /// Ends the tree, then waits briefly for the exit status so nothing is left half-dead.
    pub(super) async fn kill(&mut self) {
        if let Some(pid) = self.child.id() {
            kill_tree(pid).await;
        }
        let _ = self.child.start_kill();
        let _ = tokio::time::timeout(REAP_TIMEOUT, self.child.wait()).await;
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        // The id is gone once the child was waited for, so a finished run costs nothing here.
        // While it is there the OS keeps the pid reserved, so the kill cannot hit a stranger.
        if let Some(pid) = self.child.id() {
            kill_tree_blocking(pid);
        }
    }
}

/// `taskkill` by absolute path: one found through PATH could be planted by whoever can write there.
#[cfg(windows)]
fn taskkill(pid: u32) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    use std::process::Stdio;

    let root = std::env::var_os("SystemRoot").unwrap_or_else(|| OsString::from(r"C:\Windows"));
    let exe = Path::new(&root).join("System32").join("taskkill.exe");
    let mut command = std::process::Command::new(exe);
    command
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

#[cfg(windows)]
async fn kill_tree(pid: u32) {
    let mut command = Command::from(taskkill(pid));
    let Ok(mut killer) = command.kill_on_drop(true).spawn() else { return };
    if tokio::time::timeout(KILL_TIMEOUT, killer.wait()).await.is_err() {
        let _ = killer.kill().await;
    }
}

#[cfg(windows)]
fn kill_tree_blocking(pid: u32) {
    let Ok(mut killer) = taskkill(pid).spawn() else { return };
    let give_up = std::time::Instant::now() + KILL_TIMEOUT;
    while matches!(killer.try_wait(), Ok(None)) {
        if std::time::Instant::now() >= give_up {
            let _ = killer.kill();
            let _ = killer.wait();
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

// Without a tree-kill tool the direct child is ended by `start_kill` and `kill_on_drop`.
#[cfg(not(windows))]
async fn kill_tree(_pid: u32) {}

#[cfg(not(windows))]
fn kill_tree_blocking(_pid: u32) {}
