//! A settings save that cannot be written changes nothing anywhere: not the settings in memory,
//! not the tabs, not the events, not the schedule.

use usage_model::{ErrorCode, HarnessId, ViewState};

use crate::rig::{rig, seconds, secs, settle};
use crate::views::of;

const CODEX: HarnessId = HarnessId::Codex;

#[tokio::test(start_paused = true)]
async fn a_save_that_cannot_be_written_changes_nothing_anywhere() {
    let rig = rig().start().await;
    rig.at(seconds(5)).await;
    let before = rig.service.settings();
    // A directory where the file goes: the atomic replace of it fails.
    std::fs::remove_file(rig.settings_file()).unwrap_or_else(|e| panic!("rm: {e}"));
    std::fs::create_dir(rig.settings_file()).unwrap_or_else(|e| panic!("mkdir: {e}"));

    let mut wanted = before.clone();
    wanted.codex.enabled = false;
    wanted.general.interval_secs = 30;
    let error = rig.service.save_settings(wanted).expect_err("the file cannot be replaced");
    settle().await;

    assert_eq!((error.code, error.retryable), (ErrorCode::Internal, true));
    assert_eq!(error.params.get("where").map(String::as_str), Some("settings_write"));
    assert_eq!(rig.service.settings(), before, "memory still holds the old settings");
    assert_ne!(of(&rig.service.snapshot().views, CODEX).state, ViewState::Disabled);
    assert!(rig.sink.settings().is_empty(), "nothing was announced");
    assert_eq!(rig.fake(CODEX).count(), 1, "and nothing was read again");

    rig.at(seconds(100)).await;
    assert_eq!(rig.fake(CODEX).offsets_ms(rig.start), secs(&[0, 60]), "the old interval holds");
}
