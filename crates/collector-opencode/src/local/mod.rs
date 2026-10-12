//! The local consumption estimate, read from OpenCode's own SQLite file. Never quota: the numbers
//! are list-price cost and token sums for this PC and say nothing about plan windows.

pub(crate) mod db;
pub(crate) mod guard;
pub(crate) mod path;
pub(crate) mod sql;

use std::path::PathBuf;
use std::time::Duration;

use usage_model::Ctx;
use usage_model::settings::OpencodeSettings;

/// Why the estimate is missing. Fixed identifiers only: shown as the notice's `reason` param.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    /// The `dataDir` setting is relative, a network path or otherwise unusable.
    InvalidPath,
    NotFound,
    Permission,
    /// A writer holds the file longer than the busy timeout (rollback-journal mode only).
    Locked,
    NotADatabase,
    /// `session_message` or one of its columns is missing: the schema changed.
    Schema,
    Timeout,
    Other,
}

impl Reason {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidPath => "invalid_path",
            Self::NotFound => "not_found",
            Self::Permission => "permission",
            Self::Locked => "locked",
            Self::NotADatabase => "not_a_database",
            Self::Schema => "schema",
            Self::Timeout => "timeout",
            Self::Other => "other",
        }
    }
}

/// Sums over trailing windows. `Some(0.0)` is "nothing happened"; `None` is unknown.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Facts {
    pub cost_5h: Option<f64>,
    pub cost_7d: Option<f64>,
    pub cost_30d: Option<f64>,
    pub tokens_5h: Option<f64>,
    pub tokens_7d: Option<f64>,
    /// Newest modification time of the database or its write-ahead log.
    pub read_at: Option<i64>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Outcome {
    Facts(Facts),
    Unreadable(Reason),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Limits {
    /// How long a locked database is retried before giving up.
    pub busy: Duration,
    /// Wall-clock budget for the statements; the query is interrupted when it runs out.
    pub budget: Duration,
}

impl Limits {
    pub(crate) const PRODUCTION: Self =
        Self { busy: Duration::from_millis(1_500), budget: Duration::from_secs(8) };

    /// Backstop for the async side; the blocking work normally stops itself earlier.
    fn overall(&self) -> Duration {
        self.budget + self.busy * 2
    }
}

pub(crate) async fn read(settings: &OpencodeSettings, ctx: &Ctx) -> Outcome {
    match path::resolve(settings, ctx) {
        Ok(path) => read_path(path, ctx.now_secs, Limits::PRODUCTION).await,
        Err(reason) => Outcome::Unreadable(reason),
    }
}

/// SQLite is synchronous, so it runs on the blocking pool and is never awaited without a timeout.
pub(crate) async fn read_path(path: PathBuf, now_secs: i64, limits: Limits) -> Outcome {
    bounded(limits.overall(), move || db::read_facts(&path, now_secs, &limits)).await
}

/// A panic or a hang in the blocking work becomes a notice, never a crash or a stuck poll.
pub(crate) async fn bounded(
    overall: Duration,
    work: impl FnOnce() -> Outcome + Send + 'static,
) -> Outcome {
    match tokio::time::timeout(overall, tokio::task::spawn_blocking(work)).await {
        Ok(Ok(outcome)) => outcome,
        Ok(Err(_panicked)) => Outcome::Unreadable(Reason::Other),
        Err(_elapsed) => Outcome::Unreadable(Reason::Timeout),
    }
}
