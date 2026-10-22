//! Reads the one `cursorAuth/accessToken` row of Cursor's `state.vscdb`, read-only and fenced.
//!
//! The file also holds the editor's whole workspace state (chat history, account details,
//! settings), so three layers keep this read narrow: the statement is a constant that selects one
//! key; the connection is read-only with a deny-by-default authorizer that can reach only the
//! `key` and `value` columns of `ItemTable`; and the work has a row cap, a value-size cap and a
//! wall-clock deadline. Nothing here writes. SQLite may create empty `-wal`/`-shm` sidecars when
//! it opens a WAL database read-only while Cursor is closed.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use rusqlite::hooks::{AuthAction, AuthContext, Authorization};
use rusqlite::types::Value;
use rusqlite::{Connection, ErrorCode, OpenFlags};

pub(crate) const TABLE: &str = "ItemTable";
/// Pinned against [`SELECT_TOKEN`] by a test.
#[cfg(test)]
pub(crate) const KEY: &str = "cursorAuth/accessToken";
/// A JWT is a few hundred bytes to a couple of KiB; a bigger value is not a token and is never read.
pub(crate) const MAX_VALUE_BYTES: i64 = 16 * 1024;
/// How many SQLite VM steps pass between deadline checks.
const PROGRESS_OPS: i32 = 1_000;

/// The statement. Constant: nothing in it comes from settings or database content. `LIMIT 1` is the
/// row cap, and the value is returned only when it is within the size cap.
pub(crate) const SELECT_TOKEN: &str = "\
SELECT typeof(value), length(CAST(value AS BLOB)),
       CASE WHEN length(CAST(value AS BLOB)) <= 16384 THEN value END
FROM ItemTable WHERE key = 'cursorAuth/accessToken' LIMIT 1";

/// SQL functions the statement calls; the authorizer rejects every other one.
pub(crate) const FUNCTIONS: [&str; 2] = ["typeof", "length"];
const READABLE_COLUMNS: [&str; 2] = ["key", "value"];

/// Why the store could not be read. Fixed identifiers only: shown as the error's `reason` param.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    Permission,
    /// A writer holds the file longer than the busy timeout.
    Locked,
    NotADatabase,
    /// `ItemTable` or one of its columns is missing: the layout changed.
    Schema,
    Timeout,
    Other,
}

impl Reason {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Permission => "permission",
            Self::Locked => "locked",
            Self::NotADatabase => "not_a_database",
            Self::Schema => "schema",
            Self::Timeout => "timeout",
            Self::Other => "other",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Limits {
    /// How long a locked database is retried before giving up.
    pub busy: Duration,
    /// Wall-clock budget for the statement; the query is interrupted when it runs out.
    pub budget: Duration,
}

impl Limits {
    pub(crate) const PRODUCTION: Self =
        Self { busy: Duration::from_millis(1_500), budget: Duration::from_secs(5) };

    /// Backstop for the async side; the blocking work normally stops itself earlier.
    fn overall(&self) -> Duration {
        self.budget + self.busy * 2
    }
}

/// `Ok(None)`: no database, no such row, or a value that is not a short string. All of those mean
/// "no token", not a fault.
pub(crate) type Read = Result<Option<String>, Reason>;

/// SQLite is synchronous, so it runs on the blocking pool and is never awaited without a timeout.
pub(crate) async fn read_token(path: PathBuf, limits: Limits) -> Read {
    let work = tokio::task::spawn_blocking(move || read_blocking(&path, &limits));
    match tokio::time::timeout(limits.overall(), work).await {
        Ok(Ok(outcome)) => outcome,
        Ok(Err(_panicked)) => Err(Reason::Other),
        Err(_elapsed) => Err(Reason::Timeout),
    }
}

pub(crate) fn read_blocking(path: &Path, limits: &Limits) -> Read {
    // Stat before opening: a missing file is "Cursor was never installed here", and opening one
    // read-only would report a permission problem.
    match fs::metadata(path) {
        Ok(meta) if meta.is_file() => {}
        Ok(_) => return Ok(None),
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) if e.kind() == ErrorKind::PermissionDenied => return Err(Reason::Permission),
        Err(_) => return Err(Reason::Other),
    }
    let conn = open(path, limits)?;
    let mut statement = conn.prepare(SELECT_TOKEN).map_err(|e| classify(&e))?;
    let mut rows = statement.query([]).map_err(|e| classify(&e))?;
    let Some(row) = rows.next().map_err(|e| classify(&e))? else { return Ok(None) };
    let kind: String = row.get(0).map_err(|e| classify(&e))?;
    // `NULL` for a null value: there is nothing to read.
    let length: Option<i64> = row.get(1).map_err(|e| classify(&e))?;
    let sized = length.is_some_and(|length| (1..=MAX_VALUE_BYTES).contains(&length));
    if !matches!(kind.as_str(), "text" | "blob") || !sized {
        return Ok(None);
    }
    let bytes = match row.get::<_, Value>(2).map_err(|e| classify(&e))? {
        Value::Text(text) => text.into_bytes(),
        Value::Blob(blob) => blob,
        _ => return Ok(None),
    };
    Ok(String::from_utf8(bytes).ok().map(unquote))
}

/// Some VS Code builds store string values JSON-encoded (`"eyJ..."`); the token itself never
/// contains a quote, so one surrounding pair is dropped.
fn unquote(value: String) -> String {
    let trimmed = value.trim();
    match trimmed.strip_prefix('"').and_then(|rest| rest.strip_suffix('"')) {
        Some(inner) => inner.to_owned(),
        None => trimmed.to_owned(),
    }
}

/// Read-only, no create, no URI interpretation, a short busy timeout, a hard deadline and the
/// deny-by-default authorizer. A WAL database stays in WAL mode and is never checkpointed.
pub(crate) fn open(path: &Path, limits: &Limits) -> Result<Connection, Reason> {
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = Connection::open_with_flags(path, flags).map_err(|e| classify(&e))?;
    conn.busy_timeout(limits.busy).map_err(|e| classify(&e))?;
    let deadline = Instant::now() + limits.budget;
    conn.progress_handler(PROGRESS_OPS, Some(move || Instant::now() >= deadline))
        .map_err(|e| classify(&e))?;
    conn.authorizer(Some(authorize)).map_err(|e| classify(&e))?;
    Ok(conn)
}

/// Even a mistake in the SQL text, or a view planted in the file that points elsewhere, cannot
/// reach another table or column.
pub(crate) fn authorize(ctx: AuthContext<'_>) -> Authorization {
    if ctx.database_name.is_some_and(|name| !name.eq_ignore_ascii_case("main")) {
        return Authorization::Deny;
    }
    let allowed = match ctx.action {
        AuthAction::Select => true,
        AuthAction::Read { table_name, column_name } => {
            table_name == TABLE && READABLE_COLUMNS.contains(&column_name)
        }
        AuthAction::Function { function_name } => {
            FUNCTIONS.iter().any(|name| name.eq_ignore_ascii_case(function_name))
        }
        _ => false,
    };
    if allowed { Authorization::Allow } else { Authorization::Deny }
}

pub(crate) fn classify(error: &rusqlite::Error) -> Reason {
    // A statement that fails to prepare (no such table or column) arrives as `SqlInputError`.
    let code = match error {
        rusqlite::Error::SqliteFailure(failure, _) => failure.code,
        rusqlite::Error::SqlInputError { error, .. } => error.code,
        _ => return Reason::Other,
    };
    match code {
        ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked => Reason::Locked,
        ErrorCode::NotADatabase | ErrorCode::DatabaseCorrupt => Reason::NotADatabase,
        ErrorCode::OperationInterrupted => Reason::Timeout,
        // A statement the schema cannot serve (no such table or column) or the fence refused.
        ErrorCode::AuthorizationForStatementDenied | ErrorCode::Unknown => Reason::Schema,
        ErrorCode::PermissionDenied | ErrorCode::CannotOpen => Reason::Permission,
        _ => Reason::Other,
    }
}
