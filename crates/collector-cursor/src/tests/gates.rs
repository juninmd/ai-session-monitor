//! The gates that run before any I/O, and every way the stored token can be unusable.

use rusqlite::Connection;
use usage_model::settings::CursorSettings;
use usage_model::{HarnessId, HarnessView, ViewState};

use super::fake_server::{FakeServer, Reply};
use super::{FakeDb, fixture, on, poll, token};
use crate::gate;

fn serving() -> FakeServer {
    FakeServer::always(|| Reply::json(200, &fixture("ok_pro.json")))
}

#[test]
fn a_disabled_harness_is_never_read_whatever_else_is_set() {
    let off = CursorSettings { enabled: false, consent: true };
    assert_eq!(gate(&off), Some(HarnessView::disabled(HarnessId::Cursor)));
    let off_without_consent = CursorSettings { enabled: false, consent: false };
    assert_eq!(gate(&off_without_consent), Some(HarnessView::disabled(HarnessId::Cursor)));
}

#[test]
fn without_consent_nothing_is_read() {
    let settings = CursorSettings { enabled: true, consent: false };
    assert_eq!(gate(&settings), Some(HarnessView::needs_consent(HarnessId::Cursor)));
    assert_eq!(gate(&on()), None);
    assert_eq!(CursorSettings::default(), CursorSettings { enabled: true, consent: false });
}

#[tokio::test]
async fn gated_polls_open_no_socket_even_with_a_valid_token_on_disk() {
    let db = FakeDb::with_token(&token("valid"));
    let server = serving();
    let off = CursorSettings { enabled: false, consent: true };
    assert_eq!(poll(&off, &db, &server).await.state, ViewState::Disabled);
    let no_consent = CursorSettings { enabled: true, consent: false };
    assert_eq!(poll(&no_consent, &db, &server).await.state, ViewState::NeedsConsent);
    server.assert_untouched();
}

#[tokio::test]
async fn a_database_that_does_not_exist_needs_setup_and_opens_no_socket() {
    let db = FakeDb::absent();
    let server = serving();
    assert_eq!(poll(&on(), &db, &server).await.state, ViewState::NeedsSetup);
    assert!(!db.path().exists(), "reading must not create the file");
    server.assert_untouched();
}

#[tokio::test]
async fn a_missing_or_unusable_token_needs_setup_and_opens_no_socket() {
    let server = serving();
    let put = |value: Box<dyn rusqlite::ToSql>| {
        let db = FakeDb::empty();
        db.writer()
            .execute("INSERT INTO ItemTable VALUES ('cursorAuth/accessToken', ?1)", [value])
            .unwrap();
        db
    };
    let dbs = [
        ("no row", FakeDb::empty()),
        ("empty", put(Box::new(""))),
        ("blank", put(Box::new("   \n"))),
        ("not a jwt", put(Box::new("just-some-text"))),
        ("two segments", put(Box::new("abc.def"))),
        ("an integer", put(Box::new(42_i64))),
        ("null", put(Box::new(rusqlite::types::Null))),
        ("oversized", put(Box::new("a".repeat(20_000)))),
        ("no sub", put(Box::new(token("no_sub")))),
        ("expired", put(Box::new(token("expired")))),
        ("expires in 30 s", put(Box::new(token("expires_in_30s")))),
    ];
    for (name, db) in &dbs {
        let view = poll(&on(), db, &server).await;
        assert_eq!(view.state, ViewState::NeedsSetup, "{name}");
        assert!(view.windows.is_empty() && view.error.is_none(), "{name}");
    }
    server.assert_untouched();
}

#[tokio::test]
async fn a_corrupt_or_foreign_database_is_an_error_view_with_a_fixed_reason() {
    let server = serving();
    let db = FakeDb::absent();
    std::fs::write(db.path(), "this is not an sqlite database. ".repeat(200)).unwrap();
    let view = poll(&on(), &db, &server).await;
    assert_eq!(view.state, ViewState::Error);
    let error = view.error.unwrap();
    assert_eq!(error.params["reason"], "not_a_database");

    let schema = FakeDb::absent();
    Connection::open(schema.path()).unwrap().execute_batch("CREATE TABLE Other (a, b)").unwrap();
    let view = poll(&on(), &schema, &server).await;
    assert_eq!(view.error.unwrap().params["reason"], "schema");
    server.assert_untouched();
}
