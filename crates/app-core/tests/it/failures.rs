//! A failed collection must not blank a tab that has data.

use usage_model::{ErrorCode, HarnessId, HarnessView, ViewState};

use crate::collector::{FakeCollector, Step};
use crate::rig::{rig, seconds};
use crate::views::{failing, of, ready};

const CLAUDE: HarnessId = HarnessId::Claude;

fn network_error() -> Step {
    Step::View(failing(CLAUDE, ErrorCode::Network, true))
}

#[tokio::test(start_paused = true)]
async fn a_failed_read_keeps_the_last_good_windows_flagged_stale_with_the_error() {
    let good = ready(CLAUDE, 40.0);
    let claude = FakeCollector::scripted(CLAUDE, vec![Step::View(good.clone())], network_error());
    let rig = rig().collector(claude).start().await;
    rig.at(seconds(61)).await;

    let views = rig.service.snapshot().views;
    let shown = of(&views, CLAUDE);
    assert_eq!(shown.state, ViewState::Ready, "the tab keeps its data");
    assert!(shown.stale);
    assert_eq!(shown.windows, good.windows);
    let error = shown.error.as_ref().expect("the failure travels with the data");
    assert_eq!((error.code, error.retryable), (ErrorCode::Network, true));
    assert_eq!(of(&rig.sink.last_snapshot().views, CLAUDE), shown, "and the UI is told");
}

#[tokio::test(start_paused = true)]
async fn the_next_good_read_clears_the_stale_flag_and_the_error() {
    let steps = vec![Step::View(ready(CLAUDE, 40.0)), network_error()];
    let claude = FakeCollector::scripted(CLAUDE, steps, Step::View(ready(CLAUDE, 30.0)));
    let rig = rig().collector(claude).start().await;
    rig.at(seconds(181)).await;

    let views = rig.service.snapshot().views;
    let shown = of(&views, CLAUDE);
    assert!(!shown.stale && shown.error.is_none());
    assert_eq!(shown.windows[0].used_pct, Some(30.0));
}

#[tokio::test(start_paused = true)]
async fn the_latest_error_replaces_the_earlier_one_on_stale_data() {
    let steps = vec![
        Step::View(ready(CLAUDE, 40.0)),
        network_error(),
        Step::View(failing(CLAUDE, ErrorCode::RateLimited, true)),
    ];
    let claude = FakeCollector::scripted(CLAUDE, steps, network_error());
    let rig = rig().collector(claude).start().await;
    rig.at(seconds(181)).await;

    let views = rig.service.snapshot().views;
    let shown = of(&views, CLAUDE);
    assert_eq!(shown.error.as_ref().map(|e| e.code), Some(ErrorCode::RateLimited));
    assert_eq!(shown.windows[0].used_pct, Some(40.0), "still the last good reading");
}

#[tokio::test(start_paused = true)]
async fn a_failure_before_any_success_is_an_error_with_no_data() {
    let rig = rig().collector(FakeCollector::always(CLAUDE, network_error())).start().await;
    rig.at(seconds(1)).await;

    let views = rig.service.snapshot().views;
    let shown = of(&views, CLAUDE);
    assert_eq!(shown.state, ViewState::Error);
    assert!(shown.windows.is_empty() && !shown.stale);
}

#[tokio::test(start_paused = true)]
async fn a_failure_after_an_empty_state_is_just_an_error() {
    let empty = Step::View(HarnessView::with_state(CLAUDE, ViewState::NoData));
    let claude = FakeCollector::scripted(CLAUDE, vec![empty], network_error());
    let rig = rig().collector(claude).start().await;
    rig.at(seconds(61)).await;

    let views = rig.service.snapshot().views;
    assert_eq!(of(&views, CLAUDE).state, ViewState::Error, "there was no data to keep");
}
