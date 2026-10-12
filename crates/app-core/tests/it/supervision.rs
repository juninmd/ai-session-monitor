//! A panic in the scheduler task ends `run`. The shell supervises that task, so the next `run`
//! must be able to start polling again, and two runs at once must still make one scheduler.

use std::sync::Arc;

use app_core::NotificationKind;
use usage_model::HarnessId;

use crate::rig::{rig, seconds, settle};
use crate::sink::RecordingSink;

const GARBAGE: &[u8] = b"{ not json";
/// A minute in: the three local sources read twice, OpenCode and LiteLLM (300 s) once.
const LOCAL_TWICE: [usize; 5] = [2, 2, 2, 1, 1];

fn reads(rig: &crate::rig::Rig) -> Vec<usize> {
    HarnessId::ALL.iter().map(|&id| rig.fake(id).count()).collect()
}

#[tokio::test(start_paused = true)]
async fn run_can_be_started_again_after_a_sink_panicked_inside_the_loop() {
    let mut rig = rig().sink(RecordingSink::panicking_in_snapshots(1)).build();

    rig.run().await;
    let died = rig.take_runner().await;
    assert!(died.is_err_and(|error| error.is_panic()), "the first delivery took the loop down");
    assert_eq!(reads(&rig), [0; 5], "and it died before reading anything");

    rig.run().await;
    rig.after(seconds(1)).await;
    assert!(!rig.runner_finished(), "the new loop keeps running");
    assert_eq!(reads(&rig), [1; 5], "and polls again");
    assert!(!rig.sink.snapshots().is_empty(), "with the sink hearing about it");

    rig.after(seconds(60)).await;
    assert_eq!(reads(&rig), LOCAL_TWICE, "on the normal schedule");
}

#[tokio::test(start_paused = true)]
async fn only_one_run_polls_at_a_time_even_after_a_restart() {
    let mut rig = rig().sink(RecordingSink::panicking_in_snapshots(1)).build();
    rig.run().await;
    let _ = rig.take_runner().await;
    rig.run().await;

    let third = tokio::spawn(Arc::clone(&rig.service).run());
    settle().await;

    assert!(third.is_finished(), "the claim is held by the live loop, so this one returns at once");
    assert_eq!(reads(&rig), [1; 5], "and did not poll");
}

#[tokio::test(start_paused = true)]
async fn two_runs_started_together_make_one_scheduler() {
    let rig = rig().build();
    let first = tokio::spawn(Arc::clone(&rig.service).run());
    let second = tokio::spawn(Arc::clone(&rig.service).run());
    settle().await;

    let done = [first.is_finished(), second.is_finished()];
    assert_eq!(done.iter().filter(|done| **done).count(), 1, "the loser returned at once");
    assert_eq!(reads(&rig), [1; 5]);
    rig.after(seconds(60)).await;
    assert_eq!(reads(&rig), LOCAL_TWICE, "one scheduler, one read per interval");

    rig.service.shutdown();
    first.await.expect("ends cleanly");
    second.await.expect("ends cleanly");
}

#[tokio::test(start_paused = true)]
async fn a_cancelled_run_releases_the_claim_too() {
    let mut rig = rig().start().await;
    let runner = rig.take_runner();
    runner.abort();
    assert!(runner.await.is_err_and(|error| error.is_cancelled()));

    rig.run().await;
    assert!(!rig.runner_finished());
    assert_eq!(reads(&rig), [2; 5], "the first loop's read, then the new loop's");
}

#[tokio::test(start_paused = true)]
async fn the_loop_that_replaced_a_dead_one_still_shuts_down_cleanly() {
    let mut rig = rig().sink(RecordingSink::panicking_in_snapshots(1)).build();
    rig.run().await;
    let _ = rig.take_runner().await;
    rig.run().await;
    assert!(!rig.runner_finished(), "the replacement is alive, not a run that gave up");

    rig.service.shutdown();
    rig.take_runner().await.expect("the replacement ends cleanly");
    let seen = reads(&rig);
    rig.after(seconds(3600)).await;
    assert_eq!(reads(&rig), seen, "and polls no more");
}

#[tokio::test(start_paused = true)]
async fn the_recovery_notice_is_not_lost_when_the_sink_panics_on_it() {
    let sink = RecordingSink::panicking_in_notices(1);
    let mut rig = rig().raw_settings(GARBAGE).sink(sink).build();

    rig.run().await;
    assert!(rig.take_runner().await.is_err_and(|error| error.is_panic()));
    assert!(rig.sink.notices().is_empty(), "the delivery that panicked recorded nothing");

    rig.run().await;
    let notices = rig.sink.notices();
    assert_eq!(notices.len(), 1, "the restarted loop says it");
    assert_eq!(notices[0].kind, NotificationKind::SettingsRecovered);
    rig.after(seconds(3600)).await;
    assert_eq!(rig.sink.notices().len(), 1, "and only once");
}
