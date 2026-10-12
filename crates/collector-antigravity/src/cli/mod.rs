//! The `agy` command-line client as a quota source: `agy -p "/usage" --output-format json` prints
//! the same pools the IDE shows, costs no tokens and needs no CSRF token (the CLI embeds its
//! language server and keeps the token in memory, so the loopback path can never work for it).
//!
//! `Cli` is the seam: production runs the one fixed binary ([`SystemCli`]); tests hand the
//! collector a scripted double or a throwaway script in a temp dir.

mod parse;
mod system;

use std::future::Future;

use serde_json::Value;
use usage_model::{Ctx, HarnessError};

pub(crate) use parse::parse_usage;
pub(crate) use system::SystemCli;

/// Why there is no answer from the CLI.
#[derive(Clone, Debug)]
pub(crate) enum CliError {
    /// Not installed at the fixed location: the normal state of an IDE-only machine.
    Absent,
    /// Installed, but it did not give a usable answer.
    Failed(HarnessError),
}

pub(crate) trait Cli: Send + Sync {
    /// The parsed JSON the `/usage` command printed.
    fn usage(&self, ctx: &Ctx) -> impl Future<Output = Result<Value, CliError>> + Send;
}

#[cfg(test)]
mod tests;
