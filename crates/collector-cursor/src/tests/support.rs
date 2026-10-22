//! Fixtures and builders shared by the suites.

use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::Connection;
use tempfile::TempDir;
use usage_model::settings::CursorSettings;
use usage_model::{Ctx, HarnessView};

use super::fake_server::FakeServer;
use crate::collect_with;
use crate::remote::Endpoint;
use crate::store::Limits;

/// 2027-01-15T08:00:00Z. The fixtures' cycle dates are relative to it.
pub const NOW: i64 = 1_800_000_000;
/// The cookie value the valid fixture token must produce.
pub const COOKIE: &str = "WorkosCursorSessionToken=user_01FABCURSOR%3A%3A";
/// Planted in the other rows of the fabricated database: it must never reach any output.
pub const CANARY: &str = "CANARY-CHAT-HISTORY-5e1b07";

/// A short busy wait keeps lock tests quick; the budget stays generous so slow CI is not flaky.
pub const FAST: Limits =
    Limits { busy: Duration::from_millis(100), budget: Duration::from_secs(5) };

pub fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name);
    std::fs::read_to_string(path).unwrap()
}

/// One of the fabricated JWTs in `tokens.json`: `valid`, `expired`, `expires_in_30s`, `no_exp`,
/// `no_sub`.
pub fn token(name: &str) -> String {
    let all: serde_json::Value = serde_json::from_str(&fixture("tokens.json")).unwrap();
    all[name].as_str().unwrap().to_owned()
}

pub fn ctx() -> Ctx {
    Ctx::new(NOW, "app-data-not-used", "home-not-used")
}

pub fn on() -> CursorSettings {
    CursorSettings { enabled: true, consent: true }
}

/// A fabricated `state.vscdb` in a temp dir, with the real table shape and poisoned neighbours.
pub struct FakeDb {
    dir: TempDir,
}

impl FakeDb {
    /// `ItemTable` holding `token` (as TEXT) under the real key, plus rows that must never be read.
    pub fn with_token(token: &str) -> Self {
        let db = Self::empty();
        db.writer()
            .execute("INSERT INTO ItemTable VALUES ('cursorAuth/accessToken', ?1)", [token])
            .unwrap();
        db
    }

    /// The table and the poisoned neighbour rows, but no token.
    pub fn empty() -> Self {
        let db = Self { dir: TempDir::new().unwrap() };
        let conn = db.writer();
        conn.execute_batch(
            "CREATE TABLE ItemTable (key TEXT UNIQUE ON CONFLICT REPLACE, value BLOB);
             CREATE TABLE cursorDiskKV (key TEXT UNIQUE ON CONFLICT REPLACE, value BLOB);",
        )
        .unwrap();
        for key in ["cursorAuth/refreshToken", "cursorAuth/cachedEmail", "composer.history"] {
            conn.execute("INSERT INTO ItemTable VALUES (?1, ?2)", [key, CANARY]).unwrap();
        }
        conn.execute("INSERT INTO cursorDiskKV VALUES ('chat', ?1)", [CANARY]).unwrap();
        drop(conn);
        db
    }

    /// A directory with no database file, for the "never installed" cases.
    pub fn absent() -> Self {
        Self { dir: TempDir::new().unwrap() }
    }

    pub fn path(&self) -> PathBuf {
        self.dir.path().join("state.vscdb")
    }

    /// A read-write connection: tests build the fixture with it; the collector never does.
    pub fn writer(&self) -> Connection {
        Connection::open(self.path()).unwrap()
    }
}

pub fn loopback(server: &FakeServer) -> Endpoint {
    Endpoint::loopback(server.base_url(), Duration::from_secs(2))
}

/// One poll plus the checks every view must pass: no token, cookie or canary in it.
pub async fn poll(settings: &CursorSettings, db: &FakeDb, server: &FakeServer) -> HarnessView {
    let view = collect_with(settings, &ctx(), db.path(), FAST, &loopback(server)).await;
    assert_clean(&view);
    view
}

#[track_caller]
pub fn assert_clean(view: &HarnessView) {
    let secrets =
        [token("valid"), token("expired"), "user_01FABCURSOR".to_owned(), CANARY.to_owned()];
    let json = serde_json::to_string(view).unwrap();
    let debug = format!("{view:?}");
    for secret in secrets {
        assert!(!json.contains(&secret), "{secret} leaked into the serialized view");
        assert!(!debug.contains(&secret), "{secret} leaked into the debug output");
    }
    assert!(!json.contains("WorkosCursorSessionToken"));
}
