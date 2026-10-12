//! When a recording stops describing the present: every edge of the clock, through `collect`.
//! The fixture's windows reset at NOW + 9_000 (five hours) and NOW + 400_000 (weekly).

use serde_json::json;
use usage_model::ViewState;

use crate::common::NOW;
use crate::setup::{installed, recording, view_at};

const DAY: i64 = 86_400;

#[tokio::test]
async fn a_recording_is_fresh_until_the_last_window_resets() {
    let env = recording();
    assert!(!view_at(&env, NOW).await.stale);
    assert!(
        !view_at(&env, NOW + 9_000).await.stale,
        "the short window reset, the weekly one did not"
    );
    assert!(!view_at(&env, NOW + 399_999).await.stale, "one second before the last reset");
    let at_reset = view_at(&env, NOW + 400_000).await;
    assert!(at_reset.stale, "the reset instant counts as reset");
    assert_eq!(
        (at_reset.state, at_reset.windows.len()),
        (ViewState::Ready, 2),
        "stale numbers stay visible"
    );
    assert_eq!(at_reset.read_at, Some(NOW), "and say how old they are");
}

#[tokio::test]
async fn a_window_without_a_reset_time_never_counts_as_reset() {
    let env = installed();
    let open_ended = json!({"v": 1, "capturedAt": NOW, "rateLimits": {
        "five_hour": {"used_percentage": 5, "resets_at": NOW + 10},
        "seven_day": {"used_percentage": 6}
    }});
    env.write_rate_limits(&open_ended.to_string());
    assert!(!view_at(&env, NOW + 100_000).await.stale);
}

#[tokio::test]
async fn nothing_describes_the_present_after_eight_days() {
    let env = installed();
    let no_resets = |captured_at: i64| {
        json!({"v": 1, "capturedAt": captured_at, "rateLimits": {"seven_day": {"used_percentage": 6}}}).to_string()
    };
    env.write_rate_limits(&no_resets(NOW - 8 * DAY));
    assert!(!view_at(&env, NOW).await.stale, "exactly eight days is still inside");
    env.write_rate_limits(&no_resets(NOW - 8 * DAY - 1));
    assert!(view_at(&env, NOW).await.stale);
}

#[tokio::test]
async fn a_recording_from_the_future_is_stale_beyond_a_small_clock_skew() {
    let env = installed();
    let at = |captured_at: i64| {
        json!({"v": 1, "capturedAt": captured_at, "rateLimits": {"seven_day": {"used_percentage": 6}}}).to_string()
    };
    env.write_rate_limits(&at(NOW + 300));
    assert!(!view_at(&env, NOW).await.stale);
    env.write_rate_limits(&at(NOW + 301));
    assert!(view_at(&env, NOW).await.stale, "the clock was set back, or the file is not ours");
}

#[tokio::test]
async fn implausible_timestamps_are_a_changed_format_not_a_view() {
    let env = installed();
    for captured_at in [
        json!(0),
        json!(-5),
        json!(12),
        json!(4_102_444_801_i64),
        json!(1.5),
        json!("1800000000"),
        json!(null),
    ] {
        let doc = json!({"v": 1, "capturedAt": captured_at, "rateLimits": null});
        env.write_rate_limits(&doc.to_string());
        let view = view_at(&env, NOW).await;
        assert_eq!(view.state, ViewState::Error, "{captured_at}");
        assert_eq!(
            view.error.unwrap().code,
            usage_model::ErrorCode::EndpointChanged,
            "{captured_at}"
        );
    }
}

#[tokio::test]
async fn an_absurd_clock_makes_the_view_stale_but_never_breaks_it() {
    let env = recording();
    for now in [i64::MIN, i64::MIN + 1, -1, 0, i64::MAX - 1, i64::MAX] {
        let view = view_at(&env, now).await;
        assert_eq!(
            (view.state, view.stale, view.windows.len()),
            (ViewState::Ready, true, 2),
            "{now}"
        );
        assert!(view.error.is_none(), "{now}: arithmetic must not overflow into a lost task");
    }
}

#[tokio::test]
async fn the_view_follows_the_clock_it_is_given() {
    let env = recording();
    assert!(!view_at(&env, NOW).await.stale);
    assert!(view_at(&env, NOW + 400_000).await.stale);
    assert!(
        !view_at(&env, NOW - 100).await.stale,
        "a clock a little behind the recording is within the skew"
    );
}
