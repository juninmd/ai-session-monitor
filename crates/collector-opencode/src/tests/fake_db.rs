//! A fabricated opencode.db with the real table shapes. Every value is invented. The canary
//! strings sit where real secrets and prompts would: they must never reach any output.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, Ordering};

use rusqlite::{Connection, params};
use serde_json::json;
use tempfile::TempDir;

pub const NOW: i64 = 1_800_000_000;
pub const NOW_MS: i64 = NOW * 1_000;
pub const FIVE_H_MS: i64 = 5 * 3_600_000;
pub const SEVEN_D_MS: i64 = 7 * 86_400_000;
pub const THIRTY_D_MS: i64 = 30 * 86_400_000;

pub const CANARY_CREDENTIAL: &str = "CANARY-CREDENTIAL-7f3a9c";
pub const CANARY_PROMPT: &str = "CANARY-PROMPT-b81d42";

const SCHEMA: &str = "
CREATE TABLE session_message (
  id TEXT PRIMARY KEY, session_id TEXT NOT NULL, type TEXT NOT NULL, seq INTEGER NOT NULL,
  time_created INTEGER NOT NULL, time_updated INTEGER NOT NULL, data TEXT NOT NULL);
CREATE TABLE credential (id TEXT PRIMARY KEY, secret TEXT NOT NULL);
CREATE TABLE account (id TEXT PRIMARY KEY, email TEXT, access_token TEXT);
CREATE TABLE kv (key TEXT PRIMARY KEY, value TEXT);";

pub struct FakeDb {
    dir: TempDir,
    next_id: AtomicI64,
}

impl FakeDb {
    /// An empty database with the message table plus poisoned credential, account and kv tables.
    pub fn new() -> Self {
        let db = Self::bare();
        let conn = db.writer();
        conn.execute_batch(SCHEMA).unwrap();
        conn.execute("INSERT INTO credential VALUES ('c1', ?1)", [CANARY_CREDENTIAL]).unwrap();
        conn.execute("INSERT INTO account VALUES ('a1', ?1, ?1)", [CANARY_CREDENTIAL]).unwrap();
        conn.execute("INSERT INTO kv VALUES ('k1', ?1)", [CANARY_CREDENTIAL]).unwrap();
        drop(conn);
        db
    }

    /// A directory with no database file yet, for schema-drift and corruption cases.
    pub fn bare() -> Self {
        Self { dir: TempDir::new().unwrap(), next_id: AtomicI64::new(1) }
    }

    pub fn dir(&self) -> &Path {
        self.dir.path()
    }

    pub fn path(&self) -> PathBuf {
        self.dir.path().join("opencode.db")
    }

    /// A read-write connection: tests build the fixture with it; the collector never does.
    pub fn writer(&self) -> Connection {
        Connection::open(self.path()).unwrap()
    }

    pub fn add_raw(&self, conn: &Connection, kind: &str, created_ms: i64, data: &str) {
        let n = self.next_id.fetch_add(1, Ordering::SeqCst);
        conn.execute(
            "INSERT INTO session_message VALUES (?1, 'ses-1', ?2, ?3, ?4, ?4, ?5)",
            params![format!("msg-{n}"), kind, n, created_ms, data],
        )
        .unwrap();
    }

    pub fn add_assistant(&self, conn: &Connection, created_ms: i64, cost: f64, tokens: [i64; 3]) {
        self.add_raw(conn, "assistant", created_ms, &assistant_json(created_ms, cost, tokens));
    }
}

/// `tokens` is `[input, output, reasoning]`.
pub fn assistant_json(created_ms: i64, cost: f64, tokens: [i64; 3]) -> String {
    json!({
        "role": "assistant",
        "cost": cost,
        "tokens": {
            "input": tokens[0], "output": tokens[1], "reasoning": tokens[2],
            "cache": { "read": 7, "write": 3 },
        },
        "model": { "providerID": "fab-provider", "id": "fab-model" },
        "time": { "created": created_ms, "completed": created_ms + 1_000 },
        "text": CANARY_PROMPT,
    })
    .to_string()
}

/// The standard fixture: rows exactly on and one millisecond outside each window boundary, plus
/// rows that must be ignored. Expected sums are in `tests::local_db`.
pub fn populate(db: &FakeDb, conn: &Connection) {
    let rows = [
        (NOW_MS - 1_000, 0.10, [100, 50, 10]),
        (NOW_MS - FIVE_H_MS, 0.20, [200, 100, 20]),
        (NOW_MS - FIVE_H_MS - 1, 0.40, [400, 200, 40]),
        (NOW_MS - SEVEN_D_MS, 0.80, [800, 400, 80]),
        (NOW_MS - SEVEN_D_MS - 1, 1.60, [1_600, 800, 160]),
        (NOW_MS - THIRTY_D_MS, 3.20, [3_200, 1_600, 320]),
        (NOW_MS - THIRTY_D_MS - 1, 6.40, [6_400, 3_200, 640]),
    ];
    for (created, cost, tokens) in rows {
        db.add_assistant(conn, created, cost, tokens);
    }
    db.add_raw(conn, "user", NOW_MS - 1_000, &assistant_json(NOW_MS - 1_000, 99.0, [9, 9, 9]));
    db.add_raw(conn, "tool", NOW_MS - 1_000, &assistant_json(NOW_MS - 1_000, 99.0, [9, 9, 9]));
}
