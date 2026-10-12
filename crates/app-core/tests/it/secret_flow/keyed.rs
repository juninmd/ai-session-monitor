//! Each key-based balance provider: saving its key refreshes only its own harness.

use usage_model::{HarnessId, SecretKind};

use super::KEY;
use crate::rig::{rig, seconds, settle};

#[tokio::test(start_paused = true)]
async fn the_deepinfra_key_refreshes_only_its_own_harness() {
    let rig = rig().start().await;
    rig.at(seconds(1)).await;

    let status = rig.service.set_secret(SecretKind::Deepinfra, KEY.to_owned()).expect("stored");
    settle().await;
    assert!(status.deepinfra);
    assert_eq!(rig.fake(HarnessId::Deepinfra).count(), 2);
    assert_eq!(rig.fake(HarnessId::Deepinfra).calls()[1].deepinfra_key.as_deref(), Some(KEY));
    for id in HarnessId::ALL.into_iter().filter(|id| *id != HarnessId::Deepinfra) {
        assert_eq!(rig.fake(id).count(), 1, "{id:?} does not read this secret");
    }
}

#[tokio::test(start_paused = true)]
async fn the_moonshot_key_refreshes_only_its_own_harness() {
    let rig = rig().start().await;
    rig.at(seconds(1)).await;

    let status = rig.service.set_secret(SecretKind::Moonshot, KEY.to_owned()).expect("stored");
    settle().await;
    assert!(status.moonshot);
    assert_eq!(rig.fake(HarnessId::Moonshot).count(), 2);
    assert_eq!(rig.fake(HarnessId::Moonshot).calls()[1].moonshot_key.as_deref(), Some(KEY));
    for id in HarnessId::ALL.into_iter().filter(|id| *id != HarnessId::Moonshot) {
        assert_eq!(rig.fake(id).count(), 1, "{id:?} does not read this secret");
    }
}

#[tokio::test(start_paused = true)]
async fn the_novita_key_refreshes_only_its_own_harness() {
    let rig = rig().start().await;
    rig.at(seconds(1)).await;

    let status = rig.service.set_secret(SecretKind::Novita, KEY.to_owned()).expect("stored");
    settle().await;
    assert!(status.novita);
    assert_eq!(rig.fake(HarnessId::Novita).count(), 2);
    assert_eq!(rig.fake(HarnessId::Novita).calls()[1].novita_key.as_deref(), Some(KEY));
    for id in HarnessId::ALL.into_iter().filter(|id| *id != HarnessId::Novita) {
        assert_eq!(rig.fake(id).count(), 1, "{id:?} does not read this secret");
    }
}

#[tokio::test(start_paused = true)]
async fn the_ollama_key_refreshes_only_its_own_harness() {
    let rig = rig().start().await;
    rig.at(seconds(1)).await;

    let status = rig.service.set_secret(SecretKind::Ollama, KEY.to_owned()).expect("stored");
    settle().await;
    assert!(status.ollama);
    assert_eq!(rig.fake(HarnessId::Ollama).count(), 2);
    assert_eq!(rig.fake(HarnessId::Ollama).calls()[1].ollama_key.as_deref(), Some(KEY));
    for id in HarnessId::ALL.into_iter().filter(|id| *id != HarnessId::Ollama) {
        assert_eq!(rig.fake(id).count(), 1, "{id:?} does not read this secret");
    }
}

#[tokio::test(start_paused = true)]
async fn the_minimax_key_refreshes_only_its_own_harness() {
    let rig = rig().start().await;
    rig.at(seconds(1)).await;

    let status = rig.service.set_secret(SecretKind::Minimax, KEY.to_owned()).expect("stored");
    settle().await;
    assert!(status.minimax);
    assert_eq!(rig.fake(HarnessId::Minimax).count(), 2);
    assert_eq!(rig.fake(HarnessId::Minimax).calls()[1].minimax_key.as_deref(), Some(KEY));
    for id in HarnessId::ALL.into_iter().filter(|id| *id != HarnessId::Minimax) {
        assert_eq!(rig.fake(id).count(), 1, "{id:?} does not read this secret");
    }
}
