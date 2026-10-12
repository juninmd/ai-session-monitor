//! Consent and the enabled switch are decided before anything else happens: no lookup of the
//! binary, no process, no read of the sessions directory.

use usage_model::settings::CodexSettings;
use usage_model::{HarnessId, HarnessView};

use crate::common::{Rig, rollout};

/// A working fake and a readable rollout, either of which would show if a gate let it through.
fn tempting() -> Rig {
    let rig = Rig::new("ok");
    let line = rollout::event("2026-10-10T19:59:00.000Z", Some((31.0, 1_791_670_500)), None);
    rollout::write_session(rig.home.path(), "2026/10/10", "rollout-a.jsonl", &[line]);
    rig
}

fn nothing_ran(rig: &Rig) {
    assert!(rig.pid().is_none(), "the app-server must not have been started");
    assert!(rig.received().is_empty());
}

#[tokio::test]
async fn a_disabled_source_does_nothing_at_all() {
    let rig = tempting();
    let settings = CodexSettings { enabled: false, ..rig.settings() };
    assert_eq!(rig.collect_with(&settings).await, HarnessView::disabled(HarnessId::Codex));
    nothing_ran(&rig);
}

#[tokio::test]
async fn a_missing_consent_does_nothing_at_all() {
    let rig = tempting();
    let settings = CodexSettings { consent: false, ..rig.settings() };
    assert_eq!(rig.collect_with(&settings).await, HarnessView::needs_consent(HarnessId::Codex));
    nothing_ran(&rig);
}

#[tokio::test]
async fn disabled_wins_over_missing_consent() {
    let rig = tempting();
    let settings = CodexSettings { enabled: false, consent: false, ..rig.settings() };
    assert_eq!(rig.collect_with(&settings).await, HarnessView::disabled(HarnessId::Codex));
    nothing_ran(&rig);
}

#[tokio::test]
async fn the_gates_come_before_any_lookup_of_the_binary() {
    // Nothing named here exists, so any attempt to resolve or start it would be an error view.
    // Never `None` or blank: a broken gate would then search the real PATH, where a real `codex`
    // may be installed, and a test must not be able to start that.
    let rig = Rig::without_binary();
    let absent = rig.exe.to_string_lossy().into_owned();
    let paths = [Some(absent), Some("co\0dex".to_owned()), Some("x".repeat(10_000))];
    for binary_path in paths {
        let gated =
            |enabled, consent| CodexSettings { enabled, consent, binary_path: binary_path.clone() };
        let off = rig.collect_with(&gated(false, true)).await;
        assert_eq!(off, HarnessView::disabled(HarnessId::Codex), "{binary_path:?}");
        let unconsented = rig.collect_with(&gated(true, false)).await;
        assert_eq!(unconsented, HarnessView::needs_consent(HarnessId::Codex), "{binary_path:?}");
    }
    assert!(!rig.home.path().join(".codex").exists());
    assert!(!rig.home.path().join("data").exists());
}

#[tokio::test]
async fn a_gate_that_opens_reads_the_source() {
    // The same rig with both gates open must reach the app-server: the gates are not just "off".
    let rig = tempting();
    let view = rig.collect().await;
    assert_eq!(view.source.map(|source| source.id).as_deref(), Some("app_server"));
    assert!(rig.pid().is_some());
}
