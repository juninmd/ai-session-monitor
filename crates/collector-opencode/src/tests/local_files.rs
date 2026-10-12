//! The local reader against files in odd states: WAL with writers, locks, corruption, missing
//! tables and hostile views. Each case ends in facts or a reason, never a panic or a write to the
//! database. SQLite may still add empty -wal/-shm sidecars to a WAL database opened read-only.

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant, UNIX_EPOCH};

use rusqlite::Connection;

use super::fake_db::{
    CANARY_CREDENTIAL, CANARY_PROMPT, FakeDb, NOW, NOW_MS, assistant_json, populate,
};
use super::local_support::{facts, populated, read, reason};
use super::{ctx, settings, snapshot};
use crate::local::{self, Facts, Limits, Outcome, Reason, db};

/// Older than any file the OS creates during a test run: `NOW` itself lies in the future.
const LONG_AGO: i64 = 1_700_000_000;

fn set_mtime(path: &Path, secs: i64) {
    let when = UNIX_EPOCH + Duration::from_secs(u64::try_from(secs).unwrap());
    fs::OpenOptions::new().write(true).open(path).unwrap().set_modified(when).unwrap();
}

#[tokio::test]
async fn read_at_is_the_file_time_and_never_after_the_clock() {
    let fake = populated();
    set_mtime(&fake.path(), NOW - 600);
    assert_eq!(facts(read(&fake.path()).await).read_at, Some(NOW - 600));
    set_mtime(&fake.path(), NOW + 90_000);
    assert_eq!(facts(read(&fake.path()).await).read_at, Some(NOW));
}

#[test]
fn the_wal_counts_when_it_is_newer_than_the_main_file() {
    let fake = populated();
    let wal = fake.dir().join("opencode.db-wal");
    // Only the modification time of this fabricated log matters here.
    fs::write(&wal, b"x").unwrap();
    set_mtime(&fake.path(), NOW - 5_000);
    set_mtime(&wal, NOW - 100);
    assert_eq!(db::last_write(&fake.path(), NOW), Ok(Some(NOW - 100)));
    set_mtime(&wal, NOW - 9_000);
    assert_eq!(db::last_write(&fake.path(), NOW), Ok(Some(NOW - 5_000)));
}

#[test]
fn an_empty_wal_does_not_count_as_a_write() {
    let fake = populated();
    let wal = fake.dir().join("opencode.db-wal");
    // A read-only open of a WAL database creates exactly this: a log that holds no commits.
    fs::write(&wal, b"").unwrap();
    set_mtime(&fake.path(), NOW - 5_000);
    set_mtime(&wal, NOW - 100);
    assert_eq!(db::last_write(&fake.path(), NOW), Ok(Some(NOW - 5_000)));
}

#[tokio::test]
async fn a_wal_database_with_a_live_writer_reads_committed_rows_and_never_writes() {
    let fake = FakeDb::new();
    let writer = fake.writer();
    writer.pragma_update(None, "journal_mode", "WAL").unwrap();
    populate(&fake, &writer);
    // An open, uncommitted transaction: its row must stay invisible and must not block the reader.
    writer.execute_batch("BEGIN").unwrap();
    fake.add_assistant(&writer, NOW_MS - 500, 1_000.0, [1_000_000, 0, 0]);

    let wal = fake.dir().join("opencode.db-wal");
    let (db_before, wal_before) = (fs::read(fake.path()).unwrap(), fs::read(&wal).unwrap());
    assert!(!wal_before.is_empty(), "the committed rows live in the log, not the main file");
    let got = facts(read(&fake.path()).await);
    assert_eq!((got.cost_5h, got.cost_7d, got.cost_30d), (Some(0.3), Some(1.5), Some(6.3)));
    assert_eq!(fs::read(fake.path()).unwrap(), db_before, "the main file is never written");
    assert_eq!(fs::read(&wal).unwrap(), wal_before, "the log is never written or checkpointed");
    writer.execute_batch("ROLLBACK").unwrap();
}

#[tokio::test]
async fn a_wal_database_whose_writer_has_gone_still_reads() {
    let fake = FakeDb::new();
    let writer = fake.writer();
    writer.pragma_update(None, "journal_mode", "WAL").unwrap();
    populate(&fake, &writer);
    drop(writer);
    let before = fs::read(fake.path()).unwrap();
    assert_eq!(facts(read(&fake.path()).await).cost_30d, Some(6.3));
    assert_eq!(fs::read(fake.path()).unwrap(), before);
}

#[tokio::test]
async fn a_closed_wal_database_gains_empty_sidecars_but_read_at_stays_put() {
    let fake = FakeDb::new();
    let writer = fake.writer();
    writer.pragma_update(None, "journal_mode", "WAL").unwrap();
    populate(&fake, &writer);
    drop(writer);
    set_mtime(&fake.path(), LONG_AGO);
    let bytes = fs::read(fake.path()).unwrap();
    let before = snapshot(fake.dir());
    assert_eq!(before.keys().collect::<Vec<_>>(), ["opencode.db"], "clean close, no sidecars");

    let first = facts(read(&fake.path()).await).read_at;
    let second = facts(read(&fake.path()).await).read_at;

    // The log SQLite creates holds no commits, so it must not make old data look freshly written.
    assert_eq!((first, second), (Some(LONG_AGO), Some(LONG_AGO)));
    // Pinned so the docs wording stays honest: a read-only open cannot checkpoint, so these stay.
    let after = snapshot(fake.dir());
    let names: Vec<_> = after.keys().collect();
    assert_eq!(names, ["opencode.db", "opencode.db-shm", "opencode.db-wal"]);
    assert_eq!(after["opencode.db-wal"].0, 0, "the log is empty");
    assert_eq!(fs::read(fake.path()).unwrap(), bytes, "the main file is never written");
}

#[tokio::test]
async fn reads_stay_consistent_while_a_writer_keeps_committing() {
    let fake = FakeDb::new();
    let writer = fake.writer();
    writer.pragma_update(None, "journal_mode", "WAL").unwrap();
    let writing = std::thread::spawn(move || {
        for n in 0..150_i64 {
            let data = assistant_json(NOW_MS - 1_000 - n, 0.01, [1, 1, 1]);
            let insert =
                "INSERT INTO session_message VALUES (?1, 's', 'assistant', ?2, ?3, ?3, ?4)";
            writer.execute(insert, rusqlite::params![format!("w{n}"), n, NOW_MS, data]).unwrap();
            std::thread::sleep(Duration::from_millis(2));
        }
    });
    let mut last = 0.0;
    for _ in 0..8 {
        let cost = facts(read(&fake.path()).await).cost_5h.unwrap();
        assert!(cost >= last, "{cost} < {last}: a read went backwards");
        last = cost;
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    writing.join().unwrap();
    assert_eq!(facts(read(&fake.path()).await).cost_5h, Some(1.5));
}

#[tokio::test]
async fn a_busy_writer_is_waited_for_briefly_then_reported_as_locked() {
    let fake = populated();
    let writer = fake.writer();
    writer.execute_batch("BEGIN EXCLUSIVE").unwrap();
    let started = Instant::now();
    assert_eq!(reason(read(&fake.path()).await), Reason::Locked);
    assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
    writer.execute_batch("ROLLBACK").unwrap();
    assert_eq!(facts(read(&fake.path()).await).cost_30d, Some(6.3));
}

#[tokio::test]
async fn a_long_scan_is_interrupted_when_the_budget_runs_out() {
    let fake = FakeDb::new();
    let writer = fake.writer();
    writer.execute_batch("BEGIN").unwrap();
    for n in 0..3_000 {
        fake.add_assistant(&writer, NOW_MS - 1_000 - n, 0.01, [1, 1, 1]);
    }
    writer.execute_batch("COMMIT").unwrap();
    drop(writer);
    let tight = Limits { busy: Duration::from_millis(100), budget: Duration::ZERO };
    assert_eq!(reason(local::read_path(fake.path(), NOW, tight).await), Reason::Timeout);
    assert_eq!(facts(read(&fake.path()).await).cost_5h, Some(30.0));
}

#[tokio::test]
async fn blocking_work_that_hangs_or_panics_becomes_a_notice() {
    let started = Instant::now();
    let hung = local::bounded(Duration::from_millis(100), || {
        std::thread::sleep(Duration::from_millis(800));
        Outcome::Facts(Facts::default())
    })
    .await;
    assert_eq!(reason(hung), Reason::Timeout);
    assert!(started.elapsed() < Duration::from_millis(700), "{:?}", started.elapsed());
    let boom = local::bounded(Duration::from_secs(5), || panic!("boom")).await;
    assert_eq!(reason(boom), Reason::Other);
}

#[tokio::test]
async fn files_that_are_not_databases_are_reported_as_such() {
    for bytes in [vec![0xAB_u8; 8_192], b"SQLite format 3\0 and then garbage".repeat(200)] {
        let fake = FakeDb::bare();
        fs::write(fake.path(), &bytes).unwrap();
        assert_eq!(reason(read(&fake.path()).await), Reason::NotADatabase);
    }
    let fake = populated();
    let valid = fs::read(fake.path()).unwrap();
    fs::write(fake.path(), &valid[..150]).unwrap();
    assert_eq!(reason(read(&fake.path()).await), Reason::NotADatabase);
}

#[tokio::test]
async fn an_empty_file_has_no_schema() {
    let fake = FakeDb::bare();
    fs::write(fake.path(), b"").unwrap();
    assert_eq!(reason(read(&fake.path()).await), Reason::Schema);
}

#[tokio::test]
async fn a_missing_file_or_a_directory_in_its_place_is_not_found() {
    let fake = FakeDb::bare();
    assert_eq!(reason(read(&fake.path()).await), Reason::NotFound);
    fs::create_dir(fake.path()).unwrap();
    assert_eq!(reason(read(&fake.path()).await), Reason::NotFound);
}

#[tokio::test]
async fn schema_drift_is_reported_never_a_crash() {
    let drifts = [
        "CREATE TABLE session_v2 (id TEXT, data TEXT, type TEXT)",
        "CREATE TABLE session_message (id TEXT, type TEXT, body TEXT)",
        "CREATE TABLE session_message (id TEXT, kind TEXT, data TEXT)",
    ];
    for ddl in drifts {
        let fake = FakeDb::bare();
        Connection::open(fake.path()).unwrap().execute_batch(ddl).unwrap();
        assert_eq!(reason(read(&fake.path()).await), Reason::Schema, "{ddl}");
    }
}

#[tokio::test]
async fn a_view_pointing_at_the_credential_table_is_blocked_by_the_fence() {
    let fake = FakeDb::bare();
    let conn = Connection::open(fake.path()).unwrap();
    conn.execute_batch(&format!(
        "CREATE TABLE credential (id TEXT, secret TEXT);
         INSERT INTO credential VALUES ('c1', '{CANARY_CREDENTIAL}');
         CREATE VIEW session_message AS SELECT id, id AS session_id, 'assistant' AS type,
           0 AS seq, 0 AS time_created, 0 AS time_updated, secret AS data FROM credential;"
    ))
    .unwrap();
    drop(conn);
    let outcome = read(&fake.path()).await;
    assert!(!format!("{outcome:?}").contains(CANARY_CREDENTIAL));
    assert_eq!(reason(outcome), Reason::Schema);
}

#[tokio::test]
async fn reading_a_rollback_journal_database_changes_nothing_and_leaves_no_files_behind() {
    let fake = populated();
    let before_files = snapshot(fake.dir());
    let before_bytes = fs::read(fake.path()).unwrap();
    facts(read(&fake.path()).await);
    assert_eq!(snapshot(fake.dir()), before_files);
    assert_eq!(fs::read(fake.path()).unwrap(), before_bytes);
}

#[tokio::test]
async fn nothing_from_the_credential_tables_or_the_message_text_reaches_the_output() {
    let fake = populated();
    let text = format!("{:?}", read(&fake.path()).await);
    for canary in [CANARY_CREDENTIAL, CANARY_PROMPT] {
        assert!(!text.contains(canary), "{canary}");
    }
    assert!(
        assistant_json(NOW_MS, 1.0, [1, 1, 1]).contains(CANARY_PROMPT),
        "the rows do carry prompts"
    );
}

#[tokio::test]
async fn the_production_path_gives_up_on_a_locked_file_after_a_short_wait() {
    let fake = populated();
    let writer = fake.writer();
    writer.execute_batch("BEGIN EXCLUSIVE").unwrap();
    let home = tempfile::tempdir().unwrap();
    let started = Instant::now();
    let outcome = local::read(&settings(true, Some(fake.dir())), &ctx(home.path())).await;
    assert_eq!(reason(outcome), Reason::Locked);
    assert!(started.elapsed() < Duration::from_secs(4), "{:?}", started.elapsed());
    writer.execute_batch("ROLLBACK").unwrap();
}
