//! The read-only token read against fabricated databases, and the fence around it.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rusqlite::hooks::{AuthAction, AuthContext};
use rusqlite::{Connection, ErrorCode};

use super::{FAST, FakeDb, token};
use crate::store::{
    self, KEY, Limits, MAX_VALUE_BYTES, Reason, SELECT_TOKEN, TABLE, authorize, read_token,
};

async fn read(db: &FakeDb) -> Result<Option<String>, Reason> {
    read_token(db.path(), FAST).await
}

#[tokio::test]
async fn the_token_is_read_whether_stored_as_text_or_as_a_blob() {
    let valid = token("valid");
    assert_eq!(read(&FakeDb::with_token(&valid)).await, Ok(Some(valid.clone())));

    let blob = FakeDb::empty();
    blob.writer()
        .execute("INSERT INTO ItemTable VALUES ('cursorAuth/accessToken', ?1)", [valid.as_bytes()])
        .unwrap();
    assert_eq!(read(&blob).await, Ok(Some(valid)));
}

#[tokio::test]
async fn a_json_quoted_value_is_unquoted() {
    let valid = token("valid");
    let db = FakeDb::with_token(&format!("\"{valid}\""));
    assert_eq!(read(&db).await, Ok(Some(valid)));
}

#[tokio::test]
async fn absent_values_are_no_token_rather_than_faults() {
    assert_eq!(read(&FakeDb::absent()).await, Ok(None), "no file");
    assert_eq!(read(&FakeDb::empty()).await, Ok(None), "no row");
    let directory = tempfile::tempdir().unwrap();
    assert_eq!(read_token(directory.path().to_path_buf(), FAST).await, Ok(None), "a directory");
    let huge = FakeDb::with_token(&"a".repeat(usize::try_from(MAX_VALUE_BYTES).unwrap() + 1));
    assert_eq!(read(&huge).await, Ok(None), "over the size cap");
    let at_cap = FakeDb::with_token(&"a".repeat(usize::try_from(MAX_VALUE_BYTES).unwrap()));
    assert!(read(&at_cap).await.unwrap().is_some(), "exactly at the size cap");
}

#[tokio::test]
async fn reading_never_changes_the_file_or_creates_sidecars() {
    let db = FakeDb::with_token(&token("valid"));
    drop(db.writer());
    let before = std::fs::read(db.path()).unwrap();
    for _ in 0..3 {
        read(&db).await.unwrap();
    }
    assert_eq!(std::fs::read(db.path()).unwrap(), before);
    let names: Vec<String> = std::fs::read_dir(db.path().parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["state.vscdb"], "no journal or sidecar is left behind");
}

#[tokio::test]
async fn a_wal_database_is_read_while_the_editor_holds_it_open() {
    let db = FakeDb::with_token(&token("valid"));
    let writer = db.writer();
    writer.execute_batch("PRAGMA journal_mode=WAL").unwrap();
    writer
        .execute("INSERT INTO ItemTable VALUES ('cursorAuth/accessToken', ?1)", [token("no_exp")])
        .unwrap();
    assert_eq!(read(&db).await, Ok(Some(token("no_exp"))), "the newest commit is the one read");
    drop(writer);
}

#[tokio::test]
async fn a_busy_writer_is_waited_for_briefly_then_reported_as_locked() {
    let db = FakeDb::with_token(&token("valid"));
    let writer = db.writer();
    writer.execute_batch("BEGIN EXCLUSIVE").unwrap();
    let started = Instant::now();
    assert_eq!(read(&db).await, Err(Reason::Locked));
    assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
    writer.execute_batch("ROLLBACK").unwrap();
    assert!(read(&db).await.unwrap().is_some());
}

#[tokio::test]
async fn a_layout_change_is_a_schema_reason() {
    let renamed = FakeDb::absent();
    Connection::open(renamed.path())
        .unwrap()
        .execute_batch("CREATE TABLE Items (key TEXT, value BLOB)")
        .unwrap();
    assert_eq!(read(&renamed).await, Err(Reason::Schema));

    let columns = FakeDb::absent();
    Connection::open(columns.path())
        .unwrap()
        .execute_batch("CREATE TABLE ItemTable (name TEXT, data BLOB)")
        .unwrap();
    assert_eq!(read(&columns).await, Err(Reason::Schema));
}

#[tokio::test]
async fn a_scan_that_outlives_the_budget_is_interrupted() {
    let db = FakeDb::with_token(&token("valid"));
    // Without an index on `key` (the unique one is dropped by rebuilding the table) the lookup
    // scans every row.
    let writer = db.writer();
    writer.execute_batch("CREATE TABLE Plain (key TEXT, value BLOB); DROP TABLE ItemTable; ALTER TABLE Plain RENAME TO ItemTable;").unwrap();
    writer.execute_batch("BEGIN").unwrap();
    for n in 0..200_000 {
        writer.execute("INSERT INTO ItemTable VALUES (?1, 'x')", [format!("filler-{n}")]).unwrap();
    }
    writer.execute_batch("COMMIT").unwrap();
    drop(writer);
    let tight = Limits { busy: Duration::from_millis(100), budget: Duration::ZERO };
    let started = Instant::now();
    assert_eq!(read_token(db.path(), tight).await, Err(Reason::Timeout));
    assert!(started.elapsed() < Duration::from_secs(5), "{:?}", started.elapsed());
}

#[test]
fn the_statement_is_pinned_to_the_one_key_and_the_one_table() {
    let lower = SELECT_TOKEN.to_ascii_lowercase();
    assert!(SELECT_TOKEN.contains(&format!("FROM {TABLE}")));
    assert!(SELECT_TOKEN.contains(&format!("'{KEY}'")));
    assert!(SELECT_TOKEN.contains(&MAX_VALUE_BYTES.to_string()));
    assert!(lower.contains("limit 1"));
    assert!(!lower.contains("select *"));
    for off_limits in ["cursordiskkv", "sqlite_master", "refreshtoken", "email", "join", "union"] {
        assert!(!lower.contains(off_limits), "{off_limits}");
    }
}

/// Everything SQLite asks the authorizer while `statement` runs against the production connection.
fn reads_of(statement: &str) -> Vec<String> {
    let db = FakeDb::with_token(&token("valid"));
    let conn = store::open(&db.path(), &FAST).unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    conn.authorizer(Some(move |ctx: AuthContext<'_>| {
        if let AuthAction::Read { table_name, column_name } = ctx.action {
            sink.lock().unwrap().push(format!("{table_name}.{column_name}"));
        }
        authorize(ctx)
    }))
    .unwrap();
    let mut prepared = conn.prepare(statement).unwrap();
    let mut rows = prepared.query([]).unwrap();
    while rows.next().unwrap().is_some() {}
    seen.lock().unwrap().clone()
}

#[test]
fn the_production_statement_touches_only_the_key_and_value_columns() {
    let reads = reads_of(SELECT_TOKEN);
    assert!(!reads.is_empty());
    for read in reads {
        assert!(read == "ItemTable.key" || read == "ItemTable.value", "{read}");
    }
}

#[test]
fn the_authorizer_refuses_everything_else() {
    let denied = |statement: &str| {
        let db = FakeDb::with_token(&token("valid"));
        let conn = store::open(&db.path(), &FAST).unwrap();
        conn.prepare(statement).err().and_then(|e| match e {
            rusqlite::Error::SqliteFailure(failure, _) => Some(failure.code),
            rusqlite::Error::SqlInputError { error, .. } => Some(error.code),
            _ => None,
        })
    };
    for statement in [
        "SELECT value FROM cursorDiskKV",
        "SELECT * FROM sqlite_master",
        "SELECT rowid FROM ItemTable",
        "SELECT ItemTable.key, cursorDiskKV.value FROM ItemTable, cursorDiskKV",
        "PRAGMA writable_schema = 1",
        "ATTACH DATABASE 'other.db' AS other",
        "INSERT INTO ItemTable VALUES ('a', 'b')",
        "DELETE FROM ItemTable",
        "CREATE TABLE t (a)",
    ] {
        assert_eq!(
            denied(statement),
            Some(ErrorCode::AuthorizationForStatementDenied),
            "{statement}"
        );
    }
    // A function off the list fails to prepare too (SQLite reports it as a plain error).
    for statement in ["SELECT hex(value) FROM ItemTable", "SELECT load_extension('x')"] {
        assert!(denied(statement).is_some(), "{statement}");
    }
    assert_eq!(denied("SELECT key, value FROM ItemTable WHERE key = 'x'"), None);
}

#[test]
fn without_the_fence_the_read_only_flag_alone_still_refuses_writes() {
    let db = FakeDb::with_token(&token("valid"));
    let conn =
        Connection::open_with_flags(db.path(), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let error = conn.execute("INSERT INTO ItemTable VALUES ('a', 'b')", []).unwrap_err();
    assert_eq!(error.sqlite_error().map(|f| f.code), Some(ErrorCode::ReadOnly));
}

#[test]
fn reasons_are_fixed_identifiers() {
    for reason in [
        Reason::Permission,
        Reason::Locked,
        Reason::NotADatabase,
        Reason::Schema,
        Reason::Timeout,
        Reason::Other,
    ] {
        assert!(reason.as_str().chars().all(|c| c.is_ascii_lowercase() || c == '_'));
    }
}
