//! The fence around opencode.db, which also holds credentials, accounts and prompts. Two layers:
//! the statements are constants that never select message text, and the connection can only reach
//! what the aggregate needs. The tests pin both, and what each layer cannot do alone.

use std::fs;
use std::sync::{Arc, Mutex};

use rusqlite::hooks::{AuthAction, AuthContext, Authorization};
use rusqlite::{Connection, ErrorCode};

use super::fake_db::{CANARY_CREDENTIAL, CANARY_PROMPT, NOW};
use super::local_support::{FAST, populated};
use crate::local::{db, guard, sql};

const TABLES_OFF_LIMITS: [&str; 11] = [
    "credential",
    "account",
    "control_account",
    "account_state",
    "kv",
    "instruction",
    "session_inbox",
    "session_pending",
    "sqlite_master",
    "sqlite_schema",
    "sqlite_temp",
];

/// Table names that follow `from`/`join`; `None` stands for a sub-select.
fn sources_of(statement: &str) -> Vec<Option<String>> {
    let lower = statement.to_ascii_lowercase();
    let tokens: Vec<&str> = lower.split_whitespace().collect();
    tokens
        .windows(2)
        .filter(|pair| matches!(pair[0], "from" | "join"))
        .map(|pair| (pair[1] != "(").then(|| pair[1].to_owned()))
        .collect()
}

#[test]
fn the_statements_read_from_the_message_table_only() {
    for statement in [sql::AGGREGATE, sql::SCHEMA_PROBE, sql::DATED_ROWS_PROBE] {
        let lower = statement.to_ascii_lowercase();
        for table in TABLES_OFF_LIMITS {
            assert!(!lower.contains(table), "{table} is named in: {statement}");
        }
        for source in sources_of(statement).into_iter().flatten() {
            assert_eq!(source, sql::TABLE);
        }
    }
    assert!(!sources_of(sql::AGGREGATE).is_empty(), "the aggregate reads something");
}

#[test]
fn message_text_is_only_ever_passed_to_json_functions() {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    for statement in [sql::AGGREGATE, sql::DATED_ROWS_PROBE] {
        assert!(!statement.to_ascii_lowercase().contains("select *"));
        let mut uses = 0;
        for (at, _) in statement.match_indices("data") {
            let (before, after) = (&statement[..at], &statement[at + 4..]);
            if before.chars().next_back().is_some_and(word)
                || after.chars().next().is_some_and(word)
            {
                continue;
            }
            uses += 1;
            let wrapped =
                ["json_extract(", "json_type(", "json_valid("].iter().any(|f| before.ends_with(f));
            assert!(wrapped, "`data` used outside a json_* call at byte {at} of: {statement}");
        }
        assert!(uses > 0, "the statement must read `data` through the json functions");
    }
}

#[derive(Debug, PartialEq)]
enum Seen {
    Select,
    Read(String, String),
    Function(String),
    Pragma(String, Option<String>),
    Other(String),
}

/// Records what SQLite asks the authorizer, then lets the production fence decide.
fn recording(conn: &Connection) -> Arc<Mutex<Vec<Seen>>> {
    let log = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&log);
    let hook = move |ctx: AuthContext<'_>| {
        let seen = match ctx.action {
            AuthAction::Select => Seen::Select,
            AuthAction::Read { table_name, column_name } => {
                Seen::Read(table_name.to_owned(), column_name.to_owned())
            }
            AuthAction::Function { function_name } => Seen::Function(function_name.to_owned()),
            AuthAction::Pragma { pragma_name, pragma_value } => {
                Seen::Pragma(pragma_name.to_owned(), pragma_value.map(str::to_owned))
            }
            other => Seen::Other(format!("{other:?}")),
        };
        sink.lock().unwrap().push(seen);
        guard::authorize(ctx)
    };
    conn.authorizer(Some(hook)).unwrap();
    log
}

#[test]
fn the_queries_read_two_columns_and_call_only_the_listed_functions() {
    let fake = populated();
    let conn = db::open(&fake.path(), &FAST).unwrap();
    let log = recording(&conn);
    db::aggregate(&conn, NOW).unwrap();
    db::require_dates(&conn).unwrap();
    let seen = log.lock().unwrap();
    for event in seen.iter() {
        match event {
            Seen::Select => {}
            Seen::Read(table, column) => {
                assert_eq!(table, sql::TABLE);
                assert!(["type", "data"].contains(&column.as_str()), "column {column}");
            }
            Seen::Function(name) => {
                assert!(
                    sql::FUNCTIONS.iter().any(|f| f.eq_ignore_ascii_case(name)),
                    "function {name}"
                );
            }
            other => panic!("unexpected authorizer request {other:?}"),
        }
    }
    for column in ["type", "data"] {
        assert!(seen.contains(&Seen::Read(sql::TABLE.to_owned(), column.to_owned())), "{column}");
    }
}

#[test]
fn the_schema_probe_asks_for_table_info_of_the_message_table_and_nothing_else() {
    let fake = populated();
    let conn = db::open(&fake.path(), &FAST).unwrap();
    let log = recording(&conn);
    let mut statement = conn.prepare(sql::SCHEMA_PROBE).unwrap();
    assert!(statement.query_map([], |row| row.get::<_, String>(1)).unwrap().count() >= 2);
    let probe = Seen::Pragma("table_info".to_owned(), Some(sql::TABLE.to_owned()));
    assert!(log.lock().unwrap().iter().all(|event| *event == probe));
}

#[test]
fn the_fence_refuses_everything_outside_the_allowlist() {
    let fake = populated();
    let before = fs::read(fake.path()).unwrap();
    let conn = db::open(&fake.path(), &FAST).unwrap();
    let refused = [
        "SELECT secret FROM credential",
        "SELECT * FROM credential",
        "SELECT email, access_token FROM account",
        "SELECT value FROM kv",
        "SELECT * FROM session_message",
        "SELECT id FROM session_message",
        "SELECT time_created FROM session_message",
        "SELECT secret FROM main.credential",
        "SELECT type FROM session_message UNION SELECT secret FROM credential",
        "SELECT (SELECT secret FROM credential) FROM session_message",
        "SELECT name FROM sqlite_master",
        "SELECT name FROM sqlite_schema",
        "SELECT name FROM temp.sqlite_master",
        "SELECT random()",
        "SELECT sqlite_version()",
        "SELECT hex(type) FROM session_message",
        "PRAGMA user_version",
        "PRAGMA table_info(credential)",
        "PRAGMA writable_schema = 1",
        "PRAGMA journal_mode = DELETE",
        "PRAGMA query_only = 0",
        "INSERT INTO session_message VALUES ('x', 's', 'assistant', 1, 1, 1, '{}')",
        "UPDATE session_message SET type = 'user'",
        "DELETE FROM credential",
        "CREATE TABLE t (a)",
        "CREATE TEMP TABLE t (a)",
        "CREATE VIEW v AS SELECT 1",
        "DROP TABLE kv",
        "ALTER TABLE kv RENAME TO kv2",
        "ATTACH DATABASE ':memory:' AS other",
        "BEGIN",
        "SAVEPOINT x",
        "REINDEX",
        "ANALYZE",
        "VACUUM",
    ];
    for text in refused {
        let error = conn.execute_batch(text).expect_err(text);
        assert!(!format!("{error:?}").contains(CANARY_CREDENTIAL), "{text}");
    }
    drop(conn);
    assert_eq!(fs::read(fake.path()).unwrap(), before, "no refused statement changed the file");
    let check = fake.writer();
    let rows = |table: &str| -> i64 {
        check.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| row.get(0)).unwrap()
    };
    assert_eq!((rows("credential"), rows("account"), rows("kv")), (1, 1, 1));
}

#[test]
fn a_denied_read_is_reported_as_an_authorization_failure() {
    let fake = populated();
    let conn = db::open(&fake.path(), &FAST).unwrap();
    let error = conn.prepare("SELECT secret FROM credential").err().unwrap();
    assert_eq!(
        error.sqlite_error().map(|e| e.code),
        Some(ErrorCode::AuthorizationForStatementDenied)
    );
    assert_eq!(db::classify(&error), crate::local::Reason::Schema);
}

#[test]
fn the_connection_is_read_only_even_when_the_fence_is_removed() {
    let fake = populated();
    let conn = db::open(&fake.path(), &FAST).unwrap();
    conn.authorizer(None::<fn(AuthContext<'_>) -> Authorization>).unwrap();
    for text in [
        "INSERT INTO session_message VALUES ('x', 's', 'assistant', 1, 1, 1, '{}')",
        "DELETE FROM credential",
        "UPDATE kv SET value = 'x'",
        "CREATE TABLE t (a)",
        "DROP TABLE kv",
    ] {
        let error = conn.execute_batch(text).expect_err(text);
        assert_eq!(error.sqlite_error().map(|e| e.code), Some(ErrorCode::ReadOnly), "{text}");
    }
}

#[test]
fn the_fence_cannot_tell_json_calls_apart_so_the_statement_text_is_the_guard() {
    let fake = populated();
    let conn = db::open(&fake.path(), &FAST).unwrap();
    let text: String = conn
        .query_row("SELECT json_extract(data, '$.text') FROM session_message LIMIT 1", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(text, CANARY_PROMPT, "allowed by the fence on purpose: see the two statement tests");
}

#[test]
fn no_production_source_reads_the_environment_or_starts_a_process() {
    let sources = [
        ("lib.rs", include_str!("../lib.rs")),
        ("compose.rs", include_str!("../compose.rs")),
        ("notice.rs", include_str!("../notice.rs")),
        ("local/mod.rs", include_str!("../local/mod.rs")),
        ("local/db.rs", include_str!("../local/db.rs")),
        ("local/guard.rs", include_str!("../local/guard.rs")),
        ("local/path.rs", include_str!("../local/path.rs")),
        ("local/sql.rs", include_str!("../local/sql.rs")),
        ("remote/mod.rs", include_str!("../remote/mod.rs")),
        ("remote/month.rs", include_str!("../remote/month.rs")),
        ("remote/parse.rs", include_str!("../remote/parse.rs")),
    ];
    let banned =
        ["env::var", "var_os", "std::env", "std::process", "tokio::process", "Command::new"];
    for (name, text) in sources {
        for token in banned {
            assert!(!text.contains(token), "{name} uses {token}");
        }
    }
}
