//! `LineSession`: a child spoken to one line at a time over stdin/stdout (JSON-RPC over stdio).

use std::fmt;
use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{ChildStdin, ChildStdout};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use usage_model::{ErrorCode, HarnessError};

use super::lines::{LineBuffer, LineError};
use super::spec::{CommandSpec, MAX_CAPTURE_BYTES, MAX_TIMEOUT};
use super::tree::{ChildGuard, check_cwd, command, spawn_error};

/// Longest line accepted unless [`LineSession::with_max_line`] says otherwise.
pub const DEFAULT_MAX_LINE: usize = 1 << 20;
/// A child that stops reading its stdin must not hold the caller forever.
const SEND_TIMEOUT: Duration = Duration::from_secs(10);
const CHUNK_BYTES: usize = 8 * 1024;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Open,
    /// The child closed its stdout: only what is already buffered remains.
    Eof,
    /// A line broke the length cap, so the stream can no longer be framed.
    Violated,
}

fn protocol_error() -> HarnessError {
    HarnessError::new(ErrorCode::ProtocolError, false)
}

fn not_running() -> HarnessError {
    HarnessError::new(ErrorCode::NotRunning, true)
}

/// A running child with piped stdin and stdout. Lines go in with [`send_line`](Self::send_line)
/// and come out with [`read_line`](Self::read_line); stderr is drained and discarded.
///
/// Dropping the session ends the child and its descendants.
pub struct LineSession {
    guard: ChildGuard,
    stdin: Option<ChildStdin>,
    stdout: ChildStdout,
    lines: LineBuffer,
    max_line: usize,
    phase: Phase,
    drain: Option<JoinHandle<()>>,
}

impl fmt::Debug for LineSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LineSession")
            .field("pid", &self.guard.child.id())
            .field("phase", &self.phase)
            .field("max_line", &self.max_line)
            .finish_non_exhaustive()
    }
}

impl LineSession {
    /// Starts the child. Uses `program`, `args` and `cwd` of the spec; `stdin`, `timeout` and the
    /// output caps belong to [`run_capped`](super::run_capped) and are ignored. Must be called
    /// inside a Tokio runtime.
    pub fn spawn(spec: CommandSpec) -> Result<Self, HarnessError> {
        check_cwd(spec.cwd.as_deref())?;
        let mut launch = command(&spec.program, &spec.args, spec.cwd.as_deref(), &spec.env);
        launch.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut guard = ChildGuard::spawn(&mut launch).map_err(|e| spawn_error(&e))?;
        let pipes =
            (guard.child.stdin.take(), guard.child.stdout.take(), guard.child.stderr.take());
        let (Some(stdin), Some(stdout), Some(mut stderr)) = pipes else {
            return Err(HarnessError::new(ErrorCode::Internal, false));
        };
        // A child that fills an unread stderr pipe would stall; nothing from it is kept.
        let drain = tokio::spawn(async move {
            let mut sink = [0u8; CHUNK_BYTES];
            while matches!(stderr.read(&mut sink).await, Ok(read) if read > 0) {}
        });
        Ok(Self {
            guard,
            stdin: Some(stdin),
            stdout,
            lines: LineBuffer::new(DEFAULT_MAX_LINE),
            max_line: DEFAULT_MAX_LINE,
            phase: Phase::Open,
            drain: Some(drain),
        })
    }

    /// Sets the longest line, in either direction (at most 16 MiB). Call it before the first read.
    #[must_use]
    pub fn with_max_line(mut self, max_line: usize) -> Self {
        self.max_line = max_line.min(MAX_CAPTURE_BYTES);
        self.lines = LineBuffer::new(self.max_line);
        self
    }

    /// Sends `line` plus a newline. A line holding `\n` or `\r` would change the framing and is
    /// refused (`InvalidInput`), as is one over the line cap. A child that has gone away is
    /// `NotRunning`; one that stops reading is `Timeout`, and the session stops writing after
    /// that because the stream may hold half a line.
    pub async fn send_line(&mut self, line: &str) -> Result<(), HarnessError> {
        if line.contains(['\n', '\r']) || line.len() > self.max_line {
            return Err(HarnessError::new(ErrorCode::InvalidInput, false));
        }
        let Some(stdin) = self.stdin.as_mut() else { return Err(not_running()) };
        let mut framed = Vec::with_capacity(line.len() + 1);
        framed.extend_from_slice(line.as_bytes());
        framed.push(b'\n');
        let written = timeout(SEND_TIMEOUT, async {
            stdin.write_all(&framed).await?;
            stdin.flush().await
        })
        .await;
        match written {
            Ok(Ok(())) => Ok(()),
            Ok(Err(_)) => {
                self.stdin = None;
                Err(not_running())
            }
            Err(_elapsed) => {
                self.stdin = None;
                Err(HarnessError::new(ErrorCode::Timeout, true))
            }
        }
    }

    /// The next line, without its terminator. After `wait` without a full line it returns
    /// `Timeout`, and the bytes that did arrive are kept for the next call. A line over the cap is
    /// `ProtocolError` and ends the session's reading; invalid UTF-8 costs one `ProtocolError` and
    /// the following lines are still read. When the child closed stdout, any last unterminated
    /// line comes out first and then every call is `NotRunning`.
    pub async fn read_line(&mut self, wait: Duration) -> Result<String, HarnessError> {
        match timeout(wait.min(MAX_TIMEOUT), self.next_line()).await {
            Ok(result) => result,
            Err(_elapsed) => Err(HarnessError::new(ErrorCode::Timeout, true)),
        }
    }

    async fn next_line(&mut self) -> Result<String, HarnessError> {
        loop {
            if self.phase == Phase::Violated {
                return Err(protocol_error());
            }
            match self.lines.next_line() {
                Ok(Some(line)) => return Ok(line),
                Err(error) => return Err(self.framing_error(error)),
                Ok(None) => {}
            }
            if self.phase == Phase::Eof {
                return match self.lines.finish() {
                    Ok(Some(line)) => Ok(line),
                    Ok(None) => Err(not_running()),
                    Err(error) => Err(self.framing_error(error)),
                };
            }
            // Cancel-safe: a read dropped by the timeout has consumed nothing.
            let mut chunk = [0u8; CHUNK_BYTES];
            match self.stdout.read(&mut chunk).await {
                Ok(read) if read > 0 => self.lines.push(&chunk[..read]),
                _ => self.phase = Phase::Eof,
            }
        }
    }

    fn framing_error(&mut self, error: LineError) -> HarnessError {
        if error == LineError::TooLong {
            self.phase = Phase::Violated;
        }
        protocol_error()
    }

    /// Ends the child and its descendants. Safe to call twice; afterwards the session reports
    /// `NotRunning`.
    pub async fn kill(&mut self) {
        // The tree goes first: a child that exits at stdin EOF would be gone before `taskkill /T`
        // could find its descendants.
        self.guard.kill().await;
        self.stdin = None;
        self.phase = Phase::Eof;
        self.lines = LineBuffer::new(self.max_line);
        if let Some(drain) = self.drain.take() {
            drain.abort();
        }
    }
}

impl Drop for LineSession {
    fn drop(&mut self) {
        if let Some(drain) = self.drain.take() {
            drain.abort();
        }
    }
}
