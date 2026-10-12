use std::path::Path;
use std::sync::Mutex;
use std::thread;

use usage_model::settings::LitellmScope;

use super::*;

const NOW: i64 = 1_800_000_000;
/// Spelled out on purpose: the cap is part of the contract, not an implementation detail.
const CAP: usize = 256 * 1024;

pub(super) fn store_in(dir: &Path) -> SettingsStore {
    SettingsStore::new(dir.join("settings.json"))
}

pub(super) fn names(dir: &Path) -> Vec<String> {
    let mut found: Vec<String> = fs::read_dir(dir)
        .expect("read_dir")
        .map(|entry| entry.expect("entry").file_name().to_string_lossy().into_owned())
        .collect();
    found.sort();
    found
}

#[test]
fn a_missing_file_yields_defaults_without_recovery() {
    let dir = tempfile::tempdir().expect("tempdir");
    let loaded = store_in(dir.path()).load(NOW).expect("load");
    assert_eq!(loaded.settings, Settings::default());
    assert!(loaded.recovered.is_none());
    assert!(names(dir.path()).is_empty(), "loading must not create files");
}

#[test]
fn a_missing_directory_is_the_same_as_a_missing_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = SettingsStore::new(dir.path().join("not-yet").join("settings.json"));
    assert!(store.load(NOW).expect("load").recovered.is_none());
}

#[test]
fn unknown_keys_are_ignored_missing_ones_defaulted_and_ranges_clamped() {
    let dir = tempfile::tempdir().expect("tempdir");
    let json = r#"{"general":{"intervalSecs":5,"futureKey":true},
        "litellm":{"scope":"team","entityId":"team-a"},"somethingNew":{"a":1}}"#;
    fs::write(dir.path().join("settings.json"), json).expect("write");
    let loaded = store_in(dir.path()).load(NOW).expect("load");
    assert!(loaded.recovered.is_none());
    assert_eq!(loaded.settings.general.interval_secs, 30, "clamped by normalize");
    assert_eq!(loaded.settings.litellm.scope, LitellmScope::Team);
    assert_eq!(loaded.settings.litellm.entity_id.as_deref(), Some("team-a"));
    assert!(loaded.settings.codex.enabled, "missing sections fall back to defaults");
}

#[test]
fn a_utf8_byte_order_mark_is_accepted() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut bytes = b"\xEF\xBB\xBF".to_vec();
    bytes.extend_from_slice(br#"{"general":{"intervalSecs":120}}"#);
    fs::write(dir.path().join("settings.json"), bytes).expect("write");
    let loaded = store_in(dir.path()).load(NOW).expect("load");
    assert_eq!(loaded.settings.general.interval_secs, 120);
    assert!(loaded.recovered.is_none());
}

#[test]
fn invalid_json_is_set_aside_with_its_bytes_intact() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("settings.json"), b"{ not json").expect("write");
    let loaded = store_in(dir.path()).load(NOW).expect("load");
    assert_eq!(loaded.settings, Settings::default());
    let aside = loaded.recovered.expect("recovery must be reported");
    assert_eq!(aside, "settings.corrupt-1800000000.json");
    assert_eq!(fs::read(dir.path().join(&aside)).expect("read"), b"{ not json");
    assert!(!dir.path().join("settings.json").exists());
}

#[test]
fn content_that_is_not_a_settings_document_is_set_aside() {
    let cases = [
        "",
        "[]",
        "null",
        "42",
        "\"text\"",
        "\u{0}\u{0}",
        r#"{"general":{"intervalSecs":"fast"}}"#,
        r#"{"litellm":{"scope":"galaxy"}}"#,
        r#"{"litellm":{"tls":{"mode":"carrier-pigeon"}}}"#,
    ];
    for case in cases {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("settings.json"), case).expect("write");
        let loaded = store_in(dir.path()).load(NOW).expect("load");
        assert!(loaded.recovered.is_some(), "{case:?} must be reported as corrupt");
        assert_eq!(loaded.settings, Settings::default(), "{case:?}");
        assert_eq!(
            fs::read(dir.path().join(loaded.recovered.expect("name"))).expect("kept"),
            case.as_bytes()
        );
    }
}

#[test]
fn an_oversized_file_is_set_aside_without_being_parsed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let padding = "a".repeat(CAP);
    let json = format!(r#"{{"general":{{"intervalSecs":120}},"pad":"{padding}"}}"#);
    fs::write(dir.path().join("settings.json"), &json).expect("write");
    let loaded = store_in(dir.path()).load(NOW).expect("load");
    assert!(loaded.recovered.is_some());
    assert_eq!(loaded.settings.general.interval_secs, 60, "the oversized content is not used");
}

#[test]
fn the_size_cap_is_256_kib_to_the_byte() {
    let head = r#"{"general":{"intervalSecs":120},"pad":""#;
    let tail = r#""}"#;
    for (extra, loads) in [(0, true), (1, false)] {
        let dir = tempfile::tempdir().expect("tempdir");
        let padding = "a".repeat(CAP + extra - head.len() - tail.len());
        let json = format!("{head}{padding}{tail}");
        assert_eq!(json.len(), CAP + extra);
        fs::write(dir.path().join("settings.json"), json).expect("write");
        let loaded = store_in(dir.path()).load(NOW).expect("load");
        assert_eq!(loaded.recovered.is_none(), loads, "{} bytes", CAP + extra);
        let interval = if loads { 120 } else { 60 };
        assert_eq!(loaded.settings.general.interval_secs, interval, "{} bytes", CAP + extra);
    }
}

#[test]
fn quarantine_never_overwrites_an_earlier_copy() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = store_in(dir.path());
    for round in 0..3_u8 {
        fs::write(dir.path().join("settings.json"), [b'{', b'x', round]).expect("write");
        store.load(NOW).expect("load");
    }
    assert_eq!(
        names(dir.path()),
        [
            "settings.corrupt-1800000000-1.json",
            "settings.corrupt-1800000000-2.json",
            "settings.corrupt-1800000000.json",
        ]
    );
    assert_eq!(
        fs::read(dir.path().join("settings.corrupt-1800000000.json")).expect("read"),
        [b'{', b'x', 0]
    );
}

#[test]
fn when_the_file_cannot_be_set_aside_loading_fails_and_the_file_stays() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("settings.json"), b"{ not json").expect("write");
    fs::write(dir.path().join(format!("settings.corrupt-{NOW}.json")), b"old").expect("write");
    for n in 1..QUARANTINE_NAMES {
        fs::write(dir.path().join(format!("settings.corrupt-{NOW}-{n}.json")), b"old")
            .expect("write");
    }
    let error = store_in(dir.path()).load(NOW).err().expect("must fail");
    assert_eq!(error.code, ErrorCode::SettingsUnreadable);
    assert_eq!(fs::read(dir.path().join("settings.json")).expect("kept"), b"{ not json");
}

#[test]
fn a_path_that_cannot_be_read_is_set_aside_after_the_retries() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::create_dir(dir.path().join("settings.json")).expect("dir");
    let store = store_in(dir.path());
    // The real read of a directory, without the real waiting (that is pinned in `retry_tests`).
    let loaded = store.load_with(NOW, || store.read(), |_| {}).expect("load");
    let aside = loaded.recovered.expect("reported");
    assert!(dir.path().join(aside).is_dir());
    assert!(!dir.path().join("settings.json").exists());
}

#[test]
fn save_round_trips_and_leaves_no_temp_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = store_in(dir.path());
    let mut settings = Settings::default();
    settings.general.interval_secs = 90;
    settings.litellm.base_url = Some("https://gateway.example.dev/litellm".into());
    store.save_then(&settings, || {}).expect("save");
    assert_eq!(names(dir.path()), ["settings.json"]);
    assert_eq!(store.load(NOW).expect("load").settings, settings);
}

#[test]
fn save_creates_missing_directories() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = SettingsStore::new(dir.path().join("a").join("b").join("settings.json"));
    store.save_then(&Settings::default(), || {}).expect("save");
    assert!(dir.path().join("a").join("b").join("settings.json").is_file());
}

#[test]
fn a_failed_write_reports_an_error_and_skips_the_commit() {
    let dir = tempfile::tempdir().expect("tempdir");
    let blocker = dir.path().join("blocker");
    fs::write(&blocker, b"a file where a directory is needed").expect("write");
    let store = SettingsStore::new(blocker.join("settings.json"));
    let mut committed = false;
    let error = store.save_then(&Settings::default(), || committed = true).expect_err("must fail");
    assert_eq!(error.code, ErrorCode::Internal);
    assert!(error.retryable);
    assert_eq!(error.params.get("where").map(String::as_str), Some("settings_write"));
    assert!(!committed, "a failed write must not change the in-memory state");
}

#[test]
fn the_commit_runs_while_the_write_lock_is_still_held() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = store_in(dir.path());
    let mut held = false;
    let held_ref = &mut held;
    store
        .save_then(&Settings::default(), || *held_ref = store.write.try_lock().is_err())
        .expect("save");
    assert!(held, "released before the commit: a later save could commit first");
    assert!(store.write.try_lock().is_ok(), "and released once the save is over");
}

#[test]
fn concurrent_saves_leave_the_file_and_the_commit_order_in_agreement() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = store_in(dir.path());
    let order = Mutex::new(Vec::new());
    thread::scope(|scope| {
        for worker in 0..8_u32 {
            let (store, order) = (&store, &order);
            scope.spawn(move || {
                for step in 0..10_u32 {
                    let mut settings = Settings::default();
                    settings.general.interval_secs = 30 + worker * 10 + step;
                    let secs = settings.general.interval_secs;
                    store.save_then(&settings, || lock(order).push(secs)).expect("save");
                }
            });
        }
    });
    let last_committed = *lock(&order).last().expect("some commit");
    let on_disk = store.load(NOW).expect("load").settings.general.interval_secs;
    assert_eq!(on_disk, last_committed, "the file must hold the last committed value");
    assert_eq!(lock(&order).len(), 80);
}
