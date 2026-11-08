//! Maps one planned notification to the user's event hooks. Pure: the shell starts the programs.
//! Hooks run for every quota event, including during quiet hours, because they are not toasts.

use usage_model::{HookEvent, Settings};

use crate::notify::{Notification, NotificationKind};

/// One program to start. `env` is the only place the event reaches it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HookRun {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(&'static str, String)>,
}

/// The hook runs for `note`, in the order the hooks are configured.
pub(crate) fn plan(note: &Notification, settings: &Settings) -> Vec<HookRun> {
    let Some(event) = event_of(note.kind) else { return Vec::new() };
    settings
        .general
        .event_hooks
        .iter()
        .filter(|hook| hook.event == event)
        .map(|hook| HookRun {
            program: hook.program.clone(),
            args: hook.args.clone(),
            env: vec![
                ("AISM_HOOK_EVENT", event.as_str().to_owned()),
                ("AISM_HOOK_HARNESS", note.harness.as_str().to_owned()),
                ("AISM_HOOK_WINDOW", note.window_id.clone()),
            ],
        })
        .collect()
}

fn event_of(kind: NotificationKind) -> Option<HookEvent> {
    match kind {
        NotificationKind::WindowReset => Some(HookEvent::Reset),
        NotificationKind::WindowOut => Some(HookEvent::Out),
        NotificationKind::ThresholdCrossed => Some(HookEvent::Threshold),
        NotificationKind::PaceRisk | NotificationKind::SettingsRecovered => None,
    }
}

#[cfg(test)]
mod tests;
