//! A ready view goes stale on its own when every window has reset; nothing polls for that.

use usage_model::HarnessId;

use crate::collector::{FakeCollector, Step};
use crate::rig::{RigBuilder, rig, seconds};
use crate::views::{BASE, of, ready, ready_resetting};

const CLAUDE: HarnessId = HarnessId::Claude;

fn slow_polling(step: Step) -> RigBuilder {
    rig()
        .settings(|s| s.general.interval_secs = 3600)
        .collector(FakeCollector::always(CLAUDE, step))
}

#[tokio::test(start_paused = true)]
async fn a_view_whose_windows_all_reset_turns_stale_then_without_another_read() {
    let rig = slow_polling(Step::View(ready_resetting(CLAUDE, 40.0, 100))).start().await;

    rig.at(seconds(99)).await;
    assert!(!of(&rig.service.snapshot().views, CLAUDE).stale);
    rig.at(seconds(100)).await;
    assert!(of(&rig.service.snapshot().views, CLAUDE).stale, "the window reset");
    rig.at(seconds(102)).await;
    assert!(of(&rig.sink.last_snapshot().views, CLAUDE).stale, "and the UI was told");
    assert_eq!(rig.fake(CLAUDE).count(), 1, "without reading again");
}

#[tokio::test(start_paused = true)]
async fn one_window_still_running_keeps_the_view_fresh() {
    let mut view = ready_resetting(CLAUDE, 40.0, 100);
    let mut weekly = crate::views::window(10.0, 5000);
    weekly.id = "weekly".to_owned();
    view.windows.push(weekly);
    let rig = slow_polling(Step::View(view)).start().await;
    rig.at(seconds(200)).await;

    assert!(!of(&rig.service.snapshot().views, CLAUDE).stale);
    assert_eq!(rig.sink.snapshots().len(), 2, "no event: nothing changed for the UI");
}

#[tokio::test(start_paused = true)]
async fn a_far_future_reset_time_is_harmless() {
    let mut view = ready(CLAUDE, 10.0);
    view.windows[0].resets_at = Some(i64::MAX);
    let rig = slow_polling(Step::View(view)).start().await;
    rig.at(seconds(259_100)).await;

    assert_eq!(rig.fake(CLAUDE).count(), 72, "three days of hourly reads, the loop never stalled");
    assert!(!of(&rig.service.snapshot().views, CLAUDE).stale);
    assert_eq!(rig.sink.snapshots().len(), 2, "and the idle wake-ups said nothing");
}

#[tokio::test(start_paused = true)]
async fn a_window_already_over_when_read_is_stale_at_once_and_causes_no_spin() {
    let over = ready_resetting(CLAUDE, 40.0, -10);
    assert!(over.windows[0].resets_at.is_some_and(|at| at < BASE));
    let rig = rig().collector(FakeCollector::always(CLAUDE, Step::View(over))).start().await;
    rig.at(seconds(599)).await;

    assert!(of(&rig.service.snapshot().views, CLAUDE).stale);
    assert_eq!(rig.fake(CLAUDE).count(), 10, "the schedule is untouched");
    assert_eq!(rig.sink.snapshots().len(), 2);
}
