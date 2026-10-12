//! Opens opencode.db read-only and runs the aggregate (plus a small probe when the last 30 days are
//! empty). Nothing here writes to the database, and nothing selects message text: see `sql.rs` for
//! the statements and `guard.rs` for the fence. SQLite itself may create empty -wal/-shm sidecars
//! when it opens a WAL database read-only while OpenCode is closed; `last_write` ignores that log.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{Instant, UNIX_EPOCH};

use rusqlite::{Connection, ErrorCode, OpenFlags, Row, named_params};

use super::{Facts, Limits, Outcome, Reason, guard, sql};

/// How many SQLite VM steps pass between deadline checks: cheap, yet millisecond-responsive.
const PROGRESS_OPS: i32 = 1_000;
const HOUR_SECS: i64 = 3_600;
const DAY_SECS: i64 = 24 * HOUR_SECS;

pub(crate) fn read_facts(path: &Path, now_secs: i64, limits: &Limits) -> Outcome {
    match try_read(path, now_secs, limits) {
        Ok(facts) => Outcome::Facts(facts),
        Err(reason) => Outcome::Unreadable(reason),
    }
}

fn try_read(path: &Path, now_secs: i64, limits: &Limits) -> Result<Facts, Reason> {
    // Stat before opening: a read-only open of a WAL database can create an empty -wal.
    let read_at = last_write(path, now_secs)?;
    let conn = open(path, limits)?;
    require_schema(&conn)?;
    let sums = aggregate(&conn, now_secs)?;
    if sums.count_30d == 0 {
        require_dates(&conn)?;
    }
    sums.into_facts(read_at)
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
    conn.authorizer(Some(guard::authorize)).map_err(|e| classify(&e))?;
    Ok(conn)
}

fn require_schema(conn: &Connection) -> Result<(), Reason> {
    let mut statement = conn.prepare(sql::SCHEMA_PROBE).map_err(|e| classify(&e))?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .and_then(Iterator::collect::<Result<Vec<_>, _>>)
        .map_err(|e| classify(&e))?;
    let has = |wanted: &str| columns.iter().any(|name| name.eq_ignore_ascii_case(wanted));
    if sql::REQUIRED_COLUMNS.iter().all(|wanted| has(wanted)) {
        Ok(())
    } else {
        Err(Reason::Schema)
    }
}

/// No recent messages is normal, but assistant messages none of which has a readable time mean
/// the JSON layout moved: report that instead of a believable zero.
pub(crate) fn require_dates(conn: &Connection) -> Result<(), Reason> {
    let (assistant, dated): (i64, i64) = conn
        .prepare(sql::DATED_ROWS_PROBE)
        .and_then(|mut probe| probe.query_row([], |row| Ok((row.get(0)?, row.get(1)?))))
        .map_err(|e| classify(&e))?;
    if assistant > 0 && dated == 0 { Err(Reason::Schema) } else { Ok(()) }
}

struct WindowSums {
    count: i64,
    cost: Option<f64>,
    tokens_in: Option<f64>,
    tokens_out: Option<f64>,
    tokens_reasoning: Option<f64>,
}

impl WindowSums {
    fn from_row(row: &Row<'_>, first: usize) -> rusqlite::Result<Self> {
        Ok(Self {
            count: row.get(first)?,
            cost: row.get(first + 1)?,
            tokens_in: row.get(first + 2)?,
            tokens_out: row.get(first + 3)?,
            tokens_reasoning: row.get(first + 4)?,
        })
    }
}

pub(crate) struct Sums {
    five_hours: WindowSums,
    seven_days: WindowSums,
    count_30d: i64,
    cost_30d: Option<f64>,
}

pub(crate) fn aggregate(conn: &Connection, now_secs: i64) -> Result<Sums, Reason> {
    let since_ms = |span_secs: i64| now_secs.saturating_sub(span_secs).saturating_mul(1_000);
    let mut statement = conn.prepare(sql::AGGREGATE).map_err(|e| classify(&e))?;
    statement
        .query_row(
            named_params! {
                ":since_5h": since_ms(5 * HOUR_SECS),
                ":since_7d": since_ms(7 * DAY_SECS),
                ":since_30d": since_ms(30 * DAY_SECS),
            },
            |row| {
                Ok(Sums {
                    five_hours: WindowSums::from_row(row, 0)?,
                    seven_days: WindowSums::from_row(row, 5)?,
                    count_30d: row.get(10)?,
                    cost_30d: row.get(11)?,
                })
            },
        )
        .map_err(|e| classify(&e))
}

impl Sums {
    /// Rows exist but not one number could be read: the JSON layout changed, not "no usage".
    pub(crate) fn into_facts(self, read_at: Option<i64>) -> Result<Facts, Reason> {
        let facts = Facts {
            cost_5h: cost(self.five_hours.count, self.five_hours.cost),
            cost_7d: cost(self.seven_days.count, self.seven_days.cost),
            cost_30d: cost(self.count_30d, self.cost_30d),
            tokens_5h: tokens(&self.five_hours),
            tokens_7d: tokens(&self.seven_days),
            read_at,
        };
        let all_unknown = facts.cost_5h.is_none()
            && facts.cost_7d.is_none()
            && facts.cost_30d.is_none()
            && facts.tokens_5h.is_none()
            && facts.tokens_7d.is_none();
        if self.count_30d > 0 && all_unknown { Err(Reason::Schema) } else { Ok(facts) }
    }
}

/// No messages is a real zero; messages without a usable number are unknown. Negative or
/// non-finite sums are unknown too, never clamped. Rounded to micro-dollars to hide float noise.
fn cost(count: i64, sum: Option<f64>) -> Option<f64> {
    if count == 0 {
        return Some(0.0);
    }
    let usable = sum.filter(|value| value.is_finite() && *value >= 0.0)?;
    // Rounding a huge sum overflows to infinity, which must stay unknown too.
    Some((usable * 1e6).round() / 1e6).filter(|rounded| rounded.is_finite())
}

/// Input + output + reasoning. Cache tokens are excluded: they are re-reads, not new work.
fn tokens(window: &WindowSums) -> Option<f64> {
    if window.count == 0 {
        return Some(0.0);
    }
    if window.tokens_in.is_none() && window.tokens_out.is_none() {
        return None;
    }
    let parts = [window.tokens_in, window.tokens_out, window.tokens_reasoning];
    let valid = |part: &Option<f64>| part.is_none_or(|value| value.is_finite() && value >= 0.0);
    // The parts are finite but their sum may not be.
    parts
        .iter()
        .all(valid)
        .then(|| parts.iter().flatten().sum::<f64>())
        .filter(|total| total.is_finite())
}

/// Newest mtime of the database and its write-ahead log (the log holds the latest commits).
pub(crate) fn last_write(path: &Path, now_secs: i64) -> Result<Option<i64>, Reason> {
    let meta = fs::metadata(path).map_err(|e| match e.kind() {
        ErrorKind::NotFound => Reason::NotFound,
        ErrorKind::PermissionDenied => Reason::Permission,
        _ => Reason::Other,
    })?;
    if !meta.is_file() {
        return Err(Reason::NotFound);
    }
    // A read-only open creates an empty log (no commits): it must not move read_at.
    let wal = fs::metadata(sibling(path, "-wal")).ok().filter(|meta| meta.len() > 0);
    let newest = [Some(&meta), wal.as_ref()].into_iter().flatten().filter_map(modified_secs).max();
    Ok(newest.map(|secs| secs.min(now_secs)))
}

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

fn modified_secs(meta: &fs::Metadata) -> Option<i64> {
    let since_epoch = meta.modified().ok()?.duration_since(UNIX_EPOCH).ok()?;
    i64::try_from(since_epoch.as_secs()).ok()
}

pub(crate) fn classify(error: &rusqlite::Error) -> Reason {
    let Some(failure) = error.sqlite_error() else { return Reason::Other };
    match failure.code {
        ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked => Reason::Locked,
        ErrorCode::NotADatabase | ErrorCode::DatabaseCorrupt => Reason::NotADatabase,
        ErrorCode::OperationInterrupted => Reason::Timeout,
        ErrorCode::AuthorizationForStatementDenied => Reason::Schema,
        ErrorCode::PermissionDenied | ErrorCode::CannotOpen => Reason::Permission,
        _ => Reason::Other,
    }
}
