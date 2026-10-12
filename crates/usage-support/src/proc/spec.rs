//! What to run, and what came back. Debug output shows sizes only: arguments and output can
//! carry credentials.

use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Longest wall-clock limit honoured; the cap keeps timer arithmetic far from overflow.
pub(super) const MAX_TIMEOUT: Duration = Duration::from_secs(600);
/// Largest capture buffer honoured per stream, whatever the caller asks for.
pub(super) const MAX_CAPTURE_BYTES: usize = 16 << 20;

/// One program run to completion with bounded time and memory. The program and its arguments
/// travel as separate values; nothing here is ever parsed by a shell.
#[derive(Clone)]
pub struct CommandSpec {
    /// Prefer an absolute path from [`resolve_binary`](super::resolve_binary); a bare name is
    /// looked up by the OS rules (application directory, system directories, PATH).
    pub program: OsString,
    pub args: Vec<OsString>,
    pub cwd: Option<PathBuf>,
    /// Written to the child's stdin, which is then closed. `None` gives the child an empty stdin.
    pub stdin: Option<Vec<u8>>,
    /// Hard wall-clock limit (at most ten minutes), after which the process tree is killed.
    pub timeout: Duration,
    /// Bytes of stdout kept (at most 16 MiB); the rest is read and dropped.
    pub max_stdout: usize,
    /// Bytes of stderr kept (at most 16 MiB); the rest is read and dropped.
    pub max_stderr: usize,
}

impl CommandSpec {
    /// Defaults: no arguments, 30 s, 1 MiB of stdout, 64 KiB of stderr.
    pub fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            cwd: None,
            stdin: None,
            timeout: Duration::from_secs(30),
            max_stdout: 1 << 20,
            max_stderr: 64 << 10,
        }
    }
}

impl fmt::Debug for CommandSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = Path::new(&self.program).file_name().unwrap_or_default();
        f.debug_struct("CommandSpec")
            .field("program", &name)
            .field("args", &self.args.len())
            .field("cwd", &self.cwd.is_some())
            .field("stdin_bytes", &self.stdin.as_ref().map(Vec::len))
            .field("timeout", &self.timeout)
            .field("max_stdout", &self.max_stdout)
            .field("max_stderr", &self.max_stderr)
            .finish()
    }
}

/// How a finished process ended. A non-zero exit is data here, not an error.
#[derive(Clone, PartialEq, Eq)]
pub struct ProcOutput {
    /// `None` only when a signal ended the process (never on Windows).
    pub status_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    /// Some output beyond the caps was read and dropped.
    pub truncated: bool,
}

impl fmt::Debug for ProcOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProcOutput")
            .field("status_code", &self.status_code)
            .field("stdout_bytes", &self.stdout.len())
            .field("stderr_bytes", &self.stderr.len())
            .field("truncated", &self.truncated)
            .finish()
    }
}
