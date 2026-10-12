//! A rate-limited source is left alone for a cool-down, keeps its last good view and says when
//! the next attempt is.

use usage_model::{ErrorCode, HarnessError, HarnessId, HarnessView};

use crate::collector::{FakeCollector, Step};
use crate::rig::{rig, seconds, secs, settle};
use crate::views::{BASE, of, ready};

const CLAUDE: HarnessId = HarnessId::Claude;

fn limited(retry_after: Option<&str>) -> Step {
    let mut error = HarnessError::new(ErrorCode::RateLimited, true).with_param("httpStatus", "429");
    if let Some(value) = retry_after {
        error = error.with_param("retryAfterSecs", value);
    }
    Step::View(HarnessView::error(CLAUDE, error))
}

fn good_then(limit: Step) -> std::sync::Arc<FakeCollector> {
    FakeCollector::scripted(CLAUDE, vec![Step::View(ready(CLAUDE, 10.0))], limit)
}

#[tokio::test(start_paused = true)]
async fn a_rate_limit_waits_five_minutes_by_default() {
    let rig = rig().collector(good_then(limited(None))).start().await;
    rig.at(seconds(700)).await;

    // Good read at 0, limited at 60 (cool-down 300), limited again at 360, again at 660.
    assert_eq!(rig.fake(CLAUDE).offsets_ms(rig.start), secs(&[0, 60, 360, 660]));
}

#[tokio::test(start_paused = true)]
async fn the_view_keeps_the_last_good_windows_stale_and_names_the_next_attempt() {
    let rig = rig().collector(good_then(limited(None))).start().await;
    rig.at(seconds(100)).await;

    let snapshot = rig.service.snapshot();
    let view = of(&snapshot.views, CLAUDE);
    assert!(view.stale && view.windows.len() == 1, "last good windows kept, flagged stale");
    assert_eq!(view.error.as_ref().map(|e| e.code), Some(ErrorCode::RateLimited));
    assert_eq!(view.next_attempt_at, Some(BASE + 60 + 300), "the failure at 60 s plus 5 min");
}

#[tokio::test(start_paused = true)]
async fn a_retry_after_is_honoured_and_capped_at_thirty_minutes() {
    let first = rig().collector(good_then(limited(Some("900")))).start().await;
    first.at(seconds(1000)).await;
    assert_eq!(first.fake(CLAUDE).offsets_ms(first.start), secs(&[0, 60, 960]));
    let snapshot = first.service.snapshot();
    assert_eq!(of(&snapshot.views, CLAUDE).next_attempt_at, Some(BASE + 960 + 900));

    let capped = rig().collector(good_then(limited(Some("86400")))).start().await;
    capped.at(seconds(2000)).await;
    assert_eq!(capped.fake(CLAUDE).offsets_ms(capped.start), secs(&[0, 60, 1860]));
}

#[tokio::test(start_paused = true)]
async fn the_cool_down_never_undercuts_the_normal_cadence() {
    let rig = rig()
        .settings(|s| s.general.interval_secs = 3600)
        .collector(good_then(limited(Some("30"))))
        .start()
        .await;
    rig.at(seconds(7300)).await;

    assert_eq!(rig.fake(CLAUDE).offsets_ms(rig.start), secs(&[0, 3600, 7200]));
}

#[tokio::test(start_paused = true)]
async fn a_manual_refresh_during_the_cool_down_is_dropped_and_works_after_it() {
    let rig = rig().collector(good_then(limited(None))).start().await;
    rig.at(seconds(100)).await;
    let claude = rig.fake(CLAUDE);
    assert_eq!(claude.count(), 2);

    rig.service.refresh(Some(CLAUDE));
    settle().await;
    assert_eq!(claude.count(), 2, "dropped: the source asked to be left alone");

    rig.at(seconds(361)).await;
    assert_eq!(claude.count(), 3, "the scheduled attempt after the cool-down");
    rig.at(seconds(400)).await;
    rig.service.refresh(Some(CLAUDE));
    settle().await;
    assert_eq!(claude.count(), 3, "still cooling after the second limit");
}

#[tokio::test(start_paused = true)]
async fn a_good_read_ends_the_cool_down_and_clears_the_mark() {
    let steps = vec![Step::View(ready(CLAUDE, 10.0)), limited(None)];
    let claude = FakeCollector::scripted(CLAUDE, steps, Step::View(ready(CLAUDE, 12.0)));
    let rig = rig().collector(claude).start().await;
    rig.at(seconds(100)).await;
    assert!(of(&rig.service.snapshot().views, CLAUDE).next_attempt_at.is_some());

    rig.at(seconds(361)).await;
    let snapshot = rig.service.snapshot();
    let view = of(&snapshot.views, CLAUDE);
    assert_eq!((view.stale, view.error.is_none(), view.next_attempt_at), (false, true, None));
}

#[tokio::test(start_paused = true)]
async fn changing_the_sources_own_settings_ends_the_cool_down() {
    let rig = rig().collector(good_then(limited(None))).start().await;
    rig.at(seconds(100)).await;
    assert_eq!(rig.fake(CLAUDE).count(), 2);

    rig.edit(|s| s.claude.oauth_usage = true);
    settle().await;
    assert_eq!(rig.fake(CLAUDE).count(), 3, "new settings are a fresh try");
}

#[tokio::test(start_paused = true)]
async fn other_failures_back_off_without_a_next_attempt_mark() {
    let network =
        Step::View(HarnessView::error(CLAUDE, HarnessError::new(ErrorCode::Network, true)));
    let rig = rig().collector(good_then(network)).start().await;
    rig.at(seconds(100)).await;

    assert_eq!(of(&rig.service.snapshot().views, CLAUDE).next_attempt_at, None);
}
