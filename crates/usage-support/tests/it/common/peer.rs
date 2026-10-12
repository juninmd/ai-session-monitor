//! The `test_peer` helper process, and Windows-only process inspection for the tree-kill tests.

use std::ffi::OsString;
use std::time::Duration;

use usage_support::CommandSpec;

pub fn peer_path() -> &'static str {
    env!("CARGO_BIN_EXE_test_peer")
}

/// A spec that runs the peer with `args` and a generous deadline, so a hang fails instead of
/// freezing the suite.
pub fn peer(args: &[&str]) -> CommandSpec {
    let mut spec = CommandSpec::new(peer_path());
    spec.args = args.iter().map(OsString::from).collect();
    spec.timeout = Duration::from_secs(20);
    spec
}

#[cfg(windows)]
pub async fn is_alive(pid: u32) -> bool {
    let root = std::env::var_os("SystemRoot").unwrap_or_default();
    let tasklist = std::path::Path::new(&root).join("System32").join("tasklist.exe");
    let output = tokio::process::Command::new(tasklist)
        .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
        .output()
        .await
        .unwrap();
    // CSV rows quote every field, so a match is language independent.
    String::from_utf8_lossy(&output.stdout).contains(&format!("\"{pid}\""))
}

/// Polls until the process `pid` is (or is not) running; false after about ten seconds.
#[cfg(windows)]
pub async fn wait_for_liveness(pid: u32, alive: bool) -> bool {
    for _ in 0..100 {
        if is_alive(pid).await == alive {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    false
}

/// Reads a pid the peer wrote to `file`, waiting for it to appear.
#[cfg(windows)]
pub async fn read_pid(file: &std::path::Path) -> u32 {
    for _ in 0..100 {
        if let Some(pid) = std::fs::read_to_string(file).ok().and_then(|t| t.trim().parse().ok()) {
            return pid;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("the peer never wrote its child's pid");
}
