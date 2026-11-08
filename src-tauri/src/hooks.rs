//! Starts the user's event hooks. The program is run directly, never through a shell, with no
//! console window, its output discarded, and killed if it outlives `LIMIT`. Nothing waits on it:
//! a slow or failing hook cannot delay a snapshot or a toast.

use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use app_core::HookRun;

const LIMIT: Duration = Duration::from_secs(30);
const POLL: Duration = Duration::from_millis(200);
/// `CREATE_NO_WINDOW`, so a console program does not flash a window over the desktop.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Starts `run` and watches it on a helper thread. A program that cannot start is ignored.
pub fn start(run: &HookRun) {
    let mut command = Command::new(&run.program);
    command
        .args(&run.args)
        .envs(run.env.iter().map(|(key, value)| (*key, value.as_str())))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    if let Ok(child) = command.spawn() {
        std::thread::spawn(move || watch(child, LIMIT));
    }
}

/// Waits for `child` until it exits or `limit` passes, then kills it.
fn watch(mut child: Child, limit: Duration) {
    let deadline = Instant::now() + limit;
    loop {
        match child.try_wait() {
            Ok(None) if Instant::now() < deadline => std::thread::sleep(POLL),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return;
            }
            Ok(Some(_)) | Err(_) => return,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_program_is_ignored_without_panic() {
        let run = HookRun {
            program: "/nonexistent/aism-hook-test".to_owned(),
            args: Vec::new(),
            env: Vec::new(),
        };
        start(&run);
    }
}
