//! Pins the Rust model to the shared files in `contract/`. The TypeScript side (`src/lib/contract`)
//! parses the same files, so a shape change must be made on both sides or a test fails.
//! Regenerate the Rust-owned files with `UPDATE_GOLDEN=1 cargo test -p usage-model --test golden`.

use std::path::PathBuf;

use serde_json::{Value, json};
use usage_model::settings::{LitellmScope, TlsMode};
use usage_model::{
    ErrorCode, ExternalLink, HarnessHistory, HarnessId, HistoryEpoch, HistoryPoint, SecretKind,
    SecretStatus, Settings, ShimState, ShimStatus, Snapshot, UpdateFailure, UpdatePhase,
    UpdateStatus,
};

const NOW: i64 = 1_800_000_000;

fn contract_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../contract").join(name)
}

fn read_json(name: &str) -> Value {
    let text = std::fs::read_to_string(contract_path(name))
        .unwrap_or_else(|err| panic!("cannot read contract/{name}: {err}"));
    serde_json::from_str(&text).unwrap_or_else(|err| panic!("contract/{name} is not JSON: {err}"))
}

/// Rust-owned goldens: compared by default, rewritten when `UPDATE_GOLDEN` is set.
fn check_golden(name: &str, actual: &Value) {
    let path = contract_path(name);
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        let text = serde_json::to_string_pretty(actual).unwrap_or_default();
        std::fs::write(&path, format!("{text}\n")).unwrap_or_else(|err| panic!("{name}: {err}"));
        return;
    }
    assert_eq!(
        &read_json(name),
        actual,
        "contract/{name} is out of date; run `UPDATE_GOLDEN=1 cargo test -p usage-model --test golden` \
         and update src/lib/contract to match"
    );
}

#[test]
fn error_codes_match_the_shared_list() {
    let actual: Vec<Value> =
        ErrorCode::ALL.iter().map(|c| serde_json::to_value(c).unwrap_or(Value::Null)).collect();
    assert_eq!(read_json("error-codes.json"), Value::Array(actual));
}

#[test]
fn snapshot_shape_is_pinned() {
    let snapshot = Snapshot { generated_at: NOW, views: views::sample_views() };
    check_golden("snapshot.json", &serde_json::to_value(snapshot).unwrap_or(Value::Null));
}

#[test]
fn settings_shapes_are_pinned() {
    let mut custom_ca = Settings::default();
    custom_ca.general.tray_source = Some(HarnessId::Opencode);
    custom_ca.general.pinned = vec![HarnessId::Codex, HarnessId::Claude];
    custom_ca.claude.config_dir = Some(r"D:\profiles\claude".into());
    custom_ca.litellm.base_url = Some("https://gateway.example.dev/litellm".into());
    custom_ca.litellm.scope = LitellmScope::Team;
    custom_ca.litellm.entity_id = Some("team-a".into());
    custom_ca.litellm.tls = TlsMode::CustomCa { pem_path: r"C:\certs\corp.pem".into() };

    let mut insecure = Settings::default();
    insecure.litellm.tls = TlsMode::Insecure { host: "gateway.example.dev".into() };

    let value = |s: &Settings| serde_json::to_value(s).unwrap_or(Value::Null);
    check_golden(
        "settings.json",
        &json!({ "default": value(&Settings::default()), "customCa": value(&custom_ca), "insecure": value(&insecure) }),
    );
}

fn sample_history() -> HarnessHistory {
    let point = |minutes: i64, used_pct: f64| HistoryPoint { t: NOW + minutes * 60, used_pct };
    HarnessHistory {
        harness: HarnessId::Claude,
        epochs: vec![
            HistoryEpoch {
                window_id: "five_hour".into(),
                resets_at: NOW + 60,
                points: vec![point(-300, 0.0), point(-120, 41.5), point(0, 78.0)],
            },
            HistoryEpoch { window_id: "weekly".into(), resets_at: NOW + 3_600, points: Vec::new() },
        ],
    }
}

#[test]
fn ipc_payload_shapes_are_pinned() {
    let shims: Vec<ShimStatus> = ShimState::ALL
        .iter()
        .map(|&state| ShimStatus {
            state,
            chained: state == ShimState::Installed,
            settings_path: Some(r"C:\Users\someone\.claude\settings.json".into()),
        })
        .collect();
    check_golden(
        "ipc.json",
        &json!({
            "secretStatus": SecretStatus {
                litellm: true,
                opencode: false,
                kimi: true,
                zai: false,
                openrouter: true,
                deepseek: false,
                deepinfra: true,
                moonshot: false,
                novita: false,
                ollama: true,
                minimax: false,
            },
            "secretKinds": SecretKind::ALL,
            "externalLinks": ExternalLink::ALL,
            "shimStates": ShimState::ALL,
            "shimStatuses": shims,
            "history": sample_history(),
            "updatePhases": UpdatePhase::ALL,
            "updateFailures": UpdateFailure::ALL,
            "updateStatus": UpdateStatus::new("0.2.0", true),
        }),
    );
}

#[path = "golden/vectors.rs"]
mod vectors;
#[path = "golden/views.rs"]
mod views;
