//! Adaptive cadence through the scheduler: a calm source is read less often, a busy one not.

use usage_model::{ErrorCode, HarnessId};

use crate::collector::{FakeCollector, Step};
use crate::rig::{RigBuilder, rig, seconds, secs, settle};
use crate::views::{failing, ready, ready_resetting};

const CLAUDE: HarnessId = HarnessId::Claude;

fn adaptive(claude: Step) -> RigBuilder {
    rig().settings(|s| s.general.adaptive = true).collector(FakeCollector::always(CLAUDE, claude))
}

#[tokio::test(start_paused = true)]
async fn a_calm_far_from_reset_source_is_read_less_and_less_often() {
    let rig = adaptive(Step::View(ready(CLAUDE, 10.0))).start().await;
    rig.at(seconds(560)).await;

    // 60 s base: the first read waits 2x, every unchanged read one step more.
    assert_eq!(rig.fake(CLAUDE).offsets_ms(rig.start), secs(&[0, 120, 300, 540]));
}

#[tokio::test(start_paused = true)]
async fn a_changing_reading_does_not_stretch_beyond_two_steps() {
    let plan = |n: usize| Step::View(ready(CLAUDE, 10.0 + n as f64));
    let rig = rig()
        .settings(|s| s.general.adaptive = true)
        .collector(FakeCollector::new(CLAUDE, plan))
        .start()
        .await;
    rig.at(seconds(370)).await;

    assert_eq!(rig.fake(CLAUDE).offsets_ms(rig.start), secs(&[0, 120, 240, 360]));
}

#[tokio::test(start_paused = true)]
async fn a_busy_window_or_a_near_reset_keeps_the_base_cadence() {
    let hot = adaptive(Step::View(ready(CLAUDE, 72.0))).start().await;
    hot.at(seconds(200)).await;
    assert_eq!(hot.fake(CLAUDE).offsets_ms(hot.start), secs(&[0, 60, 120, 180]));

    let near = adaptive(Step::View(ready_resetting(CLAUDE, 5.0, 500))).start().await;
    near.at(seconds(200)).await;
    assert_eq!(near.fake(CLAUDE).offsets_ms(near.start), secs(&[0, 60, 120, 180]));
}

#[tokio::test(start_paused = true)]
async fn the_switch_off_keeps_the_fixed_interval() {
    let calm = FakeCollector::always(CLAUDE, Step::View(ready(CLAUDE, 10.0)));
    let rig = rig().collector(calm).start().await;
    rig.at(seconds(200)).await;

    assert_eq!(rig.fake(CLAUDE).offsets_ms(rig.start), secs(&[0, 60, 120, 180]));
}

#[tokio::test(start_paused = true)]
async fn turning_it_off_brings_the_pending_read_back_to_the_base_cadence() {
    let rig = adaptive(Step::View(ready(CLAUDE, 10.0))).start().await;
    rig.at(seconds(10)).await;
    assert_eq!(rig.fake(CLAUDE).count(), 1, "the next read is stretched to 120 s");

    rig.edit(|s| s.general.adaptive = false);
    rig.at(seconds(61)).await;
    assert_eq!(rig.fake(CLAUDE).offsets_ms(rig.start), secs(&[0, 60]));
}

#[tokio::test(start_paused = true)]
async fn the_remote_floor_and_a_manual_refresh_are_untouched() {
    let id = HarnessId::Opencode;
    let rig = rig()
        .settings(|s| s.general.adaptive = true)
        .collector(FakeCollector::always(id, Step::View(ready(id, 5.0))))
        .collector(FakeCollector::always(CLAUDE, Step::View(ready(CLAUDE, 10.0))))
        .start()
        .await;
    rig.at(seconds(700)).await;
    assert_eq!(rig.fake(id).offsets_ms(rig.start), secs(&[0, 600]));

    let before = rig.fake(CLAUDE).count();
    rig.service.refresh(Some(CLAUDE));
    settle().await;
    assert_eq!(rig.fake(CLAUDE).count(), before + 1, "opening the flyout still reads at once");
}

#[tokio::test(start_paused = true)]
async fn a_failure_after_a_stretch_backs_off_from_the_base_not_the_stretched_wait() {
    let steps = vec![Step::View(ready(CLAUDE, 10.0)), Step::View(ready(CLAUDE, 10.0))];
    let fail = Step::View(failing(CLAUDE, ErrorCode::Network, true));
    let rig = rig()
        .settings(|s| s.general.adaptive = true)
        .collector(FakeCollector::scripted(CLAUDE, steps, fail))
        .start()
        .await;
    rig.at(seconds(500)).await;

    // Reads at 0 and 120 are good (waits 120, 180); the failure at 300 backs off 120 s from 60.
    assert_eq!(rig.fake(CLAUDE).offsets_ms(rig.start), secs(&[0, 120, 300, 420]));
}
