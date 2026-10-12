use std::fs;

use tempfile::TempDir;
use usage_model::{HarnessId, HarnessView, ViewState};

use super::HistoryStore;
use super::disk::{self, QUARANTINE_NAME};
use super::ledger::Ledger;
use super::ledger_tests::{NOW, window};

fn view(used: f64) -> HarnessView {
    let mut view = HarnessView::with_state(HarnessId::Claude, ViewState::Ready);
    view.windows = vec![window("five_hour", Some(used), Some(NOW + 3600))];
    view
}

fn store(dir: &TempDir) -> HistoryStore {
    HistoryStore::load(dir.path().join("history.json"), NOW)
}

#[test]
fn a_reading_survives_a_restart_once_flushed() {
    let dir = TempDir::new().expect("temp dir");
    let first = store(&dir);
    first.record(&view(42.0), NOW);
    first.flush(NOW);
    let again = store(&dir);
    let epochs = again.query(HarnessId::Claude, 8).epochs;
    assert_eq!(epochs[0].points[0].used_pct, 42.0);
}

#[test]
fn writes_are_debounced_to_one_a_minute() {
    let dir = TempDir::new().expect("temp dir");
    let file = dir.path().join("history.json");
    let store = store(&dir);
    store.record(&view(10.0), NOW + 1);
    store.flush_if_due(NOW + 2);
    assert!(!file.exists(), "a write inside the gap");
    store.record(&view(11.0), NOW + 30);
    store.flush_if_due(NOW + 61);
    assert!(file.exists());
    fs::remove_file(&file).expect("remove");
    store.flush_if_due(NOW + 62);
    assert!(!file.exists(), "nothing pending, nothing written");
    store.record(&view(12.0), NOW + 70);
    store.flush_if_due(NOW + 80);
    assert!(!file.exists(), "a second write inside the gap");
    store.flush(NOW + 80);
    assert!(file.exists(), "shutdown flushes regardless of the gap");
}

#[test]
fn failed_stale_and_empty_views_record_nothing() {
    let dir = TempDir::new().expect("temp dir");
    let store = store(&dir);
    let mut stale = view(50.0);
    stale.stale = true;
    let mut errored = view(50.0);
    errored.state = ViewState::Error;
    for v in [stale, errored, HarnessView::loading(HarnessId::Claude)] {
        store.record(&v, NOW);
    }
    assert!(store.query(HarnessId::Claude, 8).epochs.is_empty());
}

#[test]
fn a_damaged_file_is_set_aside_and_history_starts_empty() {
    let dir = TempDir::new().expect("temp dir");
    let file = dir.path().join("history.json");
    fs::write(&file, b"{ not json \xff secret-looking text").expect("write");
    let store = store(&dir);
    assert!(store.query(HarnessId::Claude, 8).epochs.is_empty());
    assert!(!file.exists());
    assert!(dir.path().join(QUARANTINE_NAME).exists());
    store.record(&view(5.0), NOW);
    store.flush(NOW);
    assert!(file.exists(), "history works again after the recovery");
}

#[test]
fn an_oversized_or_foreign_version_file_is_also_set_aside() {
    let dir = TempDir::new().expect("temp dir");
    let file = dir.path().join("history.json");
    fs::write(&file, vec![b' '; 3 << 20]).expect("write");
    assert!(disk::load(&file, NOW).is_empty());
    assert!(dir.path().join(QUARANTINE_NAME).exists());
    fs::write(&file, br#"{"version":99,"series":[]}"#).expect("write");
    assert!(disk::load(&file, NOW).is_empty());
    assert!(!file.exists());
}

#[test]
fn a_ledger_over_the_byte_limit_is_trimmed_oldest_first_before_writing() {
    let mut ledger = Ledger::default();
    for n in 0..500_u32 {
        let w = window("w", Some(f64::from(n % 100)), Some(NOW + 9_999));
        ledger.record(HarnessId::Claude, &[w], NOW + i64::from(n));
    }
    let full = serde_json::to_vec(&ledger).expect("json").len();
    let bytes = disk::encode(&mut ledger, full / 2).expect("encoded");
    assert!(bytes.len() <= full / 2);
    assert!(ledger.point_count() < 500);
    let newest = ledger.epochs(HarnessId::Claude, 8).epochs[0].points.last().map(|p| p.t);
    assert_eq!(newest, Some(NOW + 499));
}

#[test]
fn clearing_forgets_memory_and_files_and_a_missing_file_is_fine() {
    let dir = TempDir::new().expect("temp dir");
    let store = store(&dir);
    assert!(store.clear(NOW).is_ok());
    store.record(&view(30.0), NOW);
    store.flush(NOW);
    fs::write(dir.path().join(QUARANTINE_NAME), b"x").expect("write");
    assert!(store.clear(NOW).is_ok());
    assert!(store.query(HarnessId::Claude, 8).epochs.is_empty());
    assert!(!dir.path().join("history.json").exists());
    assert!(!dir.path().join(QUARANTINE_NAME).exists());
}

#[test]
fn the_stored_file_holds_only_ids_times_and_percentages() {
    let dir = TempDir::new().expect("temp dir");
    let store = store(&dir);
    let mut v = view(42.0);
    v.plan = Some("Max-secret-plan".into());
    v.windows[0].label = Some("team-secret".into());
    v.meta.insert("host".into(), "gateway.secret".into());
    store.record(&v, NOW);
    store.flush(NOW);
    let text = fs::read_to_string(dir.path().join("history.json")).expect("read");
    assert!(!text.contains("secret"), "{text}");
}
