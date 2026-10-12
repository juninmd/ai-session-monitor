//! A collector that hangs, is slow, panics or answers for the wrong harness hurts nobody else.

use std::time::Duration;

use usage_model::{ErrorCode, HarnessId, ViewState};

use crate::collector::{FakeCollector, Step};
use crate::rig::{rig, seconds, secs};
use crate::views::{of, ready};

const CLAUDE: HarnessId = HarnessId::Claude;

#[tokio::test(start_paused = true)]
async fn a_hung_collector_times_out_at_thirty_seconds_and_blocks_nobody() {
    let rig = rig().collector(FakeCollector::always(CLAUDE, Step::Hang)).start().await;

    rig.at(Duration::from_millis(29_999)).await;
    assert_eq!(of(&rig.service.snapshot().views, CLAUDE).state, ViewState::Loading);
    assert_eq!(rig.fake(CLAUDE).in_flight(), 1);
    assert_eq!(of(&rig.service.snapshot().views, HarnessId::Codex).state, ViewState::Ready);

    rig.at(seconds(30)).await;
    let views = rig.service.snapshot().views;
    let error = of(&views, CLAUDE).error.as_ref().expect("timed out");
    assert_eq!(of(&views, CLAUDE).state, ViewState::Error);
    assert_eq!((error.code, error.retryable), (ErrorCode::Timeout, true));
    assert_eq!(rig.fake(CLAUDE).in_flight(), 0, "the abandoned read was dropped");
    assert_eq!(of(&views, HarnessId::Codex).state, ViewState::Ready, "others are untouched");
}

#[tokio::test(start_paused = true)]
async fn a_collector_that_keeps_hanging_backs_off_like_any_failure() {
    let rig = rig().collector(FakeCollector::always(CLAUDE, Step::Hang)).start().await;
    rig.at(seconds(1000)).await;

    // Each read ends 30 s after it starts; the wait counts from there.
    assert_eq!(rig.fake(CLAUDE).offsets_ms(rig.start), secs(&[0, 150, 420, 930]));
}

#[tokio::test(start_paused = true)]
async fn a_slow_collector_that_answers_in_time_is_not_cut_off() {
    let slow = Step::After(seconds(29), ready(CLAUDE, 10.0));
    let rig = rig().collector(FakeCollector::always(CLAUDE, slow)).start().await;
    rig.at(seconds(30)).await;

    assert_eq!(of(&rig.service.snapshot().views, CLAUDE).state, ViewState::Ready);
}

#[tokio::test(start_paused = true)]
async fn a_panicking_collector_becomes_an_error_and_the_loop_survives() {
    let claude = FakeCollector::scripted(CLAUDE, vec![Step::Panic], Step::View(ready(CLAUDE, 5.0)));
    let rig = rig().collector(claude).start().await;

    rig.at(seconds(1)).await;
    let views = rig.service.snapshot().views;
    let error = of(&views, CLAUDE).error.as_ref().expect("panic reported");
    assert_eq!((error.code, error.retryable), (ErrorCode::Internal, true));
    assert_eq!(error.params.get("where").map(String::as_str), Some("collector"));
    assert_eq!(of(&views, HarnessId::Codex).state, ViewState::Ready, "others are untouched");

    rig.at(seconds(121)).await;
    assert_eq!(of(&rig.service.snapshot().views, CLAUDE).state, ViewState::Ready, "it recovers");
}

#[tokio::test(start_paused = true)]
async fn a_view_for_another_harness_cannot_overwrite_that_harness() {
    let wrong = FakeCollector::always(CLAUDE, Step::View(ready(HarnessId::Codex, 99.0)));
    let rig = rig().collector(wrong).start().await;
    rig.at(seconds(1)).await;

    let views = rig.service.snapshot().views;
    let error = of(&views, CLAUDE).error.as_ref().expect("reported as the bug it is");
    assert_eq!((error.code, error.retryable), (ErrorCode::Internal, false));
    assert_eq!(error.params.get("where").map(String::as_str), Some("collector_id"));
    let codex = of(&views, HarnessId::Codex);
    assert_eq!(codex.windows[0].used_pct, Some(10.0), "Codex still shows its own reading");

    rig.at(seconds(1000)).await;
    assert_eq!(rig.fake(CLAUDE).offsets_ms(rig.start), secs(&[0, 900]), "straight to the maximum");
}
