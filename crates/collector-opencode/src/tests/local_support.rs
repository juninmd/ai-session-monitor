//! Helpers shared by the local-reader suites.

use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

use super::fake_db::{FakeDb, NOW, populate};
use crate::local::{self, Facts, Limits, Outcome, Reason};

/// A short busy wait keeps lock tests quick; the budget stays generous so slow CI is not flaky.
pub const FAST: Limits =
    Limits { busy: Duration::from_millis(100), budget: Duration::from_secs(5) };

pub async fn read(path: &Path) -> Outcome {
    local::read_path(path.to_path_buf(), NOW, FAST).await
}

#[track_caller]
pub fn facts(outcome: Outcome) -> Facts {
    match outcome {
        Outcome::Facts(facts) => facts,
        other => panic!("expected facts, got {other:?}"),
    }
}

#[track_caller]
pub fn reason(outcome: Outcome) -> Reason {
    match outcome {
        Outcome::Unreadable(reason) => reason,
        other => panic!("expected an unreadable database, got {other:?}"),
    }
}

pub fn mtime_secs(path: &Path) -> i64 {
    let modified = std::fs::metadata(path).unwrap().modified().unwrap();
    i64::try_from(modified.duration_since(UNIX_EPOCH).unwrap().as_secs()).unwrap()
}

/// The standard fixture, its writer already closed.
pub fn populated() -> FakeDb {
    let fake = FakeDb::new();
    let writer = fake.writer();
    populate(&fake, &writer);
    drop(writer);
    fake
}
