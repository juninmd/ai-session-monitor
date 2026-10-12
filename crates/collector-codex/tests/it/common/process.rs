//! Looking at processes from the outside, to prove that none is left behind.

use std::path::Path;
use std::time::{Duration, Instant};

#[cfg(windows)]
pub async fn is_alive(pid: u32) -> bool {
    let root = std::env::var_os("SystemRoot").unwrap_or_default();
    let tasklist = Path::new(&root).join("System32").join("tasklist.exe");
    let output = tokio::process::Command::new(tasklist)
        .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
        .output()
        .await
        .unwrap();
    // CSV rows quote every field, so a match does not depend on the display language.
    String::from_utf8_lossy(&output.stdout).contains(&format!("\"{pid}\""))
}

#[cfg(not(windows))]
pub async fn is_alive(pid: u32) -> bool {
    let status = tokio::process::Command::new("kill").args(["-0", &pid.to_string()]).status();
    status.await.is_ok_and(|status| status.success())
}

/// How long a process may take to disappear, or a pid file to appear. Wall time, not a count of
/// polls: `tasklist` itself can take a second on a busy machine.
const PATIENCE: Duration = Duration::from_secs(15);

pub async fn wait_until_gone(pid: u32) -> bool {
    let give_up = Instant::now() + PATIENCE;
    loop {
        if !is_alive(pid).await {
            return true;
        }
        if Instant::now() >= give_up {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub async fn wait_for_pid(file: &Path) -> Option<u32> {
    let give_up = Instant::now() + PATIENCE;
    loop {
        if let Some(pid) = read_pid(file) {
            return Some(pid);
        }
        if Instant::now() >= give_up {
            return None;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub fn read_pid(file: &Path) -> Option<u32> {
    std::fs::read_to_string(file).ok().and_then(|text| text.trim().parse().ok())
}
