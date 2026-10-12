use usage_model::{HarnessView, UsageWindow, WindowKind};

use super::*;

const NOW: i64 = 1_800_000_000;
const OUT: NotificationKind = NotificationKind::WindowOut;
const RESET: NotificationKind = NotificationKind::WindowReset;

fn settings(notify_on_out: bool, notify_on_reset: bool) -> Settings {
    let mut settings = Settings::default();
    settings.general.notify_on_out = notify_on_out;
    settings.general.notify_on_reset = notify_on_reset;
    settings
}

#[derive(Clone, Copy)]
struct Reading {
    state: ViewState,
    stale: bool,
    used: Option<f64>,
    resets_in: Option<i64>,
}

const fn fresh(used: f64) -> Reading {
    Reading { state: ViewState::Ready, stale: false, used: Some(used), resets_in: Some(3_600) }
}

const fn flagged_stale(used: f64) -> Reading {
    Reading { stale: true, ..fresh(used) }
}

const fn expired(used: f64) -> Reading {
    Reading { resets_in: Some(-60), ..fresh(used) }
}

fn view(harness: HarnessId, reading: Option<Reading>) -> HarnessView {
    let Some(reading) = reading else { return HarnessView::loading(harness) };
    let mut window = UsageWindow::new("five_hour", WindowKind::FiveHour);
    window.used_pct = reading.used;
    window.resets_at = reading.resets_in.map(|secs| NOW + secs);
    let mut view = HarnessView::with_state(harness, reading.state);
    view.stale = reading.stale;
    view.windows = vec![window];
    view
}

fn snapshot(reading: Option<Reading>) -> Snapshot {
    Snapshot { generated_at: NOW, views: vec![view(HarnessId::Claude, reading)] }
}

fn kinds(
    prev: Option<Reading>,
    next: Option<Reading>,
    out: bool,
    reset: bool,
) -> Vec<NotificationKind> {
    plan(&snapshot(prev), &snapshot(next), &settings(out, reset), NOW)
        .into_iter()
        .map(|n| n.kind)
        .collect()
}

#[test]
fn transitions_table() {
    let error = Reading { state: ViewState::Error, ..fresh(100.0) };
    let unknown = Reading { used: None, ..fresh(0.0) };
    // (name, previous reading, next reading, notify_on_out, notify_on_reset, expected)
    let cases = vec![
        ("available to out", Some(fresh(40.0)), Some(fresh(100.0)), true, true, vec![OUT]),
        ("available to out, flag off", Some(fresh(40.0)), Some(fresh(100.0)), false, true, vec![]),
        ("out to available", Some(fresh(100.0)), Some(fresh(10.0)), true, true, vec![RESET]),
        ("out to available, flag off", Some(fresh(100.0)), Some(fresh(10.0)), true, false, vec![]),
        ("both flags off", Some(fresh(40.0)), Some(fresh(100.0)), false, false, vec![]),
        ("available stays available", Some(fresh(40.0)), Some(fresh(60.0)), true, true, vec![]),
        ("out stays out", Some(fresh(100.0)), Some(fresh(100.0)), true, true, vec![]),
        ("over budget counts as out", Some(fresh(40.0)), Some(fresh(130.0)), true, true, vec![OUT]),
        ("99.9 used is still available", Some(fresh(40.0)), Some(fresh(99.9)), true, true, vec![]),
        ("first reading: nothing before it", None, Some(fresh(100.0)), true, true, vec![]),
        ("first reading available", None, Some(fresh(10.0)), true, true, vec![]),
        ("next view is stale", Some(fresh(40.0)), Some(flagged_stale(100.0)), true, true, vec![]),
        (
            "next window already expired (out)",
            Some(fresh(40.0)),
            Some(expired(100.0)),
            true,
            true,
            vec![],
        ),
        (
            "next window already expired (reset)",
            Some(fresh(100.0)),
            Some(expired(10.0)),
            true,
            true,
            vec![],
        ),
        ("next view is an error", Some(fresh(40.0)), Some(error), true, true, vec![]),
        ("next percentage unknown", Some(fresh(40.0)), Some(unknown), true, true, vec![]),
        ("previous percentage unknown", Some(unknown), Some(fresh(100.0)), true, true, vec![]),
        (
            "stale previous reading still counts: out",
            Some(flagged_stale(40.0)),
            Some(fresh(100.0)),
            true,
            true,
            vec![OUT],
        ),
        (
            "expired previous out, fresh available",
            Some(expired(100.0)),
            Some(fresh(5.0)),
            true,
            true,
            vec![RESET],
        ),
        ("stale out stays out", Some(flagged_stale(100.0)), Some(fresh(100.0)), true, true, vec![]),
        (
            "window without a reset time",
            Some(fresh(40.0)),
            Some(Reading { resets_in: None, ..fresh(100.0) }),
            true,
            true,
            vec![OUT],
        ),
    ];
    for (name, prev, next, out, reset, expected) in cases {
        assert_eq!(kinds(prev, next, out, reset), expected, "{name}");
    }
}

#[test]
fn a_flapping_window_notifies_once_per_transition() {
    let sequence = [10.0, 100.0, 10.0, 100.0, 100.0, 10.0, 10.0];
    let mut seen = Vec::new();
    for pair in sequence.windows(2) {
        seen.push(kinds(Some(fresh(pair[0])), Some(fresh(pair[1])), true, true));
    }
    let flat: Vec<_> = seen.into_iter().flatten().collect();
    assert_eq!(flat, [OUT, RESET, OUT, RESET]);
}

#[test]
fn a_stale_blip_between_two_readings_adds_no_notification() {
    let steps = |readings: &[Reading]| -> Vec<NotificationKind> {
        readings
            .windows(2)
            .flat_map(|pair| kinds(Some(pair[0]), Some(pair[1]), true, true))
            .collect()
    };
    // out, the source fails (old data kept, stale), out again: nothing new happened.
    assert!(steps(&[fresh(100.0), flagged_stale(100.0), fresh(100.0)]).is_empty());
    // available, blip, available.
    assert!(steps(&[fresh(40.0), flagged_stale(40.0), fresh(40.0)]).is_empty());
    // available, blip, out: exactly one "out", delivered when fresh data confirms it.
    assert_eq!(steps(&[fresh(40.0), flagged_stale(40.0), fresh(100.0)]), [OUT]);
    // out, window expires unseen, fresh data shows it is back: one "reset".
    assert_eq!(steps(&[fresh(100.0), expired(100.0), fresh(3.0)]), [RESET]);
    // out, blip, out then back: one reset only.
    assert_eq!(steps(&[fresh(100.0), flagged_stale(100.0), fresh(100.0), fresh(0.0)]), [RESET]);
}

#[test]
fn windows_are_matched_by_id_and_a_new_or_vanished_window_never_notifies() {
    let mut weekly = UsageWindow::new("weekly", WindowKind::Weekly);
    weekly.used_pct = Some(100.0);
    weekly.resets_at = Some(NOW + 600);
    let prev = snapshot(Some(fresh(40.0)));
    let mut next = snapshot(Some(fresh(40.0)));
    next.views[0].windows.push(weekly.clone());
    assert!(plan(&prev, &next, &settings(true, true), NOW).is_empty(), "new window, already out");
    assert!(plan(&next, &prev, &settings(true, true), NOW).is_empty(), "window disappeared");

    let mut flipped = next.clone();
    flipped.views[0].windows[0].used_pct = Some(100.0);
    flipped.views[0].windows[1].used_pct = Some(30.0);
    let mut before = next;
    before.views[0].windows[1].used_pct = Some(100.0);
    let planned = plan(&before, &flipped, &settings(true, true), NOW);
    let got: Vec<_> = planned.iter().map(|n| (n.kind, n.window_id.as_str())).collect();
    assert_eq!(got, [(OUT, "five_hour"), (RESET, "weekly")]);
}

#[test]
fn several_harnesses_notify_in_tab_order() {
    let build = |claude: f64, litellm: f64| Snapshot {
        generated_at: NOW,
        views: vec![
            view(HarnessId::Claude, Some(fresh(claude))),
            view(HarnessId::Litellm, Some(fresh(litellm))),
        ],
    };
    let planned = plan(&build(40.0, 40.0), &build(100.0, 100.0), &settings(true, true), NOW);
    let order: Vec<_> = planned.iter().map(|n| n.harness).collect();
    assert_eq!(order, [HarnessId::Claude, HarnessId::Litellm]);
}

#[test]
fn a_harness_missing_from_the_previous_snapshot_is_treated_as_new() {
    let prev = Snapshot { generated_at: NOW, views: Vec::new() };
    assert!(plan(&prev, &snapshot(Some(fresh(100.0))), &settings(true, true), NOW).is_empty());
}

#[test]
fn the_out_text_names_harness_window_and_reset_time_in_portuguese() {
    let note = &plan(
        &snapshot(Some(fresh(40.0))),
        &snapshot(Some(Reading { resets_in: Some(113 * 60), ..fresh(100.0) })),
        &settings(true, false),
        NOW,
    )[0];
    assert_eq!(note.kind, OUT);
    assert_eq!(note.harness, HarnessId::Claude);
    assert_eq!(note.window_id, "five_hour");
    assert_eq!(note.title, "Claude Code: limite esgotado");
    assert_eq!(note.body, "A janela de 5 horas esgotou. Reinicia em 1 h 53 min.");
}

#[test]
fn the_reset_text_reports_what_is_free_and_when_the_next_reset_is() {
    let note = &plan(
        &snapshot(Some(fresh(100.0))),
        &snapshot(Some(Reading { resets_in: Some(5 * 3600), ..fresh(12.4) })),
        &settings(false, true),
        NOW,
    )[0];
    assert_eq!(note.kind, RESET);
    assert_eq!(note.title, "Claude Code: limite disponível");
    assert_eq!(
        note.body,
        "A janela de 5 horas voltou a ter limite disponível (88% livre). Próximo reinício em 5 h."
    );
}

#[test]
fn texts_cope_with_budgets_labels_and_missing_reset_times() {
    let mut budget = UsageWindow::new("budget", WindowKind::Budget);
    budget.label = Some("  team-a\u{7}\n".into());
    budget.used_pct = Some(100.0);
    let build = |used: f64| {
        let mut w = budget.clone();
        w.used_pct = Some(used);
        let mut v = HarnessView::with_state(HarnessId::Litellm, ViewState::Ready);
        v.windows = vec![w];
        Snapshot { generated_at: NOW, views: vec![v] }
    };
    let out = &plan(&build(10.0), &build(100.0), &settings(true, true), NOW)[0];
    assert_eq!(out.title, "LiteLLM: limite esgotado");
    assert_eq!(out.body, "O orçamento (team-a) esgotou. O horário de reinício não foi informado.");
    let back = &plan(&build(100.0), &build(0.0), &settings(true, true), NOW)[0];
    assert_eq!(back.body, "O orçamento (team-a) voltou a ter limite disponível (100% livre).");
}

#[test]
fn planning_is_pure() {
    let prev = snapshot(Some(fresh(40.0)));
    let next = snapshot(Some(fresh(100.0)));
    let settings = settings(true, true);
    assert_eq!(plan(&prev, &next, &settings, NOW), plan(&prev, &next, &settings, NOW));
}

#[test]
fn the_recovery_notice_names_the_copy_and_needs_no_window() {
    let note = settings_recovered("settings.corrupt-1800000000.json");
    assert_eq!(note.kind, NotificationKind::SettingsRecovered);
    assert!(note.window_id.is_empty());
    assert_eq!(note.title, "Configurações restauradas");
    assert!(note.body.contains("settings.corrupt-1800000000.json"));
}
