//! Every harness section of the settings is watched: editing it re-reads that harness, replaces a
//! read in flight that started from the old section, and leaves the other harnesses alone.

use usage_model::{HarnessId, Settings, ViewState};

use crate::collector::{FakeCollector, Step};
use crate::rig::{rig, seconds};
use crate::views::{of, ready};

type Edit = fn(&mut Settings);

fn claude(settings: &mut Settings) {
    settings.claude.shim_consent = true;
}

fn codex(settings: &mut Settings) {
    settings.codex.consent = true;
}

fn antigravity(settings: &mut Settings) {
    settings.antigravity.consent = true;
}

fn opencode(settings: &mut Settings) {
    settings.opencode.local_consent = true;
}

fn litellm(settings: &mut Settings) {
    settings.litellm.base_url = Some("https://litellm.example.test".to_owned());
}

async fn editing_replaces_the_read_in_flight(id: HarnessId, edit: Edit) {
    let slow =
        move |n: usize| Step::After(seconds(20), ready(id, if n == 0 { 11.0 } else { 22.0 }));
    let rig = rig().collector(FakeCollector::new(id, slow)).start().await;
    rig.at(seconds(5)).await;
    assert_eq!(rig.fake(id).in_flight(), 1, "control: the first read is still running");

    rig.edit(edit);
    crate::rig::settle().await;

    for other in HarnessId::ALL {
        let expected = if other == id { 2 } else { 1 };
        assert_eq!(rig.fake(other).count(), expected, "{id:?} was edited; {other:?} reads");
    }
    assert_eq!(rig.fake(id).in_flight(), 1, "the outdated read was cancelled, not left running");
    let calls = rig.fake(id).calls();
    assert_ne!(calls[0].settings, calls[1].settings, "the second read sees the edit");
    rig.at(seconds(21)).await;
    assert_eq!(of(&rig.service.snapshot().views, id).state, ViewState::Loading);
    rig.at(seconds(26)).await;
    let shown = rig.service.snapshot();
    assert_eq!(of(&shown.views, id).windows[0].used_pct, Some(22.0), "only the new answer lands");
}

#[tokio::test(start_paused = true)]
async fn the_claude_section_is_watched() {
    editing_replaces_the_read_in_flight(HarnessId::Claude, claude).await;
}

#[tokio::test(start_paused = true)]
async fn the_codex_section_is_watched() {
    editing_replaces_the_read_in_flight(HarnessId::Codex, codex).await;
}

#[tokio::test(start_paused = true)]
async fn the_antigravity_section_is_watched() {
    editing_replaces_the_read_in_flight(HarnessId::Antigravity, antigravity).await;
}

#[tokio::test(start_paused = true)]
async fn the_opencode_section_is_watched() {
    editing_replaces_the_read_in_flight(HarnessId::Opencode, opencode).await;
}

#[tokio::test(start_paused = true)]
async fn the_litellm_section_is_watched() {
    editing_replaces_the_read_in_flight(HarnessId::Litellm, litellm).await;
}
