//! `shutdown` ends `run` promptly and takes the work in flight with it.

use std::sync::Arc;

use tokio::time::Instant;
use usage_model::HarnessId;

use crate::collector::{FakeCollector, Step};
use crate::rig::{rig, seconds, settle};

const CLAUDE: HarnessId = HarnessId::Claude;

#[tokio::test(start_paused = true)]
async fn shutdown_ends_run_at_once_and_drops_the_collection_in_flight() {
    let mut rig = rig().collector(FakeCollector::always(CLAUDE, Step::Hang)).start().await;
    rig.at(seconds(5)).await;
    assert_eq!(rig.fake(CLAUDE).in_flight(), 1);

    let before = Instant::now();
    rig.service.shutdown();
    rig.take_runner().await.expect("the loop ends cleanly");

    assert_eq!(Instant::now(), before, "no virtual time passed: nothing waited on a timer");
    assert_eq!(rig.fake(CLAUDE).in_flight(), 0, "the hung read was dropped");
}

#[tokio::test(start_paused = true)]
async fn shutdown_and_wait_returns_only_after_the_loop_unwound() {
    let mut rig = rig().collector(FakeCollector::always(CLAUDE, Step::Hang)).start().await;
    rig.at(seconds(5)).await;

    assert!(rig.service.shutdown_and_wait(seconds(2)).await);
    assert_eq!(rig.fake(CLAUDE).in_flight(), 0, "the read was dropped before the wait ended");
    rig.take_runner().await.expect("the loop ends cleanly");
}

#[tokio::test(start_paused = true)]
async fn shutdown_and_wait_without_a_running_loop_returns_at_once() {
    let rig = rig().build();
    assert!(rig.service.shutdown_and_wait(seconds(2)).await);
}

#[tokio::test(start_paused = true)]
async fn nothing_happens_after_shutdown_and_the_api_stays_safe() {
    let mut rig = rig().start().await;
    rig.at(seconds(100)).await;
    rig.service.shutdown();
    rig.service.shutdown();
    rig.take_runner().await.expect("ends cleanly");
    let (reads, events) = (rig.fake(CLAUDE).count(), rig.sink.kinds().len());

    rig.after(seconds(3600)).await;
    rig.service.refresh(None);
    settle().await;
    assert_eq!(rig.fake(CLAUDE).count(), reads, "no more reads");
    assert_eq!(rig.sink.kinds().len(), events, "no more events");

    let saved = rig.edit(|s| s.general.interval_secs = 90);
    assert_eq!(rig.saved_settings(), saved, "saving still works, there is just nobody to apply it");
    assert_eq!(rig.service.snapshot().views.len(), 7);
}

#[tokio::test(start_paused = true)]
async fn shutdown_before_run_makes_run_return_at_once_and_say_nothing() {
    let rig = rig().build();
    rig.service.shutdown();
    Arc::clone(&rig.service).run().await;

    assert!(rig.sink.kinds().is_empty());
    for id in HarnessId::ALL {
        assert_eq!(rig.fake(id).count(), 0, "{id:?}");
    }
}

#[tokio::test(start_paused = true)]
async fn a_second_run_returns_at_once_and_does_not_double_the_reads() {
    let rig = rig().start().await;

    let second = Arc::clone(&rig.service).run();
    tokio::time::timeout(seconds(1), second).await.expect("the second run returns immediately");
    settle().await;

    for id in HarnessId::ALL {
        assert_eq!(rig.fake(id).count(), 1, "{id:?}");
    }
}

#[tokio::test(start_paused = true)]
async fn the_service_can_be_shared_across_threads() {
    fn needs_send<T: Send>(_: &T) {}
    fn needs_send_sync<T: Send + Sync>(_: &T) {}

    let rig = rig().build();
    needs_send_sync(&rig.service);
    let run = Arc::clone(&rig.service).run();
    needs_send(&run);
    drop(run);

    let service = Arc::clone(&rig.service);
    let from_another_thread = std::thread::spawn(move || service.settings());
    assert_eq!(from_another_thread.join().ok(), Some(rig.service.settings()));
}
