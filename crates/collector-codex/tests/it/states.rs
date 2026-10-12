//! Every view the collector can emit, checked against the same invariants, and the set of states
//! it uses: Disabled, NeedsConsent, Ready and Error.

use std::collections::BTreeSet;

use tokio::task::JoinSet;
use usage_model::settings::CodexSettings;
use usage_model::{HarnessView, ViewState};

use crate::common::{NOW, Rig, rollout};

const MODES: [&str; 22] = [
    "ok",
    "plus_only",
    "pro_no_primary",
    "api_key",
    "api_key_rpc",
    "no_auth_provider",
    "not_signed_in",
    "not_signed_in_rpc",
    "bad_handshake",
    "bad_handshake_result",
    "garbage_line",
    "huge_line",
    "exits_early",
    "exits_midway",
    "backend_401",
    "backend_500",
    "offline",
    "bad_shape",
    "bad_numbers",
    "noisy",
    "slow",
    "backend_429",
];

fn check_invariants(view: &HarnessView, what: &str) {
    match view.state {
        ViewState::Error => {
            let error =
                view.error.as_ref().unwrap_or_else(|| panic!("{what}: error view without error"));
            assert!(view.windows.is_empty() && view.source.is_none(), "{what}");
            assert!(error.params.values().all(|v| v.len() <= 32), "{what}: {:?}", error.params);
        }
        ViewState::Ready => {
            assert!(view.error.is_none(), "{what}");
            assert!(view.source.is_some() && view.read_at.is_some(), "{what}");
        }
        ViewState::Disabled | ViewState::NeedsConsent => {
            assert_eq!(
                view,
                &HarnessView::with_state(view.id, view.state),
                "{what}: gates carry no data"
            );
        }
        other => panic!("{what}: the collector never emits {other:?}"),
    }
    let mut ids = BTreeSet::new();
    for window in &view.windows {
        assert!(ids.insert(window.id.clone()), "{what}: duplicate window id {}", window.id);
        let used = window.used_pct.unwrap_or(0.0);
        assert!(used.is_finite() && (0.0..=1000.0).contains(&used), "{what}: {window:?}");
        assert!(window.resets_at.is_none_or(|secs| secs > 0), "{what}: {window:?}");
        assert!(window.window_minutes.is_none_or(|m| m > 0), "{what}: {window:?}");
    }
    for notice in &view.notices {
        assert!(
            collector_codex::NOTICE_CODES.contains(&notice.code.as_str()),
            "{what}: {notice:?}"
        );
    }
    assert_eq!(
        view.stale,
        !view.windows.is_empty()
            && view.windows.iter().all(|w| w.resets_at.is_some_and(|at| at <= NOW)),
        "{what}"
    );
}

#[tokio::test]
async fn every_view_obeys_the_same_invariants_and_all_four_states_occur() {
    let mut reads = JoinSet::new();
    for mode in MODES {
        reads.spawn(async move {
            let rig = Rig::new(mode);
            let line = rollout::event("2026-10-10T19:59:00.000Z", Some((31.0, NOW + 100)), None);
            rollout::write_session(rig.home.path(), "2026/10/10", "rollout-a.jsonl", &[line]);
            (mode, rig.collect().await)
        });
    }
    // Settings built from the rig, never from the defaults: those search the real PATH.
    let rig = Rig::without_binary();
    let off = CodexSettings { enabled: false, ..rig.settings() };
    let unconsented = CodexSettings { consent: false, ..rig.settings() };
    let mut views = vec![
        ("disabled", rig.collect_with(&off).await),
        ("needs_consent", rig.collect_with(&unconsented).await),
    ];
    while let Some(read) = reads.join_next().await {
        views.push(read.unwrap());
    }
    let mut seen = BTreeSet::new();
    for (what, view) in &views {
        check_invariants(view, what);
        seen.insert(format!("{:?}", view.state));
    }
    assert_eq!(
        seen.into_iter().collect::<Vec<_>>(),
        ["Disabled", "Error", "NeedsConsent", "Ready"]
    );
}
