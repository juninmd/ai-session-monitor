//! Claude Code's own `CLAUDE_CONFIG_DIR`. The Claude collector never reads the environment, so the
//! shell reads it once at startup and fills the setting in when the user left the folder blank.

use crate::paths::Overrides;

#[cfg(test)]
mod tests;

const VAR: &str = "CLAUDE_CONFIG_DIR";

/// The folder Claude Code was told to use, if any. A test run never follows it: the variable
/// usually points at the real profile that the `AISM_*` switches exist to avoid.
pub fn from_env(overrides: &Overrides) -> Option<String> {
    pick(overrides.is_active(), std::env::var(VAR).ok())
}

fn pick(test_run: bool, value: Option<String>) -> Option<String> {
    if test_run {
        return None;
    }
    value.map(|dir| dir.trim().to_owned()).filter(|dir| !dir.is_empty())
}
