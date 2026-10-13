//! How often each harness is collected when everything works.

use usage_model::HarnessId;

use crate::rig::{rig, seconds, secs};

const OPENCODE: HarnessId = HarnessId::Opencode;
const LITELLM: HarnessId = HarnessId::Litellm;

fn every(step: u64, until: u64) -> Vec<u64> {
    (0..=until / step).map(|n| n * step).collect()
}

#[tokio::test(start_paused = true)]
async fn each_harness_follows_its_own_cadence() {
    let rig = rig()
        .settings(|s| {
            s.general.interval_secs = 30;
            s.claude.oauth_usage = true;
            s.litellm.poll_interval_secs = 600;
        })
        .start()
        .await;
    rig.at(seconds(659)).await;

    let offsets = |id| rig.fake(id).offsets_ms(rig.start);
    assert_eq!(offsets(HarnessId::Codex), secs(&every(30, 659)), "the general interval");
    assert_eq!(offsets(HarnessId::Antigravity), secs(&every(30, 659)));
    assert_eq!(offsets(OPENCODE), secs(&[0, 300, 600]), "the Go endpoint is not polled faster");
    let remote = |id: &HarnessId| {
        !matches!(
            id,
            HarnessId::Claude
                | HarnessId::Codex
                | HarnessId::Antigravity
                | HarnessId::Opencode
                | HarnessId::Litellm
        )
    };
    for id in HarnessId::ALL.into_iter().filter(remote) {
        assert_eq!(offsets(id), secs(&[0, 300, 600]), "{id:?} is remote too");
    }
    assert_eq!(offsets(LITELLM), secs(&[0, 600]), "LiteLLM has its own interval");
}

#[tokio::test(start_paused = true)]
async fn claude_reads_its_local_file_at_the_general_interval_even_with_oauth_on() {
    // The OAuth call is throttled inside the Claude collector, so the shim file is not slowed.
    let rig = rig()
        .settings(|s| {
            s.general.interval_secs = 45;
            s.claude.oauth_usage = true;
        })
        .start()
        .await;
    rig.at(seconds(100)).await;

    assert_eq!(rig.fake(HarnessId::Claude).offsets_ms(rig.start), secs(&[0, 45, 90]));
}

#[tokio::test(start_paused = true)]
async fn a_slow_interval_wins_over_the_remote_floor() {
    let rig = rig().settings(|s| s.general.interval_secs = 900).start().await;
    rig.at(seconds(1000)).await;

    assert_eq!(rig.fake(OPENCODE).offsets_ms(rig.start), secs(&[0, 900]));
}

#[tokio::test(start_paused = true)]
async fn an_interval_shortened_while_running_keeps_the_remote_floor_and_litellms_own_clock() {
    let rig = rig()
        .settings(|s| {
            s.general.interval_secs = 600;
            s.litellm.poll_interval_secs = 900;
        })
        .start()
        .await;
    rig.at(seconds(100)).await;

    rig.edit(|s| s.general.interval_secs = 30);
    rig.at(seconds(1000)).await;

    let offsets = |id| rig.fake(id).offsets_ms(rig.start);
    assert_eq!(offsets(OPENCODE), secs(&[0, 300, 600, 900]), "300 s apart, never 30");
    assert_eq!(offsets(LITELLM), secs(&[0, 900]), "the general interval is not its clock");
    assert_eq!(offsets(HarnessId::Claude)[..3], secs(&[0, 100, 130]), "local sources follow it");
}

#[tokio::test(start_paused = true)]
async fn an_interval_lengthened_while_running_postpones_every_source_that_follows_it() {
    let rig = rig().start().await;
    rig.at(seconds(100)).await;

    rig.edit(|s| s.general.interval_secs = 900);
    rig.at(seconds(1000)).await;

    let offsets = |id| rig.fake(id).offsets_ms(rig.start);
    assert_eq!(offsets(OPENCODE), secs(&[0, 900]), "counted from its last read");
    assert_eq!(offsets(HarnessId::Codex), secs(&[0, 60, 960]), "the read at 120 moved to 960");
    assert_eq!(offsets(LITELLM), secs(&[0, 300, 600, 900]), "LiteLLM keeps its own interval");
}
