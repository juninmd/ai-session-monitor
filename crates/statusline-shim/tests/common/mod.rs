//! Drives the built `statusline-shim` binary the way Claude Code does: JSON on stdin, a temp
//! directory standing in for the app data dir, everything fabricated.

#![allow(dead_code, reason = "each test file uses a different subset of the helpers")]
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "tests assert by panicking")]

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

const SHIM: &str = env!("CARGO_BIN_EXE_aism-statusline-shim");
const PEER: &str = env!("CARGO_BIN_EXE_shim_chain_peer");
const WATCHDOG: Duration = Duration::from_secs(60);

pub struct Outcome {
    pub code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub elapsed: Duration,
}

impl Outcome {
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }
}

pub struct Harness {
    pub dir: tempfile::TempDir,
}

impl Harness {
    pub fn new() -> Self {
        Self { dir: tempfile::tempdir().expect("temp dir") }
    }

    /// Spaces and non-ASCII on purpose: the real path is `...\Users\<name>\AppData\Local\...`.
    pub fn out(&self) -> PathBuf {
        self.dir.path().join("dados do usuário ção").join("claude").join("rate_limits.json")
    }

    pub fn read_out(&self) -> Option<Value> {
        let bytes = std::fs::read(self.out()).ok()?;
        Some(serde_json::from_slice(&bytes).expect("the recorded file is valid JSON"))
    }

    pub fn write_chain(&self, command: &str) -> PathBuf {
        let path = self.dir.path().join("chain.json");
        std::fs::write(&path, json!({ "command": command }).to_string()).expect("write chain");
        path
    }

    /// Copies the peer next to the shim's working directory so chain commands can name it
    /// relatively (`./shim_chain_peer`), which needs no quoting in any shell.
    pub fn install_peer(&self) -> String {
        let name = format!("shim_chain_peer{}", std::env::consts::EXE_SUFFIX);
        std::fs::copy(PEER, self.dir.path().join(&name)).expect("copy peer");
        format!("./{name}")
    }

    pub fn command(&self) -> Command {
        let mut command = Command::new(SHIM);
        command.current_dir(self.dir.path()).arg("--out").arg(self.out());
        command
    }

    pub fn run(&self, stdin: &[u8]) -> Outcome {
        run(self.command(), stdin)
    }

    pub fn run_chained(&self, chain_command: &str, stdin: &[u8]) -> Outcome {
        self.run_chained_then(chain_command, stdin, || {})
    }

    pub fn run_chained_then(
        &self,
        chain_command: &str,
        stdin: &[u8],
        at_exit: impl FnOnce(),
    ) -> Outcome {
        let chain = self.write_chain(chain_command);
        let mut command = self.command();
        command.arg("--chain").arg(chain);
        run_then(command, stdin, at_exit)
    }
}

pub fn payload_with_limits() -> Value {
    json!({
        "cwd": "/home/ana/projeto-secreto",
        "transcript_path": "/home/ana/.claude/projects/x/transcript-marker.jsonl",
        "session_id": "session-marker-123",
        "model": {"id": "model-marker", "display_name": "Marker"},
        "cost": {"total_cost_usd": 12.34},
        "rate_limits": {
            "five_hour": {"used_percentage": 23.5, "resets_at": 1_800_000_000},
            "seven_day": {"used_percentage": 41.2, "resets_at": 1_800_400_000}
        }
    })
}

pub fn run(command: Command, stdin: &[u8]) -> Outcome {
    run_then(command, stdin, || {})
}

/// Spawns, feeds stdin from a thread (large payloads would otherwise deadlock on the pipe) and
/// kills the process if it ever outlives the watchdog. `at_exit` runs the moment the process has
/// ended, before its pipes are drained: a background process of the chained command can hold them
/// open (the shim's own stdio is inheritable on Windows), and a test may need to release it first.
pub fn run_then(mut command: Command, stdin: &[u8], at_exit: impl FnOnce()) -> Outcome {
    command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let started = Instant::now();
    let mut child = command.spawn().expect("spawn the shim");
    let mut input = child.stdin.take().expect("stdin");
    let data = stdin.to_vec();
    let writer = std::thread::spawn(move || {
        let _ = input.write_all(&data);
    });
    let mut stdout = child.stdout.take().expect("stdout");
    let mut stderr = child.stderr.take().expect("stderr");
    let out = std::thread::spawn(move || read_all(&mut stdout));
    let err = std::thread::spawn(move || read_all(&mut stderr));
    let status = loop {
        if let Some(status) = child.try_wait().expect("poll") {
            break status;
        }
        if started.elapsed() > WATCHDOG {
            let _ = child.kill();
            panic!("the shim did not finish within {WATCHDOG:?}");
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    at_exit();
    let _ = writer.join();
    Outcome {
        code: status.code(),
        stdout: out.join().expect("stdout thread"),
        stderr: err.join().expect("stderr thread"),
        elapsed: started.elapsed(),
    }
}

fn read_all(source: &mut impl Read) -> Vec<u8> {
    let mut buf = Vec::new();
    let _ = source.read_to_end(&mut buf);
    buf
}
