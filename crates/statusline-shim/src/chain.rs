//! Runs the user's previous statusLine command, which the installer saved in the chain file as
//! `{"command": "<the old command>"}`, with the same stdin Claude Code gave us, and hands back its
//! stdout. Mirrors what Claude Code does with a status line command: the output counts only when
//! the command exits 0, the command is judged by its exit rather than by its pipe closing (a
//! background job it leaves holding stdout costs at most `SETTLE`), and a command that hangs is
//! killed (with the native processes it started).

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::capture::Payload;
use crate::collect::Collector;
use crate::shell::{self, Shell};

pub const CHAIN_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_CHAIN_FILE_BYTES: u64 = 64 * 1024;
const MAX_COMMAND_BYTES: usize = 32 * 1024;
/// A status line is a few lines of text; a command that prints more is not behaving like one.
const MAX_OUTPUT_BYTES: usize = 64 * 1024;
const POLL: Duration = Duration::from_millis(5);
const KILL_GRACE: Duration = Duration::from_secs(2);
/// After a clean exit, how long to wait for a pipe that a background job keeps open.
const SETTLE: Duration = Duration::from_millis(500);
/// Set for the child so a chain file that points back at the shim cannot recurse without end.
pub const DEPTH_VAR: &str = "AISM_SHIM_DEPTH";
const MAX_DEPTH: u32 = 3;

pub fn read_command(path: &Path) -> Option<String> {
    let mut bytes = Vec::new();
    File::open(path).ok()?.take(MAX_CHAIN_FILE_BYTES).read_to_end(&mut bytes).ok()?;
    let Value::Object(mut doc) = serde_json::from_slice(&bytes).ok()? else {
        return None;
    };
    let Some(Value::String(command)) = doc.remove("command") else {
        return None;
    };
    let command = command.trim();
    (!command.is_empty() && command.len() <= MAX_COMMAND_BYTES).then(|| command.to_owned())
}

pub fn depth_allows_chaining(current: Option<&str>) -> bool {
    current.and_then(|text| text.trim().parse::<u32>().ok()).unwrap_or(0) < MAX_DEPTH
}

pub struct Run<'a> {
    pub shell: &'a Shell,
    pub command: &'a str,
    pub respect_execution_policy: bool,
    pub depth: u32,
    pub timeout: Duration,
}

/// `Some(stdout)` only when the command ran to a clean exit within the budget.
pub fn run(spec: &Run<'_>, payload: Payload) -> Option<Vec<u8>> {
    let mut process = shell::command_for(spec.shell, spec.command, spec.respect_execution_policy);
    process.env(DEPTH_VAR, spec.depth.saturating_add(1).to_string());
    process.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
    let mut child = process.spawn().ok()?;
    let deadline = Instant::now() + spec.timeout;

    let stdin = child.stdin.take();
    thread::spawn(move || feed(stdin, &payload));
    let Some(stdout) = child.stdout.take() else {
        kill_tree(&mut child);
        return None;
    };
    let output = Collector::start(stdout, MAX_OUTPUT_BYTES);

    if !exited_cleanly(&mut child, deadline, &output) {
        return None;
    }
    // The command is done; a background job it left behind may still hold the pipe open.
    output.finish(SETTLE)
}

/// Dropping `stdin` at the end closes the pipe, which is the child's end-of-input.
fn feed(stdin: Option<ChildStdin>, payload: &Payload) {
    let Some(mut stdin) = stdin else { return };
    if stdin.write_all(&payload.head).is_err() || !payload.overflow {
        return;
    }
    // The payload was bigger than we keep: the rest still reaches the command, uninspected.
    let _ = io::copy(&mut io::stdin().lock(), &mut stdin);
}

/// Waits for the exit; a command still running at the deadline, or already past the output cap,
/// is killed.
fn exited_cleanly(child: &mut Child, deadline: Instant, output: &Collector) -> bool {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) if Instant::now() < deadline && !output.over_limit() => thread::sleep(POLL),
            _ => {
                kill_tree(child);
                return false;
            }
        }
    }
}

/// `Child::kill` ends only the shell; on Windows `taskkill /T` also ends the native processes it
/// started (`node`, `jq`, ...), so a hung command does not pile up at every refresh. MSYS tools
/// whose fork parent already exited (Git Bash's `sleep`, `cat`) are out of its reach.
fn kill_tree(child: &mut Child) {
    if cfg!(windows)
        && let Some(root) = std::env::var_os("SystemRoot")
    {
        let taskkill = PathBuf::from(root).join("System32").join("taskkill.exe");
        let mut kill = Command::new(taskkill);
        kill.args(["/PID", &child.id().to_string(), "/T", "/F"]);
        kill.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        shell::hide_window(&mut kill);
        if let Ok(mut running) = kill.spawn() {
            let until = Instant::now() + KILL_GRACE;
            while running.try_wait().is_ok_and(|status| status.is_none()) && Instant::now() < until
            {
                thread::sleep(POLL);
            }
            let _ = running.kill();
            let _ = running.wait();
        }
    }
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn write_chain(dir: &Path, text: &str) -> PathBuf {
        let path = dir.join("chain.json");
        fs::write(&path, text).unwrap();
        path
    }

    #[test]
    fn reads_the_command_from_the_chain_file() {
        let dir = tempfile::tempdir().unwrap();
        let path =
            write_chain(dir.path(), r#"{"command":"  ~/.claude/status.sh --fancy  ","x":1}"#);
        assert_eq!(read_command(&path).as_deref(), Some("~/.claude/status.sh --fancy"));
    }

    #[test]
    fn unicode_commands_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_chain(dir.path(), r#"{"command":"echo 'ção ✓'"}"#);
        assert_eq!(read_command(&path).as_deref(), Some("echo 'ção ✓'"));
    }

    #[test]
    fn rejects_chain_files_that_are_not_usable() {
        let dir = tempfile::tempdir().unwrap();
        let huge = format!(r#"{{"command":"{}"}}"#, "x".repeat(MAX_COMMAND_BYTES + 1));
        for text in [
            "",
            "not json",
            "[]",
            r#"{}"#,
            r#"{"command":""}"#,
            r#"{"command":"   "}"#,
            r#"{"command":42}"#,
            r#"{"command":null}"#,
            huge.as_str(),
        ] {
            let path = write_chain(dir.path(), text);
            assert_eq!(read_command(&path), None, "{}", &text[..text.len().min(40)]);
        }
        assert_eq!(read_command(&dir.path().join("missing.json")), None);
    }

    #[test]
    fn a_command_of_exactly_the_size_limit_is_accepted() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_chain(
            dir.path(),
            &format!(r#"{{"command":"{}"}}"#, "x".repeat(MAX_COMMAND_BYTES)),
        );
        assert_eq!(read_command(&path).map(|command| command.len()), Some(MAX_COMMAND_BYTES));
    }

    #[test]
    fn recursion_is_cut_off_at_the_depth_limit() {
        assert!(depth_allows_chaining(None));
        assert!(depth_allows_chaining(Some("0")));
        assert!(depth_allows_chaining(Some("2")));
        assert!(!depth_allows_chaining(Some("3")));
        assert!(!depth_allows_chaining(Some("99")));
        assert!(depth_allows_chaining(Some("garbage")), "unparseable means no previous depth");
    }
}
