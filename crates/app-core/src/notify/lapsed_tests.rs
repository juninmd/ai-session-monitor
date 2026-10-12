use usage_model::{HarnessView, UsageWindow, WindowKind};

use super::lapsed::{ResetKey, key_of, lapsed_resets, next_lapse, still_waiting};
use super::*;

const SHOWN_AT: i64 = 1_800_000_000;
const RESET: i64 = SHOWN_AT + 600;
const AFTER: i64 = RESET + 5;

fn settings(notify_on_reset: bool) -> Settings {
    let mut settings = Settings::default();
    settings.general.notify_on_reset = notify_on_reset;
    settings
}

fn window(id: &str, used: Option<f64>, resets_at: Option<i64>) -> UsageWindow {
    let kind = if id == "weekly" { WindowKind::Weekly } else { WindowKind::FiveHour };
    let mut window = UsageWindow::new(id, kind);
    window.used_pct = used;
    window.resets_at = resets_at;
    window
}

fn out(id: &str, resets_at: i64) -> UsageWindow {
    window(id, Some(100.0), Some(resets_at))
}

fn view(harness: HarnessId, state: ViewState, windows: Vec<UsageWindow>) -> HarnessView {
    let mut view = HarnessView::with_state(harness, state);
    view.windows = windows;
    view
}

fn ready(windows: Vec<UsageWindow>) -> HarnessView {
    view(HarnessId::Claude, ViewState::Ready, windows)
}

fn shown(views: Vec<HarnessView>) -> Snapshot {
    Snapshot { generated_at: SHOWN_AT, views }
}

fn key(id: &str, at: i64) -> ResetKey {
    (HarnessId::Claude, id.to_owned(), at)
}

fn keys(found: Vec<(ResetKey, Notification)>) -> Vec<ResetKey> {
    found.into_iter().map(|(key, _)| key).collect()
}

/// What the loop asks at `now`: `shown` is the last snapshot, `now_views` what the hub holds.
fn lapsed(before: &Snapshot, now_views: &[HarnessView], now: i64) -> Vec<ResetKey> {
    keys(lapsed_resets(before, now_views, &settings(true), now))
}

#[test]
fn an_out_window_whose_reset_passed_with_no_newer_reading_is_announced_once_keyed() {
    let before = shown(vec![ready(vec![out("five_hour", RESET)])]);
    let now = [ready(vec![out("five_hour", RESET)])];

    let found = lapsed_resets(&before, &now, &settings(true), AFTER);

    assert_eq!(found.len(), 1);
    let (key, note) = &found[0];
    assert_eq!(*key, self::key("five_hour", RESET));
    assert_eq!((note.kind, note.harness), (NotificationKind::WindowReset, HarnessId::Claude));
    assert_eq!(note.window_id, "five_hour");
    assert_eq!(note.title, "Claude Code: limite disponível");
    assert_eq!(note.body, "A janela de 5 horas reiniciou e voltou a ter limite disponível.");
}

#[test]
fn the_boundary_is_the_reset_second_itself() {
    let before = shown(vec![ready(vec![out("five_hour", RESET)])]);
    let now = [ready(vec![out("five_hour", RESET)])];
    assert!(lapsed(&before, &now, RESET - 1).is_empty(), "one second early");
    assert_eq!(lapsed(&before, &now, RESET), [key("five_hour", RESET)], "resets_at == now");
}

#[test]
fn nothing_is_announced_unless_the_user_asked_for_resets() {
    let before = shown(vec![ready(vec![out("five_hour", RESET)])]);
    let now = [ready(vec![out("five_hour", RESET)])];
    assert!(lapsed_resets(&before, &now, &settings(false), AFTER).is_empty());
}

#[test]
fn only_a_window_that_was_out_and_still_ahead_of_its_reset_when_shown_counts() {
    let now = [ready(vec![out("five_hour", RESET)])];
    let was_available = shown(vec![ready(vec![window("five_hour", Some(40.0), Some(RESET))])]);
    assert!(lapsed(&was_available, &now, AFTER).is_empty(), "it was not out");

    let was_unknown = shown(vec![ready(vec![window("five_hour", None, Some(RESET))])]);
    assert!(lapsed(&was_unknown, &now, AFTER).is_empty(), "unknown is not out");

    let first_seen_over =
        Snapshot { generated_at: RESET, views: vec![ready(vec![out("five_hour", RESET)])] };
    assert!(lapsed(&first_seen_over, &now, AFTER).is_empty(), "already reset when first shown");

    let not_shown_at_all = shown(vec![ready(Vec::new())]);
    assert!(lapsed(&not_shown_at_all, &now, AFTER).is_empty(), "never seen before");
    assert!(lapsed(&shown(Vec::new()), &now, AFTER).is_empty(), "no such harness before");
}

#[test]
fn a_newer_reading_that_says_otherwise_is_left_to_the_regular_path() {
    let before = shown(vec![ready(vec![out("five_hour", RESET)])]);
    let fresh = [ready(vec![window("five_hour", Some(3.0), Some(RESET + 18_000))])];
    assert!(lapsed(&before, &fresh, AFTER).is_empty(), "back to available with a new reset");
    let moved = [ready(vec![out("five_hour", RESET + 18_000)])];
    assert!(lapsed(&before, &moved, AFTER).is_empty(), "out again in a later window");
    let unknown = [ready(vec![window("five_hour", None, Some(RESET))])];
    assert!(lapsed(&before, &unknown, AFTER).is_empty(), "no longer known to be out");
    let renamed = [ready(vec![out("weekly", RESET)])];
    assert!(lapsed(&before, &renamed, AFTER).is_empty(), "another window is another window");
}

#[test]
fn a_window_is_out_only_with_nothing_left_whether_it_was_or_is_shown() {
    let out_now = [ready(vec![out("five_hour", RESET)])];
    let out_before = shown(vec![ready(vec![out("five_hour", RESET)])]);
    // Past 100 % used is out as well; a sliver left is still available.
    for (used, counts) in [(100.0, true), (130.0, true), (99.9, false), (99.0, false)] {
        let same_reset = || vec![ready(vec![window("five_hour", Some(used), Some(RESET))])];
        let before = shown(same_reset());
        assert_eq!(lapsed(&before, &out_now, AFTER).len(), usize::from(counts), "was {used}%");
        assert_eq!(next_lapse(&before, &settings(true), SHOWN_AT).is_some(), counts, "{used}%");
        assert_eq!(lapsed(&out_before, &same_reset(), AFTER).len(), usize::from(counts), "{used}%");
    }
}

#[test]
fn only_a_ready_view_can_announce_but_stale_data_still_counts() {
    let before = shown(vec![ready(vec![out("five_hour", RESET)])]);
    for state in [ViewState::Loading, ViewState::NoData, ViewState::NeedsConsent, ViewState::Error]
    {
        let now = [view(HarnessId::Claude, state, vec![out("five_hour", RESET)])];
        assert!(lapsed(&before, &now, AFTER).is_empty(), "{state:?}");
    }
    let mut stale = ready(vec![out("five_hour", RESET)]);
    stale.stale = true;
    assert_eq!(lapsed(&before, &[stale], AFTER), [key("five_hour", RESET)]);
}

#[test]
fn a_source_whose_proxy_applies_resets_late_is_left_to_its_confirming_reading() {
    let litellm = || view(HarnessId::Litellm, ViewState::Ready, vec![out("budget", RESET)]);
    let before = shown(vec![litellm(), ready(vec![out("five_hour", RESET)])]);
    let now = [litellm(), ready(vec![out("five_hour", RESET)])];

    assert_eq!(lapsed(&before, &now, AFTER), [key("five_hour", RESET)], "only the other source");
    let alone = shown(vec![litellm()]);
    assert_eq!(next_lapse(&alone, &settings(true), SHOWN_AT), None, "and no wake-up for it");
}

#[test]
fn each_window_is_judged_on_its_own_reset_across_harnesses() {
    let codex = |windows| view(HarnessId::Codex, ViewState::Ready, windows);
    let before = shown(vec![
        ready(vec![out("five_hour", RESET), out("weekly", RESET + 7200), window("x", None, None)]),
        codex(vec![out("five_hour", RESET + 60)]),
    ]);
    let now = [
        ready(vec![out("five_hour", RESET), out("weekly", RESET + 7200), window("x", None, None)]),
        codex(vec![out("five_hour", RESET + 60)]),
    ];
    assert_eq!(lapsed(&before, &now, RESET + 30), [key("five_hour", RESET)]);
    let both = lapsed(&before, &now, RESET + 60);
    assert_eq!(
        both,
        [key("five_hour", RESET), (HarnessId::Codex, "five_hour".to_owned(), RESET + 60)]
    );
    let all = lapsed(&before, &now, RESET + 7200);
    assert_eq!(all.len(), 3, "claude five_hour and weekly, codex five_hour");
}

#[test]
fn next_lapse_is_the_earliest_reset_still_ahead_of_an_out_window() {
    let before = shown(vec![ready(vec![
        out("weekly", RESET + 7200),
        out("five_hour", RESET),
        window("other", Some(10.0), Some(RESET - 100)),
        window("unknown", None, Some(RESET - 50)),
        window("open", Some(100.0), None),
    ])]);
    let on = settings(true);
    assert_eq!(
        next_lapse(&before, &on, SHOWN_AT),
        Some(RESET),
        "available/unknown/open are ignored"
    );
    assert_eq!(next_lapse(&before, &on, RESET - 1), Some(RESET));
    assert_eq!(
        next_lapse(&before, &on, RESET),
        Some(RESET + 7200),
        "resets at or before now are not ahead"
    );
    assert_eq!(next_lapse(&before, &on, RESET + 7200), None);
    assert_eq!(next_lapse(&before, &settings(false), SHOWN_AT), None, "nobody asked");
}

#[test]
fn next_lapse_ignores_windows_that_were_already_over_when_shown() {
    let over =
        Snapshot { generated_at: RESET + 10, views: vec![ready(vec![out("five_hour", RESET)])] };
    assert_eq!(
        next_lapse(&over, &settings(true), RESET - 100),
        None,
        "they can never be announced"
    );
}

#[test]
fn next_lapse_only_looks_at_ready_views() {
    let before =
        shown(vec![view(HarnessId::Claude, ViewState::Error, vec![out("five_hour", RESET)])]);
    assert_eq!(next_lapse(&before, &settings(true), SHOWN_AT), None);
}

#[test]
fn a_regular_reset_notification_maps_back_to_the_reset_it_confirms() {
    let before = shown(vec![ready(vec![out("five_hour", RESET), out("weekly", RESET + 99)])]);
    let note = |id: &str| Notification {
        kind: NotificationKind::WindowReset,
        harness: HarnessId::Claude,
        window_id: id.to_owned(),
        title: String::new(),
        body: String::new(),
    };
    assert_eq!(key_of(&before, &note("five_hour")), Some(key("five_hour", RESET)));
    assert_eq!(key_of(&before, &note("weekly")), Some(key("weekly", RESET + 99)));
    assert_eq!(key_of(&before, &note("nope")), None);
    let other_harness = Notification { harness: HarnessId::Codex, ..note("five_hour") };
    assert_eq!(key_of(&before, &other_harness), None);
    let open_ended = shown(vec![ready(vec![window("five_hour", Some(100.0), None)])]);
    assert_eq!(key_of(&open_ended, &note("five_hour")), None, "no reset time, no key");
    for kind in [NotificationKind::WindowOut, NotificationKind::SettingsRecovered] {
        let other_kind = Notification { kind, ..note("five_hour") };
        assert_eq!(key_of(&before, &other_kind), None, "only a reset notice confirms a reset");
    }
}

#[test]
fn a_key_is_kept_only_while_its_window_is_still_shown_out_with_that_reset() {
    let now = shown(vec![ready(vec![out("five_hour", RESET)])]);
    assert!(still_waiting(&now, &key("five_hour", RESET)));
    assert!(!still_waiting(&now, &key("five_hour", RESET + 1)), "a later reset is another event");
    assert!(!still_waiting(&now, &key("weekly", RESET)), "window gone");
    assert!(
        !still_waiting(&now, &(HarnessId::Codex, "five_hour".to_owned(), RESET)),
        "harness gone"
    );
    let back = shown(vec![ready(vec![window("five_hour", Some(5.0), Some(RESET))])]);
    assert!(
        !still_waiting(&back, &key("five_hour", RESET)),
        "available again: nothing to suppress"
    );
}
