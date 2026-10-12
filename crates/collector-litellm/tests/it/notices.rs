//! The collector's vocabulary: every notice code it can emit is declared, every declared code is
//! reachable, and the view states it produces are the ones its contract allows.

use std::collections::BTreeSet;

use collector_litellm::NOTICE_CODES;
use serde_json::{Map, Value, json};
use usage_model::settings::LitellmSettings;
use usage_model::{HarnessView, Severity, ViewState};

use crate::common::{Fake, Reply, codes, run, settings};

async fn key_scope(key: &str) -> HarnessView {
    let fake = Fake::routes([("/key/info", Reply::json(key))]).await;
    run(&settings(&fake.base())).await
}

/// 40 per-model caps: more than a panel can list.
fn many_models() -> String {
    let models: Map<String, Value> = (0..40)
        .map(|i| (format!("m{i:02}"), json!({ "current_spend": i, "budget_limit": 100 })))
        .collect();
    json!({ "info": { "spend": 1.0, "max_budget": 10.0, "model_max_budget_usage": models } })
        .to_string()
}

async fn unreadable_user() -> HarnessView {
    let fake = Fake::routes([
        ("/key/info", Reply::json(fixture!("key_follow_up.json"))),
        ("/user/info", Reply::json_status(403, "{}")),
        ("/team/info", Reply::json(fixture!("team_info_no_budget.json"))),
    ])
    .await;
    run(&settings(&fake.base())).await
}

#[test]
fn the_declared_codes_are_unique_and_namespaced() {
    let unique: BTreeSet<_> = NOTICE_CODES.iter().collect();
    assert_eq!(unique.len(), NOTICE_CODES.len(), "{NOTICE_CODES:?}");
    for code in NOTICE_CODES {
        assert!(code.starts_with("litellm_"), "{code}");
        assert!(code.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'), "{code}");
    }
}

#[tokio::test]
async fn every_emitted_code_is_declared_and_every_declared_code_is_emitted() {
    let views = [
        key_scope(fixture!("key_no_budget.json")).await,
        key_scope(fixture!("key_with_budget.json")).await,
        key_scope(fixture!("model_budgets.json")).await,
        key_scope(&many_models()).await,
        unreadable_user().await,
    ];
    let emitted: BTreeSet<&str> = views.iter().flat_map(codes).collect();
    let declared: BTreeSet<&str> = NOTICE_CODES.iter().copied().collect();
    assert_eq!(emitted, declared);
}

#[tokio::test]
async fn each_notice_has_the_severity_and_params_the_ui_expects() {
    let none = key_scope(fixture!("key_no_budget.json")).await;
    assert_eq!((none.notices[0].severity, none.notices[0].params.len()), (Severity::Info, 0));

    let cut = key_scope(&many_models()).await;
    let [notice] = cut.notices.as_slice() else { panic!("{:?}", cut.notices) };
    assert_eq!(
        (notice.code.as_str(), notice.severity),
        ("litellm_windows_truncated", Severity::Info)
    );
    assert_eq!(notice.params.get("dropped").map(String::as_str), Some("24"));
    assert_eq!(notice.params.len(), 1);
    assert_eq!(cut.windows.len(), 1 + 16, "the headline plus the sixteen most consumed");
    assert_eq!(cut.windows[1].id, "model:m39");

    let unreadable = unreadable_user().await;
    let [notice] = unreadable.notices.as_slice() else { panic!("{:?}", unreadable.notices) };
    assert_eq!(
        (notice.code.as_str(), notice.severity),
        ("litellm_level_unreadable", Severity::Warn)
    );
    let params: Vec<_> = notice.params.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    assert_eq!(params, [("level", "user"), ("reason", "unauthorized")]);
}

#[tokio::test]
async fn the_collector_only_ever_produces_the_states_its_contract_allows() {
    let fake = Fake::routes([("/key/info", Reply::json(fixture!("key_with_budget.json")))]).await;
    let broken = Fake::routes([("/key/info", Reply::json_status(500, "{}"))]).await;
    let states: BTreeSet<String> = [
        run(&LitellmSettings { enabled: false, ..settings(&fake.base()) }).await,
        run(&LitellmSettings::default()).await,
        run(&settings(&fake.base())).await,
        run(&settings(&broken.base())).await,
    ]
    .iter()
    .map(|view| format!("{:?}", view.state))
    .collect();
    let expected = [ViewState::Disabled, ViewState::NeedsSetup, ViewState::Ready, ViewState::Error];
    let expected: BTreeSet<String> = expected.iter().map(|state| format!("{state:?}")).collect();
    assert_eq!(states, expected, "no loading, consent or no-data view comes from this collector");
}
