use super::*;

#[test]
fn view_serializes_every_field_in_camel_case() {
    let view = HarnessView::with_state(HarnessId::Codex, ViewState::NoData);
    let json = serde_json::to_value(&view).expect("serialize");
    let obj = json.as_object().expect("object");
    for key in [
        "id", "state", "source", "readAt", "stale", "plan", "windows", "facts", "notices", "error",
        "meta",
    ] {
        assert!(obj.contains_key(key), "missing key {key}");
    }
    assert_eq!(obj["state"], "no_data");
    assert_eq!(obj["readAt"], serde_json::Value::Null);
}

#[test]
fn window_keeps_unknown_percentage_as_null() {
    let window = UsageWindow::new("five_hour", WindowKind::FiveHour);
    let json = serde_json::to_value(&window).expect("serialize");
    assert_eq!(json["usedPct"], serde_json::Value::Null);
    assert_eq!(json["kind"], "five_hour");
}

#[test]
fn error_codes_are_unique_snake_case() {
    let mut seen = std::collections::BTreeSet::new();
    for code in ErrorCode::ALL {
        let name = serde_json::to_value(code).expect("serialize");
        let name = name.as_str().expect("string").to_owned();
        assert!(name.chars().all(|c| c.is_ascii_lowercase() || c == '_'), "{name}");
        assert!(seen.insert(name.clone()), "duplicate {name}");
    }
    assert_eq!(seen.len(), ErrorCode::ALL.len());
}
