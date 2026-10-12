use super::*;

#[test]
fn empty_json_yields_defaults() {
    let settings: Settings = serde_json::from_str("{}").expect("parse");
    assert_eq!(settings, Settings::default());
    assert!(!settings.claude.oauth_usage, "the unofficial source must be opt-in");
    assert!(!settings.general.autostart, "autostart must be opt-in");
}

#[test]
fn unknown_keys_are_ignored_and_known_ones_kept() {
    let json = r#"{"general":{"intervalSecs":120,"futureKey":1},"litellm":{"scope":"team","entityId":"t-1"}}"#;
    let settings: Settings = serde_json::from_str(json).expect("parse");
    assert_eq!(settings.general.interval_secs, 120);
    assert_eq!(settings.litellm.scope, LitellmScope::Team);
    assert_eq!(settings.litellm.entity_id.as_deref(), Some("t-1"));
    assert!(settings.codex.enabled);
}

#[test]
fn normalize_clamps_out_of_range_values() {
    let mut settings = Settings::default();
    settings.general.interval_secs = 1;
    settings.general.warn_at_pct = 99;
    settings.litellm.poll_interval_secs = 5;
    settings.litellm.timeout_secs = 999;
    settings.normalize();
    assert_eq!(settings.general.interval_secs, MIN_INTERVAL_SECS);
    assert_eq!(settings.general.warn_at_pct, 90);
    assert_eq!(settings.litellm.poll_interval_secs, MIN_LITELLM_POLL_SECS);
    assert_eq!(settings.litellm.timeout_secs, 30);
}

#[test]
fn tls_mode_round_trips_with_a_mode_tag() {
    let tls = TlsMode::Insecure { host: "gateway.example.dev".into() };
    let json = serde_json::to_string(&tls).expect("serialize");
    assert_eq!(json, r#"{"mode":"insecure","host":"gateway.example.dev"}"#);
    assert_eq!(serde_json::from_str::<TlsMode>(&json).expect("parse"), tls);
}

#[test]
fn custom_ca_field_is_camel_case() {
    let tls = TlsMode::CustomCa { pem_path: r"C:\certs\corp.pem".into() };
    let json = serde_json::to_value(&tls).expect("serialize");
    assert_eq!(json["mode"], "custom_ca");
    assert_eq!(json["pemPath"], r"C:\certs\corp.pem");
}

#[test]
fn settings_hold_no_secret_fields() {
    let json = serde_json::to_string(&Settings::default()).expect("serialize").to_lowercase();
    for forbidden in ["apikey", "token", "password", "secret"] {
        assert!(!json.contains(forbidden), "settings JSON mentions {forbidden}");
    }
}

#[test]
fn threshold_alerts_default_on_at_80_and_95_and_adaptive_polling_is_on() {
    let general = Settings::default().general;
    assert_eq!(general.alert_thresholds, [80, 95]);
    assert!(general.adaptive);
}

#[test]
fn an_older_file_without_the_new_keys_gets_the_defaults() {
    let settings: Settings =
        serde_json::from_str(r#"{"general":{"intervalSecs":90}}"#).expect("parse");
    assert_eq!(settings.general.alert_thresholds, [80, 95]);
    assert!(settings.general.adaptive);
}

#[test]
fn thresholds_are_validated_sorted_deduped_and_capped() {
    let cases: [(&[u8], &[u8]); 6] = [
        (&[95, 80, 80], &[80, 95]),
        (&[0, 100, 255, 50], &[50]),
        (&[], &[]),
        (&[99, 1], &[1, 99]),
        (&[90, 10, 70, 30, 50, 20], &[10, 20, 30, 50]),
        (&[0, 100], &[]),
    ];
    for (given, expected) in cases {
        let mut settings = Settings::default();
        settings.general.alert_thresholds = given.to_vec();
        settings.normalize();
        assert_eq!(settings.general.alert_thresholds, expected, "{given:?}");
    }
}
