//! The crate as the app sees it: public surface only. Only the gates are exercised here, because
//! past them the default path under the home directory (or `XDG_CONFIG_HOME`) would be read; the
//! database, token and request flows are covered by the unit suites against temp dirs and a
//! loopback server.

use collector_cursor::{NOTICE_CODES, collect};
use usage_model::settings::CursorSettings;
use usage_model::{Ctx, HarnessId, ViewState};

const NOW: i64 = 1_800_000_000;

#[tokio::test]
async fn the_gates_hold_through_the_public_entry_point() {
    let home = tempfile::tempdir().unwrap();
    let ctx = Ctx::new(NOW, home.path().join("app-data"), home.path());

    let off = collect(&CursorSettings { enabled: false, consent: true }, &ctx).await;
    assert_eq!((off.id, off.state), (HarnessId::Cursor, ViewState::Disabled));

    let unconsented = collect(&CursorSettings { enabled: true, consent: false }, &ctx).await;
    assert_eq!((unconsented.id, unconsented.state), (HarnessId::Cursor, ViewState::NeedsConsent));
    let wire = serde_json::to_value(&unconsented).unwrap();
    assert_eq!(
        (wire["id"].as_str(), wire["state"].as_str()),
        (Some("cursor"), Some("needs_consent"))
    );
    assert!(std::fs::read_dir(home.path()).unwrap().next().is_none(), "nothing was created");
}

#[test]
fn this_collector_raises_no_notices() {
    assert!(NOTICE_CODES.is_empty());
}
