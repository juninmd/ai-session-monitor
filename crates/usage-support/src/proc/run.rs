//! `run_capped`: one program, run to completion within a deadline and a memory bound.

use std::pin::pin;
use std::process::Stdio;
use std::time::Duration;

use tokio::time::{Instant, timeout, timeout_at};
use usage_model::{ErrorCode, HarnessError};

use super::capture::{Capture, feed};
use super::spec::{CommandSpec, MAX_CAPTURE_BYTES, MAX_TIMEOUT, ProcOutput};
use super::tree::{ChildGuard, check_cwd, command, spawn_error};

/// Descendants of a finished child may keep its pipes open; wait this long for them, then move on.
const PIPE_GRACE: Duration = Duration::from_millis(250);

/// Runs the program and returns what it printed. A missing binary is `BinaryNotFound`, any other
/// start failure `SpawnFailed`, and running past the deadline `Timeout` after the whole process
/// tree was killed. A non-zero exit is not an error: look at `status_code`.
///
/// Dropping the future also ends the process tree.
pub async fn run_capped(spec: CommandSpec) -> Result<ProcOutput, HarnessError> {
    let CommandSpec { program, args, cwd, stdin, timeout: limit, max_stdout, max_stderr } = spec;
    check_cwd(cwd.as_deref())?;
    let deadline = Instant::now() + limit.min(MAX_TIMEOUT);

    let mut launch = command(&program, &args, cwd.as_deref());
    launch
        .stdin(if stdin.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut guard = ChildGuard::spawn(&mut launch).map_err(|e| spawn_error(&e))?;
    let pipes = (guard.child.stdin.take(), guard.child.stdout.take(), guard.child.stderr.take());
    let (stdin_pipe, Some(stdout), Some(stderr)) = pipes else {
        guard.kill().await;
        return Err(HarnessError::new(ErrorCode::Internal, false));
    };

    let mut out = Capture::new(max_stdout.min(MAX_CAPTURE_BYTES));
    let mut err = Capture::new(max_stderr.min(MAX_CAPTURE_BYTES));
    let finished = timeout_at(deadline, async {
        let mut pipes = pin!(async {
            tokio::join!(out.fill(stdout), err.fill(stderr), feed(stdin_pipe, stdin));
        });
        tokio::select! {
            status = guard.child.wait() => {
                let _ = timeout(PIPE_GRACE, &mut pipes).await;
                status
            }
            _ = &mut pipes => guard.child.wait().await,
        }
    })
    .await;

    match finished {
        Ok(Ok(status)) => Ok(ProcOutput {
            status_code: status.code(),
            truncated: out.truncated || err.truncated,
            stdout: out.bytes,
            stderr: err.bytes,
        }),
        Ok(Err(_)) => {
            guard.kill().await;
            Err(HarnessError::new(ErrorCode::Internal, false))
        }
        Err(_elapsed) => {
            guard.kill().await;
            Err(HarnessError::new(ErrorCode::Timeout, true))
        }
    }
}
