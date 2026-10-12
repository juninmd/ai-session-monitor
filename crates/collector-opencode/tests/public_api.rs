//! The crate as the app sees it: public surface only, no network, and a fabricated database in a
//! temp dir. Calls that would need the real Go endpoint are deliberately absent.

use std::collections::BTreeSet;
use std::path::Path;

use collector_opencode::{NOTICE_CODES, collect};
use rusqlite::Connection;
use usage_model::settings::OpencodeSettings;
use usage_model::{Ctx, HarnessId, ViewState};
use usage_support::Secret;

const NOW: i64 = 1_800_000_000;

fn ctx(home: &Path) -> Ctx {
    Ctx::new(NOW, home.join("app-data"), home)
}

fn settings(enabled: bool, local_consent: bool, data_dir: Option<&Path>) -> OpencodeSettings {
    OpencodeSettings {
        enabled,
        local_consent,
        data_dir: data_dir.map(|dir| dir.to_string_lossy().into_owned()),
    }
}

#[tokio::test]
async fn the_gates_hold_through_the_public_entry_point() {
    let home = tempfile::tempdir().unwrap();
    let key = Secret::new("fab-public-key");

    let off = collect(&settings(false, true, None), Some(&key), &ctx(home.path())).await;
    assert_eq!((off.id, off.state), (HarnessId::Opencode, ViewState::Disabled));

    let unset = collect(&settings(true, false, None), None, &ctx(home.path())).await;
    assert_eq!((unset.id, unset.state), (HarnessId::Opencode, ViewState::NeedsSetup));
    let wire = serde_json::to_value(&unset).unwrap();
    assert_eq!(
        (wire["id"].as_str(), wire["state"].as_str()),
        (Some("opencode"), Some("needs_setup"))
    );
}

#[tokio::test]
async fn the_local_estimate_works_without_a_key_and_without_the_network() {
    let (home, data) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let conn = Connection::open(data.path().join("opencode.db")).unwrap();
    conn.execute_batch(
        "CREATE TABLE session_message (id TEXT PRIMARY KEY, session_id TEXT NOT NULL,
           type TEXT NOT NULL, seq INTEGER NOT NULL, time_created INTEGER NOT NULL,
           time_updated INTEGER NOT NULL, data TEXT NOT NULL);",
    )
    .unwrap();
    let at = (NOW - 60) * 1_000;
    let row =
        format!(r#"{{"cost":0.5,"tokens":{{"input":7,"output":3}},"time":{{"created":{at}}}}}"#);
    conn.execute(
        "INSERT INTO session_message VALUES ('m1', 's1', 'assistant', 1, ?1, ?1, ?2)",
        rusqlite::params![at, row],
    )
    .unwrap();
    drop(conn);

    let view = collect(&settings(true, true, Some(data.path())), None, &ctx(home.path())).await;
    assert_eq!(view.state, ViewState::Ready);
    assert_eq!(
        view.source.as_ref().map(|s| (s.id.as_str(), s.official)),
        Some(("local_db", false))
    );
    let number = |id: &str| view.facts.iter().find(|f| f.id == id).and_then(|f| f.number);
    assert_eq!((number("local_cost_5h"), number("local_tokens_5h")), (Some(0.5), Some(10.0)));
    assert!(view.windows.is_empty());
}

#[test]
fn notice_codes_are_unique_prefixed_and_listed_in_the_ipc_catalog() {
    let unique: BTreeSet<_> = NOTICE_CODES.iter().collect();
    assert_eq!(unique.len(), NOTICE_CODES.len());
    let catalog = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/ipc.md");
    let catalog = std::fs::read_to_string(catalog).unwrap();
    for code in NOTICE_CODES {
        assert!(code.starts_with("opencode_"), "{code}");
        assert!(catalog.contains(&format!("`{code}`")), "{code} is not in docs/ipc.md");
    }
}

fn assert_send<T: Send>(_: &T) {}

#[test]
fn the_collect_future_is_send_so_the_scheduler_can_box_it() {
    let (settings, ctx) = (OpencodeSettings::default(), Ctx::new(NOW, "app-data", "home"));
    let key = Secret::new("fab-send-key");
    // Built and dropped without polling: only the type is checked.
    assert_send(&collect(&settings, Some(&key), &ctx));
}
