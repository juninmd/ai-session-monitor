//! Pins the Rust model to the shared files in `contract/`. The TypeScript side (`src/lib/contract`)
//! parses the same files, so a shape change must be made on both sides or a test fails.
//! Regenerate the Rust-owned files with `UPDATE_GOLDEN=1 cargo test -p usage-model --test golden`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::{Value, json};
use usage_model::settings::{LitellmScope, TlsMode};
use usage_model::view::WindowKind;
use usage_model::{
    Amount, AmountUnit, ErrorCode, ExternalLink, Fact, HarnessError, HarnessHistory, HarnessId,
    HarnessView, HistoryEpoch, HistoryPoint, Notice, SecretKind, SecretStatus, Settings, Severity,
    ShimState, ShimStatus, Snapshot, Source, UsageWindow, ViewState,
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

fn window(
    kind: WindowKind,
    id: &str,
    minutes: Option<u32>,
    used: f64,
    left_min: i64,
) -> UsageWindow {
    let mut w = UsageWindow::new(id, kind);
    w.window_minutes = minutes;
    w.used_pct = Some(used);
    w.resets_at = Some(NOW + left_min * 60);
    w
}

fn ready(id: HarnessId, source: &str, official: bool) -> HarnessView {
    let mut view = HarnessView::with_state(id, ViewState::Ready);
    view.source = Some(Source { id: source.into(), official });
    view.read_at = Some(NOW - 30);
    view
}

fn sample_views() -> Vec<HarnessView> {
    let mut claude = ready(HarnessId::Claude, "statusline", true);
    claude.plan = Some("Max 5x".into());
    claude.windows = vec![
        window(WindowKind::FiveHour, "five_hour", Some(300), 48.0, 113),
        window(WindowKind::Weekly, "weekly", Some(10_080), 45.0, 5_980),
    ];

    let mut antigravity = ready(HarnessId::Antigravity, "language_server", false);
    let mut gemini = window(WindowKind::FiveHour, "gemini:five_hour", Some(300), 54.0, 132);
    gemini.group = Some("gemini".into());
    gemini.label = Some("Gemini Pro / Flash".into());
    let mut claude_gpt = window(WindowKind::Weekly, "claude_gpt:weekly", Some(10_080), 20.0, 4_000);
    claude_gpt.group = Some("claude_gpt".into());
    antigravity.windows = vec![gemini, claude_gpt];
    antigravity.stale = true;
    antigravity.error =
        Some(HarnessError::new(ErrorCode::RateLimited, true).with_param("retryAfterSecs", "120"));
    antigravity.next_attempt_at = Some(1_800_000_300);

    let mut opencode = ready(HarnessId::Opencode, "go_usage", false);
    opencode.windows = vec![
        window(WindowKind::FiveHour, "rolling", Some(300), 12.0, 250),
        window(WindowKind::Weekly, "weekly", Some(10_080), 34.0, 3_000),
        {
            let mut monthly = window(WindowKind::Monthly, "monthly", None, 100.0, 9_000);
            monthly.used_pct = Some(100.0);
            monthly
        },
    ];
    opencode.facts = vec![
        Fact {
            id: "local_cost_5h".into(),
            number: Some(0.0037),
            text: None,
            unit: Some("usd".into()),
        },
        Fact { id: "plan_note".into(), number: None, text: Some("Go".into()), unit: None },
    ];
    opencode.notices = vec![Notice {
        code: "opencode_window_idle".into(),
        severity: Severity::Info,
        params: BTreeMap::new(),
    }];

    let mut litellm = ready(HarnessId::Litellm, "key_info", true);
    let mut budget = window(WindowKind::Budget, "budget", None, 25.0, 20_000);
    budget.label = Some("team-a".into());
    budget.amount = Some(Amount { used: 12.5, limit: Some(50.0), unit: AmountUnit::Usd });
    litellm.windows = vec![budget];
    litellm.meta.insert("host".into(), "gateway.example.dev".into());
    litellm.meta.insert("scope".into(), "key".into());

    let mut unknown_pct = ready(HarnessId::Codex, "app_server", true);
    unknown_pct.windows = vec![UsageWindow::new("five_hour", WindowKind::FiveHour)];

    vec![
        claude,
        HarnessView::needs_consent(HarnessId::Codex),
        antigravity,
        opencode,
        litellm,
        unknown_pct,
        HarnessView::loading(HarnessId::Claude),
        HarnessView::disabled(HarnessId::Antigravity),
        HarnessView::needs_setup(HarnessId::Litellm),
        HarnessView::with_state(HarnessId::Opencode, ViewState::NoData),
        HarnessView::error(
            HarnessId::Litellm,
            HarnessError::new(ErrorCode::Unauthorized, false).with_param("httpStatus", "401"),
        ),
    ]
}

#[test]
fn snapshot_shape_is_pinned() {
    let snapshot = Snapshot { generated_at: NOW, views: sample_views() };
    check_golden("snapshot.json", &serde_json::to_value(snapshot).unwrap_or(Value::Null));
}

#[test]
fn settings_shapes_are_pinned() {
    let mut custom_ca = Settings::default();
    custom_ca.general.tray_source = Some(HarnessId::Opencode);
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
            "secretStatus": SecretStatus { litellm: true, opencode: false },
            "secretKinds": SecretKind::ALL,
            "externalLinks": ExternalLink::ALL,
            "shimStates": ShimState::ALL,
            "shimStatuses": shims,
            "history": sample_history(),
        }),
    );
}

#[path = "golden/vectors.rs"]
mod vectors;
