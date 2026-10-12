//! A settings file that cannot be used is set aside and announced, never silently replaced.

use std::fs;
use std::path::PathBuf;

use app_core::NotificationKind;
use usage_model::{ErrorCode, Settings};

use crate::rig::{Rig, rig, seconds};
use crate::views::BASE;

const GARBAGE: &[u8] = b"{ \"general\": { oops";

fn defaults() -> Settings {
    let mut settings = Settings::default();
    settings.normalize();
    settings
}

fn copy_name() -> String {
    format!("settings.corrupt-{BASE}.json")
}

fn copy_path(rig: &Rig) -> PathBuf {
    rig.data_dir.join(copy_name())
}

#[tokio::test(start_paused = true)]
async fn an_unusable_file_is_set_aside_and_announced_first_thing_in_run() {
    let mut rig = rig().raw_settings(GARBAGE).build();

    assert!(!rig.settings_file().exists(), "moved, not overwritten");
    assert_eq!(
        fs::read(copy_path(&rig)).unwrap_or_default(),
        GARBAGE,
        "the user's bytes are intact"
    );
    assert_eq!(rig.service.settings(), defaults());
    assert!(rig.sink.notices().is_empty(), "run announces it, where a sink is listening");

    rig.run().await;
    let notices = rig.sink.notices();
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].kind, NotificationKind::SettingsRecovered);
    assert!(notices[0].body.contains(&copy_name()), "it names the copy: {}", notices[0].body);
    assert_eq!(rig.sink.kinds()[..2], ["notice", "snapshot"], "before the first snapshot");
}

#[tokio::test(start_paused = true)]
async fn a_settings_path_that_cannot_be_read_is_set_aside_and_announced_too() {
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("tempdir: {e}"));
    let stuck = dir.path().join("data").join("settings.json");
    fs::create_dir_all(&stuck).unwrap_or_else(|e| panic!("mkdir: {e}"));

    // Real waiting happens here: the read is retried for about a second and a half first.
    let mut rig = rig().in_dir(dir.path()).build();

    assert!(copy_path(&rig).is_dir(), "the unreadable entry was moved aside, not deleted");
    assert!(!rig.settings_file().exists());
    assert_eq!(rig.service.settings(), defaults());
    rig.run().await;
    let notices = rig.sink.notices();
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].kind, NotificationKind::SettingsRecovered);
    assert_eq!(rig.sink.kinds()[..2], ["notice", "snapshot"]);
}

#[tokio::test(start_paused = true)]
async fn every_kind_of_non_settings_content_is_recovered() {
    let documents: [&[u8]; 6] =
        [b"", b"null", b"[1, 2]", b"\"text\"", b"{\"general\": 5}", b"\xff\xfe\x00"];
    for document in documents {
        let mut rig = rig().raw_settings(document).build();
        rig.run().await;

        let copy = fs::read(copy_path(&rig)).unwrap_or_default();
        assert_eq!(copy, document, "kept: {document:?}");
        assert_eq!(rig.sink.notices().len(), 1, "announced: {document:?}");
        assert_eq!(rig.service.settings(), defaults(), "defaults apply: {document:?}");
    }
}

#[tokio::test(start_paused = true)]
async fn a_good_file_is_not_announced_and_unknown_keys_are_ignored() {
    let good = br#"{"general":{"intervalSecs":120,"futureKey":1},"extra":true}"#;
    let rig = rig().raw_settings(good).start().await;
    rig.after(seconds(600)).await;

    assert_eq!(rig.service.settings().general.interval_secs, 120);
    assert!(rig.sink.notices().is_empty());
    assert!(!copy_path(&rig).exists());
}

#[tokio::test(start_paused = true)]
async fn the_announcement_is_made_once() {
    let rig = rig().raw_settings(GARBAGE).start().await;
    rig.edit(|s| s.general.interval_secs = 90);
    rig.after(seconds(3600)).await;

    assert_eq!(rig.sink.notices().len(), 1);
}

#[tokio::test(start_paused = true)]
async fn saving_after_a_recovery_never_touches_the_set_aside_copy() {
    let rig = rig().raw_settings(GARBAGE).build();
    let saved = rig.edit(|s| s.general.interval_secs = 90);

    assert_eq!(rig.saved_settings(), saved, "a new file now stands at the old path");
    assert_eq!(fs::read(copy_path(&rig)).unwrap_or_default(), GARBAGE);
}

#[tokio::test(start_paused = true)]
async fn a_file_that_cannot_be_set_aside_stops_the_start_and_stays_where_it_is() {
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("tempdir: {e}"));
    let mut builder = rig().in_dir(dir.path()).raw_settings(GARBAGE).file(&copy_name(), b"older");
    for n in 1..100 {
        builder = builder.file(&format!("settings.corrupt-{BASE}-{n}.json"), b"older");
    }

    let error = builder.try_build().err().expect("every copy name is taken");
    assert_eq!(error.code, ErrorCode::SettingsUnreadable);
    assert_eq!(
        fs::read(dir.path().join("data").join("settings.json")).unwrap_or_default(),
        GARBAGE
    );
}
