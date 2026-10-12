//! Windows that reset between two polls. The fake always reports the same reset instants, so the
//! only thing that changes from poll to poll is the clock the caller passes in.

use usage_model::ViewState;

use crate::common::{FIVE_HOUR_RESET, Rig, WEEKLY_RESET};

#[tokio::test]
async fn the_view_turns_stale_only_once_every_window_has_reset() {
    let rig = Rig::new("ok");
    let polls = [
        (FIVE_HOUR_RESET - 1, false, "nothing has reset"),
        (
            FIVE_HOUR_RESET,
            false,
            "the five-hour window resets at this very second, the week runs on",
        ),
        (WEEKLY_RESET - 1, false, "one second before the last reset"),
        (WEEKLY_RESET, true, "the reset instant itself counts as reset"),
        (WEEKLY_RESET + 86_400, true, "a day later"),
    ];
    for (now, stale, why) in polls {
        let view = rig.collect_at(now).await;
        assert_eq!(view.state, ViewState::Ready, "{why}");
        assert_eq!(view.stale, stale, "{why}");
        assert_eq!(view.read_at, Some(now), "{why}: the data is as old as the poll");
        assert_eq!(view.windows.len(), 2, "{why}: reset windows stay on screen, dimmed");
        assert_eq!(view.windows[0].used_pct, Some(42.0), "{why}");
    }
}

#[tokio::test]
async fn a_single_window_is_stale_the_moment_it_resets() {
    let rig = Rig::new("plus_only");
    assert!(!rig.collect_at(FIVE_HOUR_RESET - 1).await.stale);
    assert!(rig.collect_at(FIVE_HOUR_RESET).await.stale);

    let weekly_only = Rig::new("pro_no_primary");
    assert!(!weekly_only.collect_at(WEEKLY_RESET - 1).await.stale);
    assert!(weekly_only.collect_at(WEEKLY_RESET).await.stale);
}

#[tokio::test]
async fn extreme_clocks_do_not_break_the_view() {
    let rig = Rig::new("ok");
    for (now, stale) in [(i64::MIN, false), (-1, false), (0, false), (i64::MAX, true)] {
        let view = rig.collect_at(now).await;
        assert_eq!(view.state, ViewState::Ready, "now = {now}");
        assert_eq!((view.stale, view.read_at), (stale, Some(now)), "now = {now}");
    }
}

#[tokio::test]
async fn the_api_key_view_is_never_stale() {
    let view = Rig::new("api_key").collect_at(i64::MAX).await;
    assert!(!view.stale, "no windows, nothing that could have reset");
}
