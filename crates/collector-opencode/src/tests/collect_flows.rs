//! Whole polls through `collect_with`: what each combination of key and consent produces.

use std::collections::BTreeMap;
use std::fs;

use usage_model::{ErrorCode, Fact, HarnessId, Severity, Source, ViewState};

use super::fake_db::{FakeDb, NOW};
use super::flow_support::{notice_codes, ok_server, replying, run, serving, window_ids};
use super::local_support::{mtime_secs, populated};
use super::{secret, settings, snapshot};
use crate::notice;

fn fact(id: &str, number: f64, unit: &str) -> Fact {
    Fact { id: id.to_owned(), number: Some(number), text: None, unit: Some(unit.to_owned()) }
}

/// The sums of the standard fixture, in display order.
fn estimate() -> Vec<Fact> {
    vec![
        fact("local_cost_5h", 0.3, "usd"),
        fact("local_cost_7d", 1.5, "usd"),
        fact("local_cost_30d", 6.3, "usd"),
        fact("local_tokens_5h", 480.0, "tokens"),
        fact("local_tokens_7d", 2_400.0, "tokens"),
    ]
}

fn unreadable(reason: &str) -> usage_model::Notice {
    notice::make(notice::DB_UNREADABLE, Severity::Warn, &[("reason", reason)])
}

#[tokio::test]
async fn a_key_alone_shows_the_three_windows_and_nothing_local() {
    let (server, home) = (ok_server(), tempfile::tempdir().unwrap());
    let view = run(&settings(false, None), Some(&secret()), &server, home.path()).await;
    assert_eq!((view.id, view.state), (HarnessId::Opencode, ViewState::Ready));
    assert_eq!(view.source, Some(Source { id: "go_usage".to_owned(), official: false }));
    assert_eq!((view.read_at, view.stale), (Some(NOW), false));
    assert_eq!(window_ids(&view), ["rolling", "weekly", "monthly"]);
    assert!(view.facts.is_empty() && view.notices.is_empty() && view.meta.is_empty());
    assert!(view.error.is_none() && view.plan.is_none());
    assert_eq!(server.requests().len(), 1);
}

#[tokio::test]
async fn consent_alone_shows_the_estimate_as_facts_never_as_quota() {
    let (server, fake, home) = (ok_server(), populated(), tempfile::tempdir().unwrap());
    let view = run(&settings(true, Some(fake.dir())), None, &server, home.path()).await;
    server.assert_untouched();
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(view.source, Some(Source { id: "local_db".to_owned(), official: false }));
    assert_eq!(view.read_at, Some(mtime_secs(&fake.path()).min(NOW)));
    assert!(view.windows.is_empty(), "an estimate has no percent and no reset");
    assert_eq!(view.facts, estimate());
    assert_eq!(notice_codes(&view), [notice::NO_KEY, notice::LOCAL_ESTIMATE]);
    assert!(view.notices.iter().all(|n| n.severity == Severity::Info));
    assert!(!view.stale && view.error.is_none());
}

#[tokio::test]
async fn both_sources_compose_windows_with_the_estimate_attached() {
    let (server, fake, home) = (ok_server(), populated(), tempfile::tempdir().unwrap());
    let view = run(&settings(true, Some(fake.dir())), Some(&secret()), &server, home.path()).await;
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(view.source, Some(Source { id: "go_usage".to_owned(), official: false }));
    assert_eq!(view.read_at, Some(NOW), "the windows were just read; the file time is not used");
    assert_eq!(window_ids(&view), ["rolling", "weekly", "monthly"]);
    assert_eq!(view.facts, estimate());
    assert_eq!(notice_codes(&view), [notice::LOCAL_ESTIMATE]);
    let meta = BTreeMap::from([("local_estimate".to_owned(), "true".to_owned())]);
    assert_eq!(view.meta, meta);
}

#[tokio::test]
async fn a_remote_failure_wins_over_the_estimate() {
    let cases = [
        (replying(401, r#"{"error":"nope"}"#), ErrorCode::Unauthorized, false),
        (replying(429, "{}"), ErrorCode::RateLimited, true),
        (replying(500, "{}"), ErrorCode::ServerError, true),
        (serving("mismatch.json"), ErrorCode::EndpointChanged, false),
        (replying(200, "this is not json"), ErrorCode::BadResponse, false),
    ];
    for (server, code, retryable) in cases {
        let (fake, home) = (populated(), tempfile::tempdir().unwrap());
        let view =
            run(&settings(true, Some(fake.dir())), Some(&secret()), &server, home.path()).await;
        assert_eq!(view.state, ViewState::Error, "{code:?}");
        let error = view.error.as_ref().unwrap();
        assert_eq!((error.code, error.retryable), (code, retryable));
        assert!(view.windows.is_empty() && view.facts.is_empty() && view.notices.is_empty());
    }
}

#[tokio::test]
async fn an_unreadable_database_never_costs_the_windows() {
    let (server, empty, home) = (ok_server(), FakeDb::bare(), tempfile::tempdir().unwrap());
    let view = run(&settings(true, Some(empty.dir())), Some(&secret()), &server, home.path()).await;
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(window_ids(&view).len(), 3);
    assert!(view.facts.is_empty() && view.meta.is_empty(), "no estimate was attached");
    assert_eq!(view.notices, [unreadable("not_found")]);
}

/// Sets up the data dir for one unreadable-database scenario.
type Prepare = fn(&FakeDb);

fn no_file(_: &FakeDb) {}

fn garbage_file(db: &FakeDb) {
    fs::write(db.path(), [0xAB_u8; 4_096]).unwrap();
}

fn renamed_column(db: &FakeDb) {
    let conn = rusqlite::Connection::open(db.path()).unwrap();
    conn.execute_batch("CREATE TABLE session_message (id TEXT, type TEXT, body TEXT)").unwrap();
}

#[tokio::test]
async fn an_estimate_alone_that_cannot_be_read_is_no_data_with_the_reason() {
    let cases: [(&str, Prepare); 3] =
        [("not_found", no_file), ("not_a_database", garbage_file), ("schema", renamed_column)];
    for (reason, prepare) in cases {
        let (server, fake, home) = (ok_server(), FakeDb::bare(), tempfile::tempdir().unwrap());
        prepare(&fake);
        let view = run(&settings(true, Some(fake.dir())), None, &server, home.path()).await;
        server.assert_untouched();
        assert_eq!(view.state, ViewState::NoData, "{reason}");
        assert_eq!(view.source, Some(Source { id: "local_db".to_owned(), official: false }));
        assert_eq!(view.read_at, None);
        assert!(view.windows.is_empty() && view.facts.is_empty());
        assert_eq!(
            view.notices,
            [notice::make(notice::NO_KEY, Severity::Info, &[]), unreadable(reason)]
        );
    }
}

#[tokio::test]
async fn an_unusable_data_dir_is_reported_and_never_followed() {
    for bad in ["relative/dir", "dir\0evil", "..", "."] {
        let (server, home) = (ok_server(), tempfile::tempdir().unwrap());
        let before = snapshot(home.path());
        let mut configured = settings(true, None);
        configured.data_dir = Some(bad.to_owned());

        let alone = run(&configured, None, &server, home.path()).await;
        assert_eq!(alone.state, ViewState::NoData, "{bad:?}");
        assert_eq!(alone.notices[1], unreadable("invalid_path"));

        let with_key = run(&configured, Some(&secret()), &server, home.path()).await;
        assert_eq!(with_key.state, ViewState::Ready);
        assert_eq!(with_key.notices, [unreadable("invalid_path")]);
        assert_eq!(snapshot(home.path()), before, "nothing may be created for a rejected path");
    }
}

#[tokio::test]
async fn a_blank_data_dir_falls_back_to_the_default_location() {
    let (server, home) = (ok_server(), tempfile::tempdir().unwrap());
    let default_dir = home.path().join(".local").join("share").join("opencode");
    fs::create_dir_all(&default_dir).unwrap();
    fs::copy(populated().path(), default_dir.join("opencode.db")).unwrap();
    let mut configured = settings(true, None);
    configured.data_dir = Some("   ".to_owned());
    let view = run(&configured, None, &server, home.path()).await;
    assert_eq!(view.facts, estimate());
}

#[tokio::test]
async fn limit_notices_come_before_the_estimate_notice() {
    let cases = [
        ("rate_limited.json", notice::RATE_LIMITED, "monthly"),
        ("idle.json", notice::WINDOW_IDLE, "rolling"),
    ];
    for (name, code, window) in cases {
        let (server, fake, home) = (serving(name), populated(), tempfile::tempdir().unwrap());
        let view =
            run(&settings(true, Some(fake.dir())), Some(&secret()), &server, home.path()).await;
        assert_eq!(notice_codes(&view), [code, notice::LOCAL_ESTIMATE]);
        assert_eq!(view.notices[0].params["window"], window);
    }
}

#[tokio::test]
async fn polling_twice_gives_the_same_view() {
    let (server, fake, home) = (ok_server(), populated(), tempfile::tempdir().unwrap());
    let configured = settings(true, Some(fake.dir()));
    let first = run(&configured, Some(&secret()), &server, home.path()).await;
    assert_eq!(run(&configured, Some(&secret()), &server, home.path()).await, first);
    assert_eq!(server.requests().len(), 2, "one request per poll, never more");
}
