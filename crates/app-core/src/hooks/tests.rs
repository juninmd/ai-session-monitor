use usage_model::{EventHook, HarnessId, HookEvent, Settings};

use super::plan;
use crate::notify::{Notification, NotificationKind};

fn note(kind: NotificationKind) -> Notification {
    Notification {
        kind,
        harness: HarnessId::Claude,
        window_id: "five_hour".to_owned(),
        title: "t".to_owned(),
        body: "b".to_owned(),
    }
}

fn hook(event: HookEvent, args: &[&str]) -> EventHook {
    EventHook {
        event,
        program: "/usr/bin/notify-send".to_owned(),
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
    }
}

fn with_hooks(hooks: Vec<EventHook>) -> Settings {
    let mut settings = Settings::default();
    settings.general.event_hooks = hooks;
    settings
}

#[test]
fn no_hooks_plans_nothing() {
    let runs = plan(&note(NotificationKind::WindowReset), &Settings::default());
    assert!(runs.is_empty());
}

#[test]
fn only_the_matching_event_runs_its_hooks() {
    let settings = with_hooks(vec![
        hook(HookEvent::Reset, &["a"]),
        hook(HookEvent::Out, &["b"]),
        hook(HookEvent::Reset, &["c"]),
    ]);
    let runs = plan(&note(NotificationKind::WindowReset), &settings);
    let args: Vec<_> = runs.iter().map(|run| run.args.clone()).collect();
    assert_eq!(args, vec![vec!["a".to_owned()], vec!["c".to_owned()]]);
}

#[test]
fn event_reaches_the_program_only_as_environment() {
    let settings = with_hooks(vec![hook(HookEvent::Threshold, &["--flag"])]);
    let runs = plan(&note(NotificationKind::ThresholdCrossed), &settings);
    assert_eq!(runs.len(), 1);
    let run = &runs[0];
    assert_eq!(run.args, vec!["--flag".to_owned()]);
    assert!(run.env.contains(&("AISM_HOOK_EVENT", "threshold".to_owned())));
    assert!(run.env.contains(&("AISM_HOOK_WINDOW", "five_hour".to_owned())));
    assert!(!run.args.iter().any(|arg| arg.contains("five_hour")));
}

#[test]
fn pace_and_recovery_notices_run_nothing() {
    let settings = with_hooks(vec![
        hook(HookEvent::Reset, &[]),
        hook(HookEvent::Out, &[]),
        hook(HookEvent::Threshold, &[]),
    ]);
    assert!(plan(&note(NotificationKind::PaceRisk), &settings).is_empty());
    assert!(plan(&note(NotificationKind::SettingsRecovered), &settings).is_empty());
}
