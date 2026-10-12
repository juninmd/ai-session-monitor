//! Stands in for `codex app-server` in the integration tests (`CARGO_BIN_EXE_fake_app_server`).
//! It speaks the same newline-delimited JSON-RPC subset as the real one and plays one scripted
//! behaviour per run. The behaviour comes from the `fake-mode` file next to the executable (tests
//! copy the executable into a directory of their own, because the environment cannot be set per
//! test without `unsafe`), else from `FAKE_MODE`, else it is `ok`. Every value here is invented.
//!
//! Files it leaves next to the executable: `fake-pid` (its own pid), `fake-cwd` (the directory it
//! was started in), `fake-child-pid` (the grandchild of the `grandchild` mode) and
//! `fake-received.log` (each line it received).

mod hold;
mod log;
mod mode;
mod replies;
mod server;

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("app-server") => serve(),
        Some("hold") => {
            hold::run();
            ExitCode::SUCCESS
        }
        _ => ExitCode::from(2),
    }
}

fn serve() -> ExitCode {
    let dir = std::env::current_exe().ok().and_then(|exe| exe.parent().map(PathBuf::from));
    let Some(dir) = dir else { return ExitCode::from(2) };
    let Some(mode) = mode::load(&dir) else { return ExitCode::from(2) };
    match server::run(mode, &dir) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(1),
    }
}
