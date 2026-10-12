//! What failures do to the schedule: exponential backoff, its reset, and parked harnesses.

use usage_model::{ErrorCode, HarnessId, SecretKind, ViewState};

use crate::collector::{FakeCollector, Step};
use crate::rig::{rig, seconds, secs, settle};
use crate::views::{failing, of, ready};

const CLAUDE: HarnessId = HarnessId::Claude;
const CODEX: HarnessId = HarnessId::Codex;
const LITELLM: HarnessId = HarnessId::Litellm;

fn network_error() -> Step {
    Step::View(failing(CLAUDE, ErrorCode::Network, true))
}

#[tokio::test(start_paused = true)]
async fn retryable_failures_double_the_wait_up_to_fifteen_minutes() {
    let rig = rig().collector(FakeCollector::always(CLAUDE, network_error())).start().await;
    rig.at(seconds(2699)).await;

    let offsets = rig.fake(CLAUDE).offsets_ms(rig.start);
    assert_eq!(offsets, secs(&[0, 120, 360, 840, 1740, 2640]));
    assert_eq!(rig.fake(HarnessId::Codex).count(), 45, "a failing harness slows only itself");
}

#[tokio::test(start_paused = true)]
async fn a_success_resets_the_progression() {
    let ok = Step::View(ready(CLAUDE, 10.0));
    let claude = FakeCollector::scripted(
        CLAUDE,
        vec![network_error(), network_error(), ok],
        network_error(),
    );
    let rig = rig().collector(claude).start().await;
    rig.at(seconds(800)).await;

    assert_eq!(rig.fake(CLAUDE).offsets_ms(rig.start), secs(&[0, 120, 360, 420, 540, 780]));
}

#[tokio::test(start_paused = true)]
async fn an_error_that_needs_the_user_goes_straight_to_fifteen_minutes() {
    let denied = Step::View(failing(CLAUDE, ErrorCode::Unauthorized, false));
    let rig = rig().collector(FakeCollector::always(CLAUDE, denied)).start().await;
    rig.at(seconds(1850)).await;

    assert_eq!(rig.fake(CLAUDE).offsets_ms(rig.start), secs(&[0, 900, 1800]));
}

#[tokio::test(start_paused = true)]
async fn backoff_never_polls_faster_than_the_interval() {
    let rig = rig()
        .settings(|s| s.general.interval_secs = 3600)
        .collector(FakeCollector::always(CLAUDE, network_error()))
        .start()
        .await;
    rig.at(seconds(3700)).await;

    assert_eq!(rig.fake(CLAUDE).offsets_ms(rig.start), secs(&[0, 3600]));
}

#[tokio::test(start_paused = true)]
async fn a_harness_waiting_for_consent_is_not_polled_until_something_changes() {
    let gated = Step::View(usage_model::HarnessView::needs_consent(HarnessId::Codex));
    let rig = rig().collector(FakeCollector::always(HarnessId::Codex, gated)).start().await;
    rig.at(seconds(3600)).await;
    let codex = rig.fake(HarnessId::Codex);

    assert_eq!(codex.count(), 1, "an hour of silence: nothing to read until the user acts");
    assert_eq!(of(&rig.service.snapshot().views, HarnessId::Codex).state, ViewState::NeedsConsent);

    rig.service.refresh(Some(HarnessId::Codex));
    settle().await;
    assert_eq!(codex.count(), 2, "a manual refresh looks again");

    rig.after(seconds(3600)).await;
    assert_eq!(codex.count(), 2, "and parks again");

    rig.edit(|s| s.codex.consent = true);
    settle().await;
    assert_eq!(codex.count(), 3, "granting consent reads at once");
}

#[tokio::test(start_paused = true)]
async fn a_failure_after_a_parked_state_starts_the_progression_over() {
    let gated = Step::View(usage_model::HarnessView::needs_setup(CLAUDE));
    let claude = FakeCollector::scripted(CLAUDE, vec![gated], network_error());
    let rig = rig().collector(claude).start().await;
    rig.at(seconds(10)).await;
    rig.service.refresh(Some(CLAUDE));
    rig.at(seconds(131)).await;

    let offsets = rig.fake(CLAUDE).offsets_ms(rig.start);
    assert_eq!(offsets, secs(&[0, 10, 130]), "first failure waits 120 s, not a longer step");
}

/// Always failing and retryable: reads at 0 and 120 leave the third one waiting until 360.
fn failing_codex() -> std::sync::Arc<FakeCollector> {
    FakeCollector::always(CODEX, Step::View(failing(CODEX, ErrorCode::Network, true)))
}

#[tokio::test(start_paused = true)]
async fn a_new_secret_starts_the_backoff_over_instead_of_continuing_it() {
    let failing = Step::View(failing(LITELLM, ErrorCode::Network, true));
    let rig = rig()
        .settings(|s| s.litellm.poll_interval_secs = 60)
        .collector(FakeCollector::always(LITELLM, failing))
        .start()
        .await;
    rig.at(seconds(130)).await;
    assert_eq!(rig.fake(LITELLM).offsets_ms(rig.start), secs(&[0, 120]), "control: 2 failures");

    rig.service.set_secret(SecretKind::Litellm, "sk-fake-5d3b".to_owned()).expect("stored");
    rig.at(seconds(300)).await;

    let offsets = rig.fake(LITELLM).offsets_ms(rig.start);
    assert_eq!(offsets, secs(&[0, 120, 130, 250]), "the next wait is 120 s, not the old 480 s");
}

#[tokio::test(start_paused = true)]
async fn an_edited_section_starts_the_backoff_over_instead_of_continuing_it() {
    let rig = rig().collector(failing_codex()).start().await;
    rig.at(seconds(130)).await;
    assert_eq!(rig.fake(CODEX).offsets_ms(rig.start), secs(&[0, 120]), "control: 2 failures");

    rig.edit(|s| s.codex.consent = true);
    rig.at(seconds(300)).await;

    assert_eq!(rig.fake(CODEX).offsets_ms(rig.start), secs(&[0, 120, 130, 250]));
}

#[tokio::test(start_paused = true)]
async fn an_off_and_on_pair_the_loop_never_saw_starts_the_backoff_over() {
    let rig = rig().collector(failing_codex()).start().await;
    rig.at(seconds(130)).await;

    // No await between the saves: the loop only sees the harness enabled, with the same section.
    rig.edit(|s| s.codex.enabled = false);
    rig.edit(|s| s.codex.enabled = true);
    rig.at(seconds(300)).await;

    assert_eq!(rig.fake(CODEX).offsets_ms(rig.start), secs(&[0, 120, 130, 250]));
}

#[tokio::test(start_paused = true)]
async fn switching_a_failing_harness_off_and_on_starts_the_backoff_over() {
    let rig = rig().collector(failing_codex()).start().await;
    rig.at(seconds(130)).await;

    rig.edit(|s| s.codex.enabled = false);
    rig.at(seconds(135)).await;
    rig.edit(|s| s.codex.enabled = true);
    rig.at(seconds(300)).await;

    assert_eq!(rig.fake(CODEX).offsets_ms(rig.start), secs(&[0, 120, 135, 255]));
}
