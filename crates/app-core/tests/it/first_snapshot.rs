//! What the first snapshot looks like, and which harnesses are ever collected.

use std::time::Duration;

use usage_model::{HarnessId, ViewState};

use crate::collector::{FakeCollector, Step};
use crate::rig::{rig, seconds, settle};
use crate::views::{BASE, of};

const MILLIS_300: Duration = Duration::from_millis(300);

#[tokio::test(start_paused = true)]
async fn the_first_snapshot_has_enabled_harnesses_loading_and_the_disabled_one_off() {
    let rig = rig().settings(|s| s.litellm.enabled = false).start().await;

    let snapshots = rig.sink.snapshots();
    assert_eq!(snapshots.len(), 1, "one initial event, before any collector's answer is out");
    let first = &snapshots[0].1;
    assert_eq!(first.generated_at, BASE);
    let states: Vec<_> = first.views.iter().map(|view| (view.id, view.state)).collect();
    assert_eq!(
        states,
        [
            (HarnessId::Claude, ViewState::Loading),
            (HarnessId::Codex, ViewState::Loading),
            (HarnessId::Antigravity, ViewState::Loading),
            (HarnessId::Opencode, ViewState::Loading),
            (HarnessId::Litellm, ViewState::Disabled),
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn the_answers_arrive_together_in_the_second_event() {
    let rig = rig().start().await;
    rig.after(MILLIS_300).await;

    let snapshots = rig.sink.snapshots();
    assert_eq!(snapshots.len(), 2);
    assert!(snapshots[1].1.views.iter().all(|view| view.state == ViewState::Ready));
    assert_eq!(rig.service.snapshot().views, snapshots[1].1.views, "get_snapshot agrees");
}

#[tokio::test(start_paused = true)]
async fn get_snapshot_answers_before_run_and_never_waits_for_a_collector() {
    let mut rig = rig().collector(FakeCollector::always(HarnessId::Claude, Step::Hang)).build();

    let before_run = rig.service.snapshot();
    assert_eq!(before_run.generated_at, BASE);
    assert!(before_run.views.iter().all(|view| view.state == ViewState::Loading));

    rig.run().await;
    rig.after(seconds(10)).await;
    assert_eq!(of(&rig.service.snapshot().views, HarnessId::Claude).state, ViewState::Loading);
    assert_eq!(rig.fake(HarnessId::Claude).in_flight(), 1, "still reading, and nobody waits");
}

#[tokio::test(start_paused = true)]
async fn a_disabled_harness_is_never_collected() {
    let rig = rig()
        .settings(|s| {
            s.codex.enabled = false;
            s.opencode.enabled = false;
        })
        .start()
        .await;
    rig.after(seconds(3600)).await;

    for id in [HarnessId::Codex, HarnessId::Opencode] {
        assert_eq!(rig.fake(id).count(), 0, "{id:?}");
        assert_eq!(of(&rig.service.snapshot().views, id).state, ViewState::Disabled);
    }
    assert!(rig.fake(HarnessId::Claude).count() > 1, "the enabled ones keep polling");

    rig.service.refresh(Some(HarnessId::Codex));
    rig.service.refresh(None);
    settle().await;
    assert_eq!(rig.fake(HarnessId::Codex).count(), 0, "not even when asked");
}

#[tokio::test(start_paused = true)]
async fn a_harness_without_a_collector_reads_as_no_data_and_is_not_polled() {
    let rig = rig().without_collector(HarnessId::Litellm).start().await;
    rig.after(seconds(599)).await;

    assert_eq!(of(&rig.service.snapshot().views, HarnessId::Litellm).state, ViewState::NoData);
    assert_eq!(rig.fake(HarnessId::Claude).count(), 10, "the others are unaffected");
}

#[tokio::test(start_paused = true)]
async fn collectors_get_the_clock_the_directories_and_the_current_settings() {
    let rig = rig().settings(|s| s.general.interval_secs = 90).start().await;
    rig.at(seconds(91)).await;

    let calls = rig.fake(HarnessId::Codex).calls();
    assert_eq!(calls.len(), 2);
    assert_eq!((calls[0].now_secs, calls[1].now_secs), (BASE, BASE + 90));
    assert_eq!(calls[0].data_dir, rig.data_dir);
    assert_eq!(calls[0].home_dir, rig.home_dir);
    assert_eq!(calls[1].settings.general.interval_secs, 90);
}
