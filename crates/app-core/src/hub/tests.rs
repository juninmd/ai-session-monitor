use usage_model::{
    Amount, AmountUnit, ErrorCode, Fact, HarnessError, Notice, Severity, UsageWindow, WindowKind,
};

use super::*;

const NOW: i64 = 1_800_000_000;

fn window(id: &str, used: f64, resets_in_secs: i64) -> UsageWindow {
    let mut w = UsageWindow::new(id, WindowKind::FiveHour);
    w.used_pct = Some(used);
    w.resets_at = Some(NOW + resets_in_secs);
    w
}

fn ready(id: HarnessId, windows: Vec<UsageWindow>) -> HarnessView {
    let mut view = HarnessView::with_state(id, ViewState::Ready);
    view.windows = windows;
    view.read_at = Some(NOW - 30);
    view
}

fn failure(id: HarnessId, code: ErrorCode) -> HarnessView {
    HarnessView::error(id, HarnessError::new(code, true))
}

fn all_enabled() -> Hub {
    Hub::new(&Settings::default())
}

fn view_of(hub: &Hub, id: HarnessId) -> HarnessView {
    hub.views_at(NOW).swap_remove(id as usize)
}

#[test]
fn the_hub_starts_loading_for_enabled_harnesses_and_disabled_for_the_rest() {
    let mut settings = Settings::default();
    settings.codex.enabled = false;
    settings.litellm.enabled = false;
    let views = Hub::new(&settings).views_at(NOW);
    let states: Vec<_> = views.iter().map(|v| (v.id, v.state)).collect();
    assert_eq!(
        states,
        [
            (HarnessId::Claude, ViewState::Loading),
            (HarnessId::Codex, ViewState::Disabled),
            (HarnessId::Antigravity, ViewState::Loading),
            (HarnessId::Opencode, ViewState::Loading),
            (HarnessId::Kimi, ViewState::Loading),
            (HarnessId::Zai, ViewState::Loading),
            (HarnessId::Openrouter, ViewState::Loading),
            (HarnessId::Deepseek, ViewState::Loading),
            (HarnessId::Litellm, ViewState::Disabled),
        ]
    );
}

#[test]
fn harness_ids_index_the_views_in_tab_order() {
    for (index, id) in HarnessId::ALL.iter().enumerate() {
        assert_eq!(*id as usize, index);
    }
    let hub = all_enabled();
    assert!(hub.views_at(NOW).iter().zip(HarnessId::ALL).all(|(view, id)| view.id == id));
}

#[test]
fn reconcile_enabled_is_idempotent_and_resets_only_what_flipped() {
    let mut hub = all_enabled();
    hub.apply(ready(HarnessId::Claude, vec![window("five_hour", 10.0, 600)]));
    hub.apply(ready(HarnessId::Codex, vec![window("five_hour", 20.0, 600)]));
    let mut settings = Settings::default();
    settings.codex.enabled = false;
    hub.reconcile_enabled(&settings);
    hub.reconcile_enabled(&settings);
    assert_eq!(view_of(&hub, HarnessId::Codex), HarnessView::disabled(HarnessId::Codex));
    assert_eq!(view_of(&hub, HarnessId::Claude).windows.len(), 1, "untouched harness keeps data");
    settings.codex.enabled = true;
    hub.reconcile_enabled(&settings);
    assert_eq!(view_of(&hub, HarnessId::Codex), HarnessView::loading(HarnessId::Codex));
}

#[test]
fn a_failed_collection_keeps_the_last_good_windows_stale_with_the_error() {
    let mut hub = all_enabled();
    let good = ready(HarnessId::Claude, vec![window("five_hour", 40.0, 600)]);
    hub.apply(good.clone());
    hub.apply(failure(HarnessId::Claude, ErrorCode::Network));
    let shown = view_of(&hub, HarnessId::Claude);
    assert_eq!(shown.state, ViewState::Ready, "the tab must not be blanked");
    assert!(shown.stale);
    assert_eq!(shown.error.as_ref().map(|e| e.code), Some(ErrorCode::Network));
    assert_eq!(shown.windows, good.windows);
    assert_eq!(shown.read_at, good.read_at, "read_at still says when the data was produced");
}

#[test]
fn a_later_failure_replaces_the_error_but_keeps_the_same_good_windows() {
    let mut hub = all_enabled();
    let good = ready(HarnessId::Claude, vec![window("five_hour", 40.0, 600)]);
    hub.apply(good.clone());
    hub.apply(failure(HarnessId::Claude, ErrorCode::Network));
    hub.apply(failure(HarnessId::Claude, ErrorCode::Timeout));
    let shown = view_of(&hub, HarnessId::Claude);
    assert_eq!(shown.windows, good.windows);
    assert_eq!(shown.error.as_ref().map(|e| e.code), Some(ErrorCode::Timeout));
    assert!(shown.stale);
}

#[test]
fn a_success_after_a_failure_replaces_everything() {
    let mut hub = all_enabled();
    hub.apply(ready(HarnessId::Claude, vec![window("five_hour", 40.0, 600)]));
    hub.apply(failure(HarnessId::Claude, ErrorCode::Network));
    let fresh = ready(HarnessId::Claude, vec![window("five_hour", 55.0, 900)]);
    hub.apply(fresh.clone());
    assert_eq!(view_of(&hub, HarnessId::Claude), fresh);
}

#[test]
fn a_failure_with_nothing_to_keep_shows_the_error_state() {
    for before in [
        HarnessView::loading(HarnessId::Codex),
        HarnessView::with_state(HarnessId::Codex, ViewState::NoData),
        HarnessView::needs_setup(HarnessId::Codex),
        HarnessView::needs_consent(HarnessId::Codex),
        failure(HarnessId::Codex, ErrorCode::Network),
    ] {
        let mut hub = all_enabled();
        hub.apply(before);
        hub.apply(failure(HarnessId::Codex, ErrorCode::BinaryNotFound));
        let shown = view_of(&hub, HarnessId::Codex);
        assert_eq!(shown.state, ViewState::Error);
        assert_eq!(shown.error.map(|e| e.code), Some(ErrorCode::BinaryNotFound));
        assert!(!shown.stale);
    }
}

#[test]
fn states_that_are_not_failures_replace_the_view_and_drop_old_windows() {
    for replacement in [
        HarnessView::needs_consent(HarnessId::Codex),
        HarnessView::needs_setup(HarnessId::Codex),
        HarnessView::with_state(HarnessId::Codex, ViewState::NoData),
        HarnessView::disabled(HarnessId::Codex),
    ] {
        let mut hub = all_enabled();
        hub.apply(ready(HarnessId::Codex, vec![window("five_hour", 40.0, 600)]));
        hub.apply(replacement.clone());
        assert_eq!(view_of(&hub, HarnessId::Codex), replacement, "revoked consent must blank data");
    }
}

#[test]
fn results_land_on_their_own_tab_only() {
    let mut hub = all_enabled();
    hub.apply(ready(HarnessId::Opencode, vec![window("rolling", 5.0, 600)]));
    for id in [HarnessId::Claude, HarnessId::Codex, HarnessId::Antigravity, HarnessId::Litellm] {
        assert_eq!(view_of(&hub, id).state, ViewState::Loading, "{id:?}");
    }
}

#[test]
fn a_ready_view_with_every_window_expired_is_presented_stale() {
    let mut hub = all_enabled();
    hub.apply(ready(
        HarnessId::Claude,
        vec![window("five_hour", 90.0, 60), window("weekly", 50.0, 120)],
    ));
    assert!(!hub.views_at(NOW)[0].stale);
    assert!(!hub.views_at(NOW + 119)[0].stale, "one window is still current");
    assert!(hub.views_at(NOW + 120)[0].stale, "resets_at == now counts as expired");
    assert!(hub.views_at(NOW + 5_000)[0].stale);
}

#[test]
fn a_window_without_a_reset_time_keeps_the_view_fresh() {
    let mut open_ended = window("budget", 10.0, 0);
    open_ended.resets_at = None;
    let mut hub = all_enabled();
    hub.apply(ready(HarnessId::Litellm, vec![window("five_hour", 90.0, 60), open_ended]));
    assert!(!hub.views_at(NOW + 10_000)[4].stale);
}

#[test]
fn expiry_never_marks_views_without_windows_or_outside_the_ready_state() {
    let mut hub = all_enabled();
    hub.apply(ready(HarnessId::Claude, Vec::new()));
    hub.apply(HarnessView::with_state(HarnessId::Codex, ViewState::NoData));
    let later = hub.views_at(NOW + 1_000_000);
    assert!(later.iter().all(|view| !view.stale));
}

#[test]
fn expiry_is_derived_at_read_time_and_never_stored() {
    let mut hub = all_enabled();
    let good = ready(HarnessId::Claude, vec![window("five_hour", 40.0, 60)]);
    hub.apply(good.clone());
    assert!(hub.views_at(NOW + 61)[0].stale);
    assert_eq!(hub.views_at(NOW)[0], good, "an earlier read is unaffected");
}

#[test]
fn next_expiry_is_the_earliest_moment_a_view_becomes_stale() {
    let mut hub = all_enabled();
    assert_eq!(hub.next_expiry(NOW), None);
    hub.apply(ready(HarnessId::Claude, vec![window("a", 1.0, 100), window("b", 1.0, 300)]));
    hub.apply(ready(HarnessId::Codex, vec![window("a", 1.0, 200)]));
    assert_eq!(hub.next_expiry(NOW), Some(NOW + 200), "claude only goes stale at its last reset");
    assert_eq!(hub.next_expiry(NOW + 200), Some(NOW + 300), "past moments are skipped");
    assert_eq!(hub.next_expiry(NOW + 300), None);
}

#[test]
fn next_expiry_ignores_open_ended_stale_and_non_ready_views() {
    let mut hub = all_enabled();
    let mut open = window("a", 1.0, 100);
    open.resets_at = None;
    hub.apply(ready(HarnessId::Claude, vec![window("b", 1.0, 100), open]));
    hub.apply(HarnessView::needs_consent(HarnessId::Codex));
    let mut stale = ready(HarnessId::Opencode, vec![window("a", 1.0, 100)]);
    stale.stale = true;
    hub.apply(stale);
    assert_eq!(hub.next_expiry(NOW), None);
}

#[test]
fn next_expiry_survives_absurd_reset_times() {
    let mut hub = all_enabled();
    let mut far = window("a", 1.0, 0);
    far.resets_at = Some(i64::MAX);
    hub.apply(ready(HarnessId::Claude, vec![far]));
    assert_eq!(hub.next_expiry(NOW), Some(i64::MAX));
    assert_eq!(hub.next_expiry(i64::MIN), Some(i64::MAX));
}

#[test]
fn non_finite_numbers_become_unknown_and_snapshots_stay_comparable() {
    let mut poisoned = window("budget", f64::NAN, 600);
    poisoned.amount =
        Some(Amount { used: f64::INFINITY, limit: Some(10.0), unit: AmountUnit::Usd });
    let mut partial = window("other", 10.0, 600);
    partial.amount = Some(Amount { used: 1.0, limit: Some(f64::NAN), unit: AmountUnit::Usd });
    let mut view = ready(HarnessId::Litellm, vec![poisoned, partial]);
    view.facts =
        vec![Fact { id: "rpm".into(), number: Some(f64::NEG_INFINITY), text: None, unit: None }];
    view.notices =
        vec![Notice { code: "x".into(), severity: Severity::Info, params: Default::default() }];
    let mut hub = all_enabled();
    hub.apply(view);
    let shown = view_of(&hub, HarnessId::Litellm);
    assert_eq!(shown.windows[0].used_pct, None);
    assert_eq!(shown.windows[0].amount, None);
    assert_eq!(shown.windows[1].amount.as_ref().map(|a| (a.used, a.limit)), Some((1.0, None)));
    assert_eq!(shown.facts[0].number, None);
    assert_eq!(hub.views_at(NOW), hub.views_at(NOW), "a NaN would make this unequal");
}

#[test]
fn apply_reports_whether_the_windows_changed() {
    let mut hub = all_enabled();
    let id = HarnessId::Codex;
    assert!(hub.apply(ready(id, vec![window("five_hour", 10.0, 3600)])), "first reading");
    assert!(!hub.apply(ready(id, vec![window("five_hour", 10.0, 3600)])), "same windows");
    assert!(hub.apply(ready(id, vec![window("five_hour", 11.0, 3600)])), "new percentage");
    assert!(!hub.apply(failure(id, ErrorCode::Network)), "a failure keeps the windows");
}

#[test]
fn the_cool_down_mark_rides_on_the_stale_view_and_ends_with_the_next_result() {
    let mut hub = all_enabled();
    let id = HarnessId::Claude;
    hub.apply(ready(id, vec![window("five_hour", 10.0, 3600)]));
    hub.apply(failure(id, ErrorCode::RateLimited));
    hub.set_next_attempt(id, Some(NOW + 300));
    let shown = view_of(&hub, id);
    assert_eq!((shown.stale, shown.next_attempt_at), (true, Some(NOW + 300)));
    assert_eq!(shown.windows.len(), 1);

    hub.apply(failure(id, ErrorCode::Network));
    assert_eq!(view_of(&hub, id).next_attempt_at, None, "a later failure clears it");
    hub.set_next_attempt(id, Some(NOW + 300));
    hub.apply(ready(id, vec![window("five_hour", 12.0, 3600)]));
    assert_eq!(view_of(&hub, id).next_attempt_at, None, "so does a good read");
}
