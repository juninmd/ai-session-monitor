//! `snapshot` events: only when something changed, at most four a second, and the last change is
//! never lost.

use std::sync::Arc;
use std::time::Duration;

use usage_model::{HarnessId, ViewState};

use crate::collector::{FakeCollector, Step};
use crate::rig::{rig, seconds};
use crate::views::ready;

const GAP: Duration = Duration::from_millis(250);

/// Answers after `delay_ms` with different content on every call, so every poll is a change.
fn staggered(id: HarnessId, delay_ms: u64) -> Arc<FakeCollector> {
    let delay = Duration::from_millis(delay_ms);
    FakeCollector::new(id, move |n| Step::After(delay, ready(id, f64::from((n % 90) as u32))))
}

fn event_times_ms(rig: &crate::rig::Rig) -> Vec<u128> {
    rig.sink.snapshots().iter().map(|(at, _)| at.duration_since(rig.start).as_millis()).collect()
}

#[tokio::test(start_paused = true)]
async fn five_answers_at_the_start_make_two_events_not_six() {
    let rig = rig().start().await;
    rig.at(seconds(1)).await;

    assert_eq!(event_times_ms(&rig), [0, 250], "the initial state, then everything at once");
    let last = rig.sink.last_snapshot();
    assert!(last.views.iter().all(|view| view.state == ViewState::Ready));
}

#[tokio::test(start_paused = true)]
async fn staggered_answers_are_batched_into_events_at_least_250_ms_apart() {
    let builder = [0, 100, 200, 300]
        .into_iter()
        .chain(std::iter::repeat(400))
        .zip(HarnessId::ALL)
        .fold(rig(), |builder, (delay, id)| builder.collector(staggered(id, delay)));
    let rig = builder.start().await;
    rig.at(seconds(2)).await;

    assert_eq!(event_times_ms(&rig), [0, 250, 500]);
    let events = rig.sink.snapshots();
    let ready_in =
        |n: usize| events[n].1.views.iter().filter(|view| view.state == ViewState::Ready).count();
    assert_eq!((ready_in(0), ready_in(1), ready_in(2)), (0, 3, HarnessId::ALL.len()));
}

#[tokio::test(start_paused = true)]
async fn identical_results_are_not_announced_again() {
    let rig = rig().start().await;
    rig.at(seconds(599)).await;

    assert!(rig.fake(HarnessId::Claude).count() > 5, "it kept reading");
    assert_eq!(rig.sink.snapshots().len(), 2, "but nothing changed after the first answers");
}

#[tokio::test(start_paused = true)]
async fn a_change_after_a_quiet_period_goes_out_at_once() {
    let steps = vec![Step::View(ready(HarnessId::Claude, 10.0))];
    let claude = FakeCollector::scripted(
        HarnessId::Claude,
        steps,
        Step::View(ready(HarnessId::Claude, 20.0)),
    );
    let rig = rig().collector(claude).start().await;
    rig.at(seconds(61)).await;

    assert_eq!(event_times_ms(&rig), [0, 250, 60_000]);
}

#[tokio::test(start_paused = true)]
async fn under_constant_change_events_stay_spaced_and_the_last_change_arrives() {
    let builder = [0, 130, 270, 90, 410]
        .into_iter()
        .chain(std::iter::repeat(400))
        .zip(HarnessId::ALL)
        .fold(rig().settings(|s| s.general.interval_secs = 30), |builder, (delay, id)| {
            builder.collector(staggered(id, delay))
        });
    let rig = builder.start().await;
    rig.at(seconds(601)).await;

    let times = rig.sink.snapshots().iter().map(|(at, _)| *at).collect::<Vec<_>>();
    assert!(times.len() > 20, "plenty of changes: {}", times.len());
    for pair in times.windows(2) {
        assert!(pair[1].duration_since(pair[0]) >= GAP, "two events closer than 250 ms");
    }
    assert_eq!(rig.sink.last_snapshot().views, rig.service.snapshot().views, "nothing is lost");
}
