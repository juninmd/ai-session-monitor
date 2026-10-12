//! A secret or shim change makes the data in flight outdated: it is replaced, never waited for,
//! and the old answer can never land after the new one.

use std::path::Path;

use usage_model::{HarnessId, SecretKind, ViewState};

use crate::collector::{FakeCollector, Step};
use crate::rig::{Rig, RigBuilder, rig, seconds, settle};
use crate::views::{of, ready};

const KEY: &str = "sk-fake-91c0d4aa7e52";
const LITELLM: HarnessId = HarnessId::Litellm;

/// The first read hangs (a source that never answers); every later one answers at once.
fn hung_then_ready(id: HarnessId) -> RigBuilder {
    let plan = move |n: usize| if n == 0 { Step::Hang } else { Step::View(ready(id, 22.0)) };
    rig().settings(|s| s.claude.shim_consent = true).collector(FakeCollector::new(id, plan))
}

async fn trigger_replaces_the_hung_read(id: HarnessId, trigger: impl FnOnce(&Rig)) {
    let rig = hung_then_ready(id).start().await;
    rig.at(seconds(5)).await;
    assert_eq!(rig.fake(id).in_flight(), 1, "control: the first read is stuck");

    trigger(&rig);
    settle().await;

    assert_eq!(rig.fake(id).count(), 2, "read again at once, not after the 30 s timeout");
    assert_eq!(rig.fake(id).in_flight(), 0, "and the stuck one is gone");
    let shown = rig.service.snapshot();
    assert_eq!(of(&shown.views, id).windows[0].used_pct, Some(22.0));
    for other in HarnessId::ALL.into_iter().filter(|other| *other != id) {
        assert_eq!(rig.fake(other).count(), 1, "{other:?} has nothing to do with it");
    }
}

#[tokio::test(start_paused = true)]
async fn a_key_saved_during_a_hung_read_is_used_at_once() {
    trigger_replaces_the_hung_read(LITELLM, |rig| {
        rig.service.set_secret(SecretKind::Litellm, KEY.to_owned()).expect("stored");
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn a_key_removed_during_a_hung_read_is_honoured_at_once() {
    trigger_replaces_the_hung_read(HarnessId::Opencode, |rig| {
        rig.service.clear_secret(SecretKind::Opencode).expect("cleared");
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn installing_the_shim_during_a_hung_read_reads_the_file_at_once() {
    trigger_replaces_the_hung_read(HarnessId::Claude, |rig| {
        rig.service.install_claude_shim(Path::new("shim.exe")).expect("installed");
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn removing_the_shim_during_a_hung_read_reads_again_at_once() {
    trigger_replaces_the_hung_read(HarnessId::Claude, |rig| {
        rig.service.uninstall_claude_shim().expect("removed");
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn the_answer_to_the_old_key_never_lands_after_the_new_one() {
    // The first read (old key) answers after 20 s; the one forced by the new key answers at once.
    let plan = |n: usize| match n {
        0 => Step::After(seconds(20), ready(LITELLM, 11.0)),
        _ => Step::View(ready(LITELLM, 22.0)),
    };
    let rig = rig().collector(FakeCollector::new(LITELLM, plan)).start().await;
    rig.at(seconds(5)).await;

    rig.service.set_secret(SecretKind::Litellm, KEY.to_owned()).expect("stored");
    settle().await;
    assert_eq!(of(&rig.service.snapshot().views, LITELLM).state, ViewState::Ready);

    rig.at(seconds(25)).await;
    let shown = rig.service.snapshot();
    assert_eq!(of(&shown.views, LITELLM).windows[0].used_pct, Some(22.0), "the old answer is gone");
    let ever_showed_old = rig.sink.snapshots().iter().any(|(_, snapshot)| {
        of(&snapshot.views, LITELLM).windows.first().is_some_and(|w| w.used_pct == Some(11.0))
    });
    assert!(!ever_showed_old);
}
