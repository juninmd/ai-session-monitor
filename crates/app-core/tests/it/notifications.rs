//! `plan` wired into the service: the right toasts, once, after the snapshot they belong to.

use app_core::NotificationKind;
use usage_model::HarnessId;

use crate::collector::{FakeCollector, Step};
use crate::rig::{RigBuilder, rig, seconds};
use crate::views::ready;

const CODEX: HarnessId = HarnessId::Codex;

fn codex_script(used: &[f64]) -> std::sync::Arc<FakeCollector> {
    let steps = used.iter().map(|pct| Step::View(ready(CODEX, *pct))).collect();
    FakeCollector::scripted(CODEX, steps, Step::View(ready(CODEX, used[used.len() - 1])))
}

fn notifying(used: &[f64]) -> RigBuilder {
    rig()
        .settings(|s| {
            s.general.notify_on_out = true;
            s.general.notify_on_reset = true;
        })
        .collector(codex_script(used))
}

#[tokio::test(start_paused = true)]
async fn a_window_running_out_and_coming_back_notifies_once_each() {
    // Reads at 0, 60, 120, 180, 240: 60 % used, out, still out, back to 20 % used, same.
    let rig = notifying(&[60.0, 100.0, 100.0, 20.0, 20.0]).start().await;

    rig.at(seconds(130)).await;
    let notices = rig.sink.notices();
    assert_eq!(notices.len(), 1, "staying out is not a new event");
    assert_eq!((notices[0].kind, notices[0].harness), (NotificationKind::WindowOut, CODEX));
    assert_eq!(notices[0].window_id, "five_hour");
    assert!(notices[0].title.contains("Codex"), "{}", notices[0].title);

    rig.at(seconds(250)).await;
    let kinds: Vec<_> = rig.sink.notices().iter().map(|n| n.kind).collect();
    assert_eq!(kinds, [NotificationKind::WindowOut, NotificationKind::WindowReset]);
}

#[tokio::test(start_paused = true)]
async fn nothing_is_notified_unless_the_user_asked_for_it() {
    let rig = rig().collector(codex_script(&[60.0, 100.0, 20.0])).start().await;
    rig.at(seconds(200)).await;

    assert!(rig.sink.notices().is_empty());
}

#[tokio::test(start_paused = true)]
async fn the_first_snapshot_never_notifies_even_when_already_out() {
    let rig = notifying(&[100.0]).start().await;
    rig.at(seconds(200)).await;

    assert!(rig.sink.notices().is_empty());
}

#[tokio::test(start_paused = true)]
async fn a_notification_follows_the_snapshot_it_belongs_to() {
    let rig = notifying(&[60.0, 100.0]).start().await;
    rig.at(seconds(61)).await;

    let kinds = rig.sink.kinds();
    let at = kinds.iter().position(|kind| *kind == "notice").expect("one notice");
    assert_eq!(kinds[at - 1], "snapshot", "the UI shows the new state before the toast");
    assert_eq!(kinds.iter().filter(|kind| **kind == "notice").count(), 1);
}

#[tokio::test(start_paused = true)]
async fn quiet_hours_hold_the_toasts_inside_the_window_and_not_after_it() {
    // BASE is 08:00 UTC. The out is read at 120 s (08:02), inside 08:00-08:03; the reset at 240 s
    // (08:04) falls after it.
    let rig = notifying(&[60.0, 100.0, 100.0, 20.0, 20.0])
        .settings(|s| {
            s.general.quiet_hours_on = true;
            s.general.quiet_from = 8 * 60;
            s.general.quiet_to = 8 * 60 + 3;
        })
        .start()
        .await;

    rig.at(seconds(130)).await;
    assert!(rig.sink.notices().is_empty(), "the out inside the window is dropped, not queued");

    rig.at(seconds(250)).await;
    let kinds: Vec<_> = rig.sink.notices().iter().map(|n| n.kind).collect();
    assert_eq!(kinds, [NotificationKind::WindowReset]);
}
