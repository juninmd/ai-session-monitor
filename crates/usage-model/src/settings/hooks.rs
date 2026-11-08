//! Event hooks: a program the user names to run when a quota event happens. The program is started
//! directly, never through a shell, so its arguments are exactly the stored ones. The event, the
//! harness and the window reach it only as environment variables, never as argument text.

use serde::{Deserialize, Serialize};

/// At most this many hooks are kept; the rest are dropped on load.
pub const MAX_EVENT_HOOKS: usize = 8;
/// Arguments per hook.
pub const MAX_HOOK_ARGS: usize = 16;
const MAX_TEXT: usize = 256;
const MAX_PROGRAM: usize = 260;

/// The quota event a hook is run for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HookEvent {
    /// A window that was out has quota again.
    Reset,
    /// A window just ran out of quota.
    Out,
    /// A window's used percentage crossed one of the alert thresholds.
    Threshold,
}

impl HookEvent {
    /// The value passed to the program in `AISM_HOOK_EVENT`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reset => "reset",
            Self::Out => "out",
            Self::Threshold => "threshold",
        }
    }
}

/// One hook: run `program` with `args` when `event` happens.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventHook {
    pub event: HookEvent,
    /// Absolute path to the program. A bare name is refused: nothing is searched on `PATH`.
    pub program: String,
    pub args: Vec<String>,
}

impl EventHook {
    /// Whether the hook may be stored and run: an absolute program path and bounded, printable
    /// text in every field.
    pub fn is_valid(&self) -> bool {
        let program = self.program.trim();
        program.len() <= MAX_PROGRAM
            && !program.is_empty()
            && std::path::Path::new(program).is_absolute()
            && clean(program)
            && self.args.len() <= MAX_HOOK_ARGS
            && self.args.iter().all(|arg| arg.len() <= MAX_TEXT && clean(arg))
    }
}

/// No control characters, which would break the line the user sees or the process arguments.
fn clean(text: &str) -> bool {
    !text.chars().any(char::is_control)
}

/// Drops invalid hooks and keeps at most `MAX_EVENT_HOOKS` of the rest, in their order.
pub(crate) fn normalize_hooks(hooks: &mut Vec<EventHook>) {
    for hook in hooks.iter_mut() {
        hook.program = hook.program.trim().to_owned();
    }
    hooks.retain(EventHook::is_valid);
    hooks.truncate(MAX_EVENT_HOOKS);
}
