//! Usage-threshold alerts wired into the service: one toast per crossing per reset epoch.

use app_core::NotificationKind;
use usage_model::HarnessId;

use crate::collector::{FakeCollector, Step};
use crate::rig::{Rig, RigBuilder, rig, seconds};
use crate::views::{ready, ready_resetting};

const CODEX: HarnessId = HarnessId::Codex;

fn alerting(steps: Vec<Step>) -> RigBuilder {
    let last = steps.last().cloned().unwrap_or(Step::Hang);
    rig()
        .settings(|s| s.general.alert_thresholds = vec![80, 95])
        .collector(FakeCollector::scripted(CODEX, steps, last))
}

fn titles(rig: &Rig) -> Vec<String> {
    rig.sink.notices().iter().map(|n| n.title.clone()).collect()
}

#[tokio::test(start_paused = true)]
async fn each_threshold_toasts_once_when_crossed() {
    let used = [60.0, 85.0, 88.0, 96.0, 97.0];
    let rig = alerting(used.iter().map(|u| Step::View(ready(CODEX, *u))).collect()).start().await;
    rig.at(seconds(300)).await;

    assert_eq!(titles(&rig), ["Codex: 80% do limite usado", "Codex: 95% do limite usado"]);
    assert!(rig.sink.notices().iter().all(|n| n.kind == NotificationKind::ThresholdCrossed));
}

#[tokio::test(start_paused = true)]
async fn a_jump_over_both_thresholds_toasts_only_the_highest() {
    let steps = vec![Step::View(ready(CODEX, 20.0)), Step::View(ready(CODEX, 97.0))];
    let rig = alerting(steps).start().await;
    rig.at(seconds(200)).await;

    assert_eq!(titles(&rig), ["Codex: 95% do limite usado"]);
}

#[tokio::test(start_paused = true)]
async fn the_window_resetting_re_arms_the_alert() {
    let steps = vec![
        Step::View(ready_resetting(CODEX, 50.0, 1000)),
        Step::View(ready_resetting(CODEX, 90.0, 1000)),
        Step::View(ready_resetting(CODEX, 5.0, 9000)),
        Step::View(ready_resetting(CODEX, 85.0, 9000)),
    ];
    let rig = alerting(steps).start().await;
    rig.at(seconds(300)).await;

    assert_eq!(titles(&rig), ["Codex: 80% do limite usado", "Codex: 80% do limite usado"]);
}

#[tokio::test(start_paused = true)]
async fn already_above_a_threshold_at_start_is_silent_and_off_means_off() {
    let on = alerting(vec![Step::View(ready(CODEX, 90.0))]).start().await;
    on.at(seconds(300)).await;
    assert!(titles(&on).is_empty(), "the first reading only sets the baseline");

    let script = FakeCollector::scripted(
        CODEX,
        vec![Step::View(ready(CODEX, 20.0))],
        Step::View(ready(CODEX, 99.0)),
    );
    let off =
        rig().settings(|s| s.general.alert_thresholds.clear()).collector(script).start().await;
    off.at(seconds(300)).await;
    assert!(titles(&off).is_empty());
}
