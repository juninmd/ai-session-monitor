//! A window that ran out resets by the clock even when no reading says so: one toast at its
//! `resetsAt`, and never a second one when a reading finally confirms it.
//! Except LiteLLM, whose proxy applies resets late: only the poll that confirms one announces it.

use std::sync::Arc;

use app_core::NotificationKind;
use usage_model::{ErrorCode, HarnessId, UsageWindow, WindowKind};

use crate::collector::{FakeCollector, Step};
use crate::rig::{Rig, RigBuilder, rig, seconds};
use crate::views::{failing, ready_resetting, window};

const CODEX: HarnessId = HarnessId::Codex;
const LITELLM: HarnessId = HarnessId::Litellm;
/// Off the poll grid on purpose: a toast at this second can only come from the reset itself.
const RESET_IN: i64 = 610;
const LAPSE_TEXT: &str = "A janela de 5 horas reiniciou e voltou a ter limite disponível.";

fn out_until(secs: i64) -> Step {
    Step::View(ready_resetting(CODEX, 100.0, secs))
}

fn free_until(secs: i64) -> Step {
    Step::View(ready_resetting(CODEX, 5.0, secs))
}

/// Polls hourly, so nothing but the reset itself can wake the loop around `RESET_IN`.
fn watching(codex: Arc<FakeCollector>) -> RigBuilder {
    rig()
        .settings(|s| {
            s.general.interval_secs = 3600;
            s.general.notify_on_reset = true;
        })
        .collector(codex)
}

fn script(steps: Vec<Step>) -> Arc<FakeCollector> {
    let last = steps.last().cloned().unwrap_or_else(|| out_until(RESET_IN));
    FakeCollector::scripted(CODEX, steps, last)
}

fn kinds(rig: &Rig) -> Vec<NotificationKind> {
    rig.sink.notices().iter().map(|notice| notice.kind).collect()
}

/// A LiteLLM budget as its collector reports it: after `resets_at` it is flagged stale.
fn budget(used: f64, resets_in: i64, stale: bool) -> Step {
    let mut view = ready_resetting(LITELLM, used, resets_in);
    view.windows[0].id = "budget".to_owned();
    view.windows[0].kind = WindowKind::Budget;
    view.stale = stale;
    Step::View(view)
}

/// Adds LiteLLM, polled every 300 s and out until `RESET_IN`. Its proxy sweeps resets late, so the
/// poll at 900 s still shows the old period and only the one at 1200 s shows the new one.
fn with_lagging_litellm(builder: RigBuilder) -> RigBuilder {
    let litellm = FakeCollector::new(LITELLM, |call| match call {
        0..=2 => budget(100.0, RESET_IN, false),
        3 => budget(100.0, RESET_IN, true),
        _ => budget(5.0, RESET_IN + 86_400, false),
    });
    builder.settings(|s| s.litellm.poll_interval_secs = 300).collector(litellm)
}

#[tokio::test(start_paused = true)]
async fn a_window_that_ran_out_is_announced_once_when_its_reset_arrives() {
    let rig = watching(FakeCollector::always(CODEX, out_until(RESET_IN))).start().await;

    rig.at(seconds(609)).await;
    assert!(rig.sink.notices().is_empty(), "one second early: the window is still out");
    rig.at(seconds(611)).await;
    let notices = rig.sink.notices();
    assert_eq!(notices.len(), 1);
    assert_eq!((notices[0].kind, notices[0].harness), (NotificationKind::WindowReset, CODEX));
    assert_eq!(notices[0].window_id, "five_hour");
    assert_eq!(notices[0].title, "Codex: limite disponível");
    assert_eq!(notices[0].body, LAPSE_TEXT, "no percentage: nobody read one");
    assert_eq!(rig.fake(CODEX).count(), 1, "and nothing was read to find out");

    rig.at(seconds(3 * 3600 + 5)).await;
    assert_eq!(rig.fake(CODEX).count(), 4, "the source kept repeating the old reading");
    assert_eq!(rig.sink.notices().len(), 1, "told once, however long the source stays silent");
}

#[tokio::test(start_paused = true)]
async fn the_toast_follows_the_snapshot_that_shows_the_window_expired() {
    let rig = watching(FakeCollector::always(CODEX, out_until(RESET_IN))).start().await;
    rig.at(seconds(611)).await;

    let kinds = rig.sink.kinds();
    assert_eq!(kinds.last(), Some(&"notice"));
    assert_eq!(kinds[kinds.len() - 2], "snapshot", "the UI shows the new state before the toast");
}

#[tokio::test(start_paused = true)]
async fn a_source_that_fails_is_just_as_silent() {
    let steps = vec![out_until(RESET_IN), Step::View(failing(CODEX, ErrorCode::Network, true))];
    let rig =
        rig().settings(|s| s.general.notify_on_reset = true).collector(script(steps)).start().await;

    rig.at(seconds(609)).await;
    assert!(rig.sink.notices().is_empty());
    assert!(rig.fake(CODEX).count() > 1, "it kept failing meanwhile");
    rig.at(seconds(611)).await;
    assert_eq!(kinds(&rig), [NotificationKind::WindowReset]);
}

#[tokio::test(start_paused = true)]
async fn a_reading_that_confirms_the_reset_later_does_not_notify_a_second_time() {
    let rig = watching(script(vec![out_until(RESET_IN), free_until(3600 + 18_000)])).start().await;

    rig.at(seconds(611)).await;
    assert_eq!(rig.sink.notices().len(), 1);
    rig.at(seconds(3700)).await;
    assert_eq!(rig.fake(CODEX).count(), 2, "the confirming reading arrived");
    let shown = rig.service.snapshot();
    assert_eq!(crate::views::of(&shown.views, CODEX).windows[0].used_pct, Some(5.0));
    let notices = rig.sink.notices();
    assert_eq!(notices.len(), 1, "the clock already said it: {notices:?}");
    assert_eq!(notices[0].body, LAPSE_TEXT);
}

#[tokio::test(start_paused = true)]
async fn the_next_cycle_is_told_again_in_its_own_right() {
    // Out until 610, back at the hourly read, out again until 8200 and silent after that.
    let steps = vec![out_until(RESET_IN), free_until(3600 + 18_000), out_until(7200 + 1000)];
    let rig = watching(script(steps)).settings(|s| s.general.notify_on_out = true).start().await;

    rig.at(seconds(7300)).await;
    assert_eq!(rig.fake(CODEX).count(), 3);
    assert_eq!(
        kinds(&rig),
        [NotificationKind::WindowReset, NotificationKind::WindowOut],
        "the first reset by the clock, then the window running out again"
    );
    rig.at(seconds(8201)).await;
    assert_eq!(
        kinds(&rig),
        [NotificationKind::WindowReset, NotificationKind::WindowOut, NotificationKind::WindowReset],
        "and its reset by the clock too, a different reset than the one already told"
    );
}

#[tokio::test(start_paused = true)]
async fn a_window_out_again_with_a_new_reset_time_is_announced_at_that_time() {
    let steps = vec![out_until(RESET_IN), out_until(3600 + 1000)];
    let rig = watching(script(steps)).start().await;

    rig.at(seconds(3700)).await;
    assert_eq!(kinds(&rig), [NotificationKind::WindowReset], "the first reset, told at 610");
    rig.at(seconds(4601)).await;
    assert_eq!(kinds(&rig).len(), 2, "the second window's reset");
    rig.at(seconds(7300)).await;
    assert_eq!(kinds(&rig).len(), 2, "and the repeats of that reading add nothing");
}

#[tokio::test(start_paused = true)]
async fn a_reading_that_confirms_the_reset_before_the_clock_does_is_the_only_toast() {
    let steps = vec![out_until(RESET_IN), free_until(60 + 18_000)];
    let rig =
        rig().settings(|s| s.general.notify_on_reset = true).collector(script(steps)).start().await;

    rig.at(seconds(61)).await;
    assert_eq!(kinds(&rig), [NotificationKind::WindowReset], "the provider reset it early");
    assert!(rig.sink.notices()[0].body.contains("% livre"), "a real reading, with its number");
    rig.at(seconds(700)).await;
    assert_eq!(rig.sink.notices().len(), 1, "the old reset time passing is no news");
}

#[tokio::test(start_paused = true)]
async fn nothing_is_announced_unless_the_user_asked_for_resets() {
    let rig = rig()
        .settings(|s| s.general.interval_secs = 3600)
        .collector(FakeCollector::always(CODEX, out_until(RESET_IN)))
        .start()
        .await;
    rig.at(seconds(700)).await;

    assert!(rig.sink.notices().is_empty());
}

#[tokio::test(start_paused = true)]
async fn a_window_that_was_not_out_has_nothing_to_announce() {
    let rig = watching(FakeCollector::always(CODEX, free_until(RESET_IN))).start().await;
    rig.at(seconds(700)).await;

    assert!(rig.sink.notices().is_empty(), "it never ran out, so nothing came back");
}

#[tokio::test(start_paused = true)]
async fn a_harness_switched_off_before_its_reset_stays_quiet() {
    let rig = watching(FakeCollector::always(CODEX, out_until(RESET_IN))).start().await;
    rig.at(seconds(100)).await;
    rig.edit(|s| s.codex.enabled = false);
    rig.at(seconds(700)).await;

    assert!(rig.sink.notices().is_empty());
}

#[tokio::test(start_paused = true)]
async fn switching_the_toggle_on_while_waiting_arms_the_reset() {
    let rig = rig()
        .settings(|s| s.general.interval_secs = 3600)
        .collector(FakeCollector::always(CODEX, out_until(RESET_IN)))
        .start()
        .await;
    rig.at(seconds(100)).await;
    rig.edit(|s| s.general.notify_on_reset = true);

    rig.at(seconds(609)).await;
    assert!(rig.sink.notices().is_empty());
    rig.at(seconds(611)).await;
    assert_eq!(kinds(&rig), [NotificationKind::WindowReset], "the wake-up was re-armed");
}

#[tokio::test(start_paused = true)]
async fn each_window_is_announced_at_its_own_reset_even_with_the_view_still_fresh() {
    let mut view = ready_resetting(CODEX, 100.0, RESET_IN);
    let mut weekly: UsageWindow = window(100.0, 1230);
    weekly.id = "weekly".to_owned();
    weekly.kind = WindowKind::Weekly;
    view.windows.push(weekly);
    let rig = watching(FakeCollector::always(CODEX, Step::View(view))).start().await;

    rig.at(seconds(611)).await;
    let notices = rig.sink.notices();
    assert_eq!(notices.iter().map(|n| n.window_id.as_str()).collect::<Vec<_>>(), ["five_hour"]);
    assert_eq!(rig.sink.snapshots().len(), 2, "the view is still fresh: the toast is the news");
    rig.at(seconds(1231)).await;
    let notices = rig.sink.notices();
    let ids: Vec<_> = notices.iter().map(|n| n.window_id.as_str()).collect();
    assert_eq!(ids, ["five_hour", "weekly"]);
    assert_eq!(notices[1].body, "A janela semanal reiniciou e voltou a ter limite disponível.");
}

#[tokio::test(start_paused = true)]
async fn a_reset_days_away_is_announced_on_time_and_idle_wakeups_say_nothing() {
    // Three days and ten seconds: the loop re-arms at its one-day cap twice before the reset.
    let rig = watching(FakeCollector::always(CODEX, out_until(259_210))).start().await;

    rig.at(seconds(259_209)).await;
    assert!(rig.sink.notices().is_empty());
    rig.at(seconds(259_211)).await;
    assert_eq!(kinds(&rig), [NotificationKind::WindowReset]);
}

#[tokio::test(start_paused = true)]
async fn an_absurd_reset_time_neither_stalls_nor_announces() {
    let mut view = ready_resetting(CODEX, 100.0, 0);
    view.windows[0].resets_at = Some(i64::MAX);
    let rig = watching(FakeCollector::always(CODEX, Step::View(view))).start().await;
    rig.at(seconds(2 * 3600 + 5)).await;

    assert_eq!(rig.fake(CODEX).count(), 3, "the loop keeps its schedule");
    assert!(rig.sink.notices().is_empty());
}

#[tokio::test(start_paused = true)]
async fn a_source_that_lags_its_reset_is_announced_by_the_poll_that_confirms_it() {
    let notifying = rig().settings(|s| s.general.notify_on_reset = true);
    let rig = with_lagging_litellm(notifying).start().await;

    rig.at(seconds(611)).await;
    assert!(rig.sink.notices().is_empty(), "the proxy has not applied it yet: nothing to tell");
    rig.at(seconds(901)).await;
    assert_eq!(rig.fake(LITELLM).count(), 4, "the poll after the reset ran");
    assert!(rig.sink.notices().is_empty(), "and it still showed the old period");

    rig.at(seconds(1201)).await;
    let notices = rig.sink.notices();
    assert_eq!(notices.len(), 1, "one toast, from the new period's first reading: {notices:?}");
    assert_eq!((notices[0].kind, notices[0].harness), (NotificationKind::WindowReset, LITELLM));
    assert_eq!(notices[0].window_id, "budget");
    assert!(notices[0].body.contains("voltou a ter limite disponível (95% livre)"), "{notices:?}");
    rig.at(seconds(3601)).await;
    assert_eq!(rig.sink.notices().len(), 1, "later polls repeat the new period: no second toast");
}

#[tokio::test(start_paused = true)]
async fn a_source_that_does_not_lag_is_still_announced_by_the_clock_beside_one_that_does() {
    let codex = watching(FakeCollector::always(CODEX, out_until(RESET_IN)));
    let rig = with_lagging_litellm(codex).start().await;

    rig.at(seconds(611)).await;
    let notices = rig.sink.notices();
    assert_eq!(notices.len(), 1, "only Codex is announced by the clock: {notices:?}");
    assert_eq!((notices[0].harness, notices[0].body.as_str()), (CODEX, LAPSE_TEXT));
    rig.at(seconds(1201)).await;
    let told: Vec<_> = rig.sink.notices().iter().map(|notice| notice.harness).collect();
    assert_eq!(told, [CODEX, LITELLM], "LiteLLM follows with the poll that confirms its reset");
}
