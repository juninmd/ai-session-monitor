//! The read retry rules, driven with scripted reads and a recorded pause, plus the settings
//! version rules that decide what a save rewrites.

use serde_json::Value;
use usage_model::settings::SETTINGS_VERSION;

use super::tests::{names, store_in};
use super::*;

const NOW: i64 = 1_800_000_000;

fn ms(list: &[u64]) -> Vec<Duration> {
    list.iter().map(|n| Duration::from_millis(*n)).collect()
}

fn found() -> Read {
    Read::Found(Box::default())
}

/// A store whose settings file exists, so "it was moved aside" can be seen.
fn store_with_file() -> (tempfile::TempDir, SettingsStore) {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("settings.json"), b"{}").expect("write");
    let store = store_in(dir.path());
    (dir, store)
}

#[test]
fn a_busy_file_is_retried_with_growing_pauses_and_then_loaded() {
    let (dir, store) = store_with_file();
    let (mut reads, mut pauses) = (0, Vec::new());
    let script = || {
        reads += 1;
        if reads <= 5 { Read::Io } else { found() }
    };

    let loaded = store.load_with(NOW, script, |wait| pauses.push(wait)).expect("load");

    assert!(loaded.recovered.is_none(), "a file that opened in the end was never corrupt");
    assert_eq!(pauses, ms(&[100, 200, 300, 400, 500]), "one step longer each time");
    assert_eq!(names(dir.path()), ["settings.json"], "nothing was moved");
}

#[test]
fn a_file_that_stays_unreadable_is_set_aside_only_after_the_whole_budget() {
    let (dir, store) = store_with_file();
    let (mut reads, mut pauses) = (0, Vec::new());

    let loaded = store
        .load_with(
            NOW,
            || {
                reads += 1;
                Read::Io
            },
            |wait| pauses.push(wait),
        )
        .expect("load");

    assert_eq!(reads, 6, "six tries");
    assert_eq!(pauses.len(), 5, "and no pause after the last one");
    assert_eq!(pauses.iter().sum::<Duration>(), Duration::from_millis(1500));
    assert_eq!(loaded.recovered.as_deref(), Some("settings.corrupt-1800000000.json"));
    assert!(!dir.path().join("settings.json").exists());
}

#[test]
fn a_corrupt_file_is_judged_at_once_and_a_missing_one_means_defaults() {
    let (_dir, store) = store_with_file();
    let (mut reads, mut pauses) = (0, Vec::new());
    let loaded = store
        .load_with(
            NOW,
            || {
                reads += 1;
                Read::Corrupt
            },
            |wait| pauses.push(wait),
        )
        .expect("load");
    assert_eq!((reads, pauses.len()), (1, 0), "retrying cannot fix the content");
    assert!(loaded.recovered.is_some());

    let (_dir, store) = store_with_file();
    let (mut reads, mut pauses) = (0, Vec::new());
    let script = || {
        reads += 1;
        if reads == 1 { Read::Io } else { Read::Missing }
    };
    let loaded = store.load_with(NOW, script, |wait| pauses.push(wait)).expect("load");
    assert_eq!(pauses, ms(&[100]), "busy once, then gone");
    assert!(loaded.recovered.is_none(), "a vanished file is a first run, not a corruption");
}

#[cfg(windows)]
#[test]
fn a_file_another_process_holds_for_a_moment_is_loaded_not_set_aside() {
    use std::os::windows::fs::OpenOptionsExt;

    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("settings.json");
    fs::write(&path, br#"{"general":{"intervalSecs":120}}"#).expect("write");
    // Share mode 0: no other open succeeds, like a scanner or backup tool with an exclusive handle.
    let holder = fs::OpenOptions::new().read(true).share_mode(0).open(&path).expect("hold");
    let release = thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        drop(holder);
    });

    let loaded = store_in(dir.path()).load(NOW).expect("load");

    release.join().expect("holder thread");
    assert!(loaded.recovered.is_none(), "a valid file must not be moved aside for a busy moment");
    assert_eq!(loaded.settings.general.interval_secs, 120);
    assert_eq!(names(dir.path()), ["settings.json"]);
}

#[test]
fn a_file_from_another_version_loads_as_this_one_and_is_rewritten_as_this_one() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("settings.json");
    fs::write(&path, br#"{"version":99,"general":{"intervalSecs":120},"futureKey":1}"#)
        .expect("write");
    let store = store_in(dir.path());

    let loaded = store.load(NOW).expect("load");
    assert!(loaded.recovered.is_none());
    assert_eq!(loaded.settings.version, SETTINGS_VERSION);
    assert_eq!(loaded.settings.general.interval_secs, 120, "its values still count");

    store.save_then(&loaded.settings, || {}).expect("save");
    let on_disk: Value = serde_json::from_slice(&fs::read(&path).expect("read")).expect("json");
    assert_eq!(on_disk["version"], SETTINGS_VERSION, "never a version this build cannot read");
    assert!(on_disk.get("futureKey").is_none(), "unknown keys do not survive a rewrite");
}

#[test]
fn a_version_that_is_not_a_number_is_corruption_not_a_guess() {
    for document in [r#"{"version":"two"}"#, r#"{"version":-1}"#, r#"{"version":1.5}"#] {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("settings.json"), document).expect("write");
        let loaded = store_in(dir.path()).load(NOW).expect("load");
        assert!(loaded.recovered.is_some(), "{document}");
        assert_eq!(loaded.settings.version, SETTINGS_VERSION);
    }
}
