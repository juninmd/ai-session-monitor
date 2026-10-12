//! `statusline-shim --out <file> [--chain <file>]`: the command the tray app installs as Claude
//! Code's `statusLine` (see `collector-claude`).
//!
//! Claude Code pipes a JSON document to the command on stdin; since v2.1.80 it carries
//! `rate_limits.five_hour` / `.seven_day` (`used_percentage`, `resets_at`), documented at
//! code.claude.com/docs/en/statusline. The shim
//!
//! 1. keeps ONLY that `rate_limits` object (never cwd, transcript path, model, cost or session id)
//!    and replaces `--out` atomically with `{"v":1,"capturedAt":<epoch s>,"rateLimits":<object|null>}`;
//! 2. runs the previous status line command saved in `--chain`, with the same stdin, and prints its
//!    stdout, so the user's own status line keeps working unchanged.
//!
//! It is on the hot path of every status line refresh, so it uses `std` and `serde_json` only. It
//! ALWAYS exits 0 and never prints a diagnostic: anything it wrote to stdout or stderr could end up
//! in the user's status line, and a failure here must cost the app a data point, not the user a
//! working terminal. The file is skipped when nothing changed in the last 5 s, and real limits are
//! kept through payloads that carry none (every session starts with one).

mod args;
mod atomic;
mod capture;
mod chain;
mod collect;
mod shell;
mod store;

use std::io::{self, Write};
use std::path::Path;

use capture::Payload;

fn main() {
    // A panic message or exit code 101 would reach Claude Code; swallow both.
    std::panic::set_hook(Box::new(|_| {}));
    let _ = std::panic::catch_unwind(run);
    std::process::exit(0);
}

fn run() {
    let args = args::parse(std::env::args_os().skip(1), args::home_from_env().as_deref());
    let payload = capture::read_payload(&mut io::stdin().lock());
    if let Some(out) = &args.out {
        store::record(out, &capture::extract_rate_limits(&payload));
    }
    let Some(output) = args.chain.as_deref().and_then(|file| forward(file, payload)) else {
        return;
    };
    let mut stdout = io::stdout().lock();
    let _ = stdout.write_all(&output);
    let _ = stdout.flush();
}

fn forward(chain_file: &Path, payload: Payload) -> Option<Vec<u8>> {
    let depth = std::env::var(chain::DEPTH_VAR).ok();
    if !chain::depth_allows_chaining(depth.as_deref()) {
        return None;
    }
    let command = chain::read_command(chain_file)?;
    let shell = shell::detect(&|name| std::env::var_os(name));
    let respect_execution_policy =
        std::env::var_os("CLAUDE_CODE_POWERSHELL_RESPECT_EXECUTION_POLICY")
            .is_some_and(|v| v == "1");
    let spec = chain::Run {
        shell: &shell,
        command: &command,
        respect_execution_policy,
        depth: depth.and_then(|text| text.trim().parse().ok()).unwrap_or(0),
        timeout: chain::CHAIN_TIMEOUT,
    };
    chain::run(&spec, payload)
}
