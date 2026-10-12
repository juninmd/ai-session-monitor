//! Settings saved while the service runs take effect without a restart.

use usage_model::{ErrorCode, HarnessId, HarnessView, ViewState};

use crate::collector::{FakeCollector, Step};
use crate::rig::{rig, seconds, secs, settle};
use crate::views::{of, ready};

const CLAUDE: HarnessId = HarnessId::Claude;
const CODEX: HarnessId = HarnessId::Codex;

#[tokio::test(start_paused = true)]
async fn disabling_a_harness_stops_it_at_once_and_turns_its_view_off() {
    let rig = rig().start().await;
    rig.at(seconds(100)).await;
    assert_eq!(rig.fake(CODEX).count(), 2);

    rig.edit(|s| s.codex.enabled = false);
    let shown = rig.service.snapshot();
    assert_eq!(of(&shown.views, CODEX).state, ViewState::Disabled, "no waiting for the loop");
    assert!(of(&shown.views, CODEX).windows.is_empty(), "and its data is dropped");

    rig.at(seconds(3000)).await;
    assert_eq!(rig.fake(CODEX).count(), 2, "never read again");
    assert!(rig.fake(CLAUDE).count() > 40, "the rest carry on");
    let events = rig.sink.settings();
    assert_eq!(events.len(), 1);
    assert!(!events[0].codex.enabled);
}

#[tokio::test(start_paused = true)]
async fn enabling_a_harness_reads_it_straight_away() {
    let rig = rig().settings(|s| s.codex.enabled = false).start().await;
    rig.at(seconds(100)).await;
    assert_eq!(rig.fake(CODEX).count(), 0);

    rig.edit(|s| s.codex.enabled = true);
    assert_eq!(of(&rig.service.snapshot().views, CODEX).state, ViewState::Loading);
    settle().await;
    assert_eq!(rig.fake(CODEX).offsets_ms(rig.start), secs(&[100]));
    rig.at(seconds(161)).await;
    assert_eq!(rig.fake(CODEX).offsets_ms(rig.start), secs(&[100, 160]));
}

#[tokio::test(start_paused = true)]
async fn a_harness_switched_off_and_on_before_the_loop_looks_is_still_read_again() {
    let rig = rig().start().await;
    rig.at(seconds(5)).await;

    // No await between the saves, so the loop never sees the "off": only the tab shows it.
    rig.edit(|s| s.codex.enabled = false);
    rig.edit(|s| s.codex.enabled = true);
    assert_eq!(of(&rig.service.snapshot().views, CODEX).state, ViewState::Loading);
    settle().await;

    assert_eq!(rig.fake(CODEX).offsets_ms(rig.start), secs(&[0, 5]), "read again at once");
    assert_eq!(of(&rig.service.snapshot().views, CODEX).state, ViewState::Ready);
    rig.at(seconds(66)).await;
    assert_eq!(rig.fake(CODEX).offsets_ms(rig.start), secs(&[0, 5, 65]), "a full interval after");
}

#[tokio::test(start_paused = true)]
async fn a_parked_harness_switched_off_and_on_again_does_not_stay_loading_for_good() {
    let waiting = Step::View(HarnessView::needs_consent(CODEX));
    let rig = rig().collector(FakeCollector::always(CODEX, waiting)).start().await;
    rig.at(seconds(5)).await;
    assert_eq!(of(&rig.service.snapshot().views, CODEX).state, ViewState::NeedsConsent);

    rig.edit(|s| s.codex.enabled = false);
    rig.edit(|s| s.codex.enabled = true);
    settle().await;

    assert_eq!(rig.fake(CODEX).count(), 2, "nothing else would ever look at it again");
    assert_eq!(of(&rig.service.snapshot().views, CODEX).state, ViewState::NeedsConsent);
}

#[tokio::test(start_paused = true)]
async fn the_answer_of_a_read_cancelled_by_disabling_is_never_shown() {
    let slow = Step::After(seconds(10), ready(CODEX, 10.0));
    let rig = rig().collector(FakeCollector::always(CODEX, slow)).start().await;
    rig.at(seconds(5)).await;
    assert_eq!(rig.fake(CODEX).in_flight(), 1);

    rig.edit(|s| s.codex.enabled = false);
    settle().await;
    assert_eq!(rig.fake(CODEX).in_flight(), 0, "the read was cancelled");
    rig.at(seconds(30)).await;
    assert_eq!(of(&rig.service.snapshot().views, CODEX).state, ViewState::Disabled);
    assert_eq!(rig.fake(CODEX).count(), 1);
}

#[tokio::test(start_paused = true)]
async fn a_shorter_interval_re_times_the_next_read() {
    let rig = rig().settings(|s| s.general.interval_secs = 600).start().await;
    rig.at(seconds(100)).await;

    rig.edit(|s| s.general.interval_secs = 30);
    rig.at(seconds(131)).await;
    // Already overdue under the new interval: read now, then every 30 s.
    assert_eq!(rig.fake(CODEX).offsets_ms(rig.start), secs(&[0, 100, 130]));
}

#[tokio::test(start_paused = true)]
async fn a_longer_interval_postpones_the_next_read() {
    let rig = rig().start().await;
    rig.at(seconds(70)).await;

    rig.edit(|s| s.general.interval_secs = 3600);
    rig.at(seconds(3000)).await;
    assert_eq!(rig.fake(CODEX).offsets_ms(rig.start), secs(&[0, 60]));
    rig.at(seconds(3661)).await;
    assert_eq!(rig.fake(CODEX).offsets_ms(rig.start), secs(&[0, 60, 3660]));
}

#[tokio::test(start_paused = true)]
async fn an_interval_change_does_not_cut_a_backoff_short() {
    let error = Step::View(crate::views::failing(CODEX, ErrorCode::Network, true));
    let rig = rig().collector(FakeCollector::always(CODEX, error)).start().await;
    rig.at(seconds(10)).await;

    rig.edit(|s| s.general.interval_secs = 30);
    rig.at(seconds(130)).await;
    assert_eq!(rig.fake(CODEX).offsets_ms(rig.start), secs(&[0, 120]));
}

#[tokio::test(start_paused = true)]
async fn a_changed_section_reads_that_harness_again_with_the_new_settings() {
    let rig = rig().start().await;
    rig.at(seconds(5)).await;

    rig.edit(|s| s.codex.consent = true);
    settle().await;
    let calls = rig.fake(CODEX).calls();
    assert_eq!(calls.len(), 2);
    assert!(!calls[0].settings.codex.consent && calls[1].settings.codex.consent);
    for id in [CLAUDE, HarnessId::Antigravity, HarnessId::Opencode, HarnessId::Litellm] {
        assert_eq!(rig.fake(id).count(), 1, "{id:?} reads nothing from the codex section");
    }
    rig.at(seconds(66)).await;
    assert_eq!(rig.fake(CODEX).offsets_ms(rig.start), secs(&[0, 5, 65]), "a full interval after");
}

#[tokio::test(start_paused = true)]
async fn a_read_in_flight_with_outdated_settings_is_replaced_and_its_answer_dropped() {
    let plan = |n: usize| Step::After(seconds(20), ready(CODEX, if n == 0 { 11.0 } else { 22.0 }));
    let rig = rig().collector(FakeCollector::new(CODEX, plan)).start().await;
    rig.at(seconds(5)).await;

    rig.edit(|s| s.codex.consent = true);
    settle().await;
    assert_eq!((rig.fake(CODEX).count(), rig.fake(CODEX).in_flight()), (2, 1));
    rig.at(seconds(21)).await;
    assert_eq!(of(&rig.service.snapshot().views, CODEX).state, ViewState::Loading);
    rig.at(seconds(26)).await;
    let shown = rig.service.snapshot();
    assert_eq!(of(&shown.views, CODEX).windows[0].used_pct, Some(22.0));
    let ever_showed_old = rig.sink.snapshots().iter().any(|(_, s)| {
        of(&s.views, CODEX).windows.first().is_some_and(|w| w.used_pct == Some(11.0))
    });
    assert!(!ever_showed_old);
}

#[tokio::test(start_paused = true)]
async fn a_change_outside_every_harness_section_reads_nothing() {
    let rig = rig().start().await;
    rig.at(seconds(5)).await;

    rig.edit(|s| {
        s.general.notify_on_out = true;
        s.general.warn_at_pct = 40;
    });
    settle().await;
    for id in HarnessId::ALL {
        assert_eq!(rig.fake(id).count(), 1, "{id:?}");
    }
    assert_eq!(rig.sink.settings().len(), 1, "but it is announced");
}

#[tokio::test(start_paused = true)]
async fn a_rejected_save_changes_nothing() {
    let rig = rig().start().await;
    rig.at(seconds(5)).await;
    let before = rig.service.settings();

    let mut bad = before.clone();
    bad.litellm.base_url = Some("http://example.com/v1".to_owned());
    bad.codex.enabled = false;
    let error = rig.service.save_settings(bad).expect_err("plain http to a remote host");
    settle().await;

    assert_eq!(error.code, ErrorCode::InvalidUrl);
    assert_eq!(error.params.get("field").map(String::as_str), Some("baseUrl"));
    assert_eq!(rig.service.settings(), before);
    assert!(!rig.settings_file().exists(), "nothing was written");
    assert!(rig.sink.settings().is_empty(), "nothing was announced");
    assert_ne!(of(&rig.service.snapshot().views, CODEX).state, ViewState::Disabled);
    assert_eq!(rig.fake(CODEX).count(), 1);
}

#[tokio::test(start_paused = true)]
async fn a_save_returns_persists_and_announces_the_same_normalized_settings() {
    let rig = rig().build();
    let mut wild = rig.service.settings();
    wild.general.interval_secs = 1;
    wild.general.warn_at_pct = 99;
    wild.litellm.poll_interval_secs = 5;
    wild.litellm.timeout_secs = 999;

    let saved = rig.service.save_settings(wild).expect("valid once clamped");
    assert_eq!(
        (saved.general.interval_secs, saved.general.warn_at_pct),
        (30, 90),
        "clamped, not rejected"
    );
    assert_eq!((saved.litellm.poll_interval_secs, saved.litellm.timeout_secs), (60, 30));
    assert_eq!(rig.service.settings(), saved);
    assert_eq!(rig.saved_settings(), saved);
    assert_eq!(rig.sink.settings(), [saved]);
}
