//! Test-only stand-in for "the user's previous status line command". The end-to-end tests chain
//! the shim to it, then check what came back, what it received and whether it was left running.
//!
//! Flags run in order: `--echo` prints `PEER:` plus the stdin, `--len` prints `LEN=<bytes>`,
//! `--env <NAME>` prints `<NAME>=<value>`, `--big <n>` prints n bytes, `--err <text>` writes to
//! stderr, `--exit <code>` sets the exit code, `--sleep-ms <n>` then `--marker <file>` write the
//! file only if still alive after n ms, `--until <file>` ends that sleep early once the file
//! exists, `--stamp <file>` records the exit time (epoch ms) just before exiting.
//! `--straggler-ms <n>` hands the `--marker` and `--until` given before it to a background copy of
//! this program that inherits stdout and lives n ms: an unredirected `cmd &` in the user's script.

use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn main() {
    let mut stdin = Vec::new();
    let _ = std::io::stdin().read_to_end(&mut stdin);
    let mut out = std::io::stdout().lock();
    let mut args = std::env::args().skip(1);
    let (mut exit_code, mut sleep_ms) = (0, 0_u64);
    let (mut marker, mut until, mut stamp) = (None, None, None);

    while let Some(flag) = args.next() {
        let mut value = || args.next().unwrap_or_default();
        let _ = match flag.as_str() {
            "--echo" => out.write_all(b"PEER:").and_then(|()| out.write_all(&stdin)),
            "--len" => write!(out, "LEN={}", stdin.len()),
            "--env" => {
                let name = value();
                write!(out, "{name}={}", std::env::var(&name).unwrap_or_default())
            }
            "--big" => out.write_all(&vec![b'x'; value().parse().unwrap_or(0)]),
            "--err" => std::io::stderr().write_all(value().as_bytes()),
            "--exit" => {
                exit_code = value().parse().unwrap_or(1);
                Ok(())
            }
            "--sleep-ms" => {
                sleep_ms = value().parse().unwrap_or(0);
                Ok(())
            }
            "--marker" => {
                marker = Some(value());
                Ok(())
            }
            "--until" => {
                until = Some(value());
                Ok(())
            }
            "--stamp" => {
                stamp = Some(value());
                Ok(())
            }
            "--straggler-ms" => spawn_straggler(&value(), until.take(), marker.take()),
            _ => Ok(()),
        };
    }
    let _ = out.flush();
    drop(out);

    wait(sleep_ms, until.as_deref());
    if let Some(path) = marker {
        let _ = std::fs::write(path, b"survived");
    }
    if let Some(path) = stamp {
        let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        let _ = std::fs::write(path, since_epoch.as_millis().to_string());
    }
    std::process::exit(exit_code);
}

/// Sleeps `ms`, or less when `until` names a file that shows up first.
fn wait(ms: u64, until: Option<&str>) {
    let end = Instant::now() + Duration::from_millis(ms);
    while Instant::now() < end && !until.is_some_and(|file| Path::new(file).exists()) {
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn spawn_straggler(ms: &str, until: Option<String>, marker: Option<String>) -> std::io::Result<()> {
    let mut straggler = Command::new(std::env::current_exe()?);
    straggler.args(["--sleep-ms", ms]);
    for (flag, file) in [("--until", until), ("--marker", marker)] {
        straggler.args(file.iter().flat_map(|file| [flag, file.as_str()]));
    }
    // stdout stays inherited on purpose: the pipe left open is what this stands in for.
    straggler.stdin(Stdio::null()).stdout(Stdio::inherit()).stderr(Stdio::null()).spawn().map(drop)
}
