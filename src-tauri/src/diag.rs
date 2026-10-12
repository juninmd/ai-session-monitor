//! Diagnostics for a windowless app: one line on stderr, visible when started from a terminal and
//! discarded in release builds (no console). Callers pass fixed context strings and the Display of
//! window or OS errors; secrets never reach this module.

use std::fmt::Display;

#[allow(clippy::print_stderr)] // stderr is the only channel a tray app has before any window exists
pub fn warn(context: &str, detail: &dyn Display) {
    eprintln!("ai-session-monitor: {context}: {detail}");
}

/// Reports a startup failure and ends the process; there is nothing to fall back to.
pub fn fatal(context: &str, detail: &dyn Display) -> ! {
    warn(context, detail);
    std::process::exit(1)
}
