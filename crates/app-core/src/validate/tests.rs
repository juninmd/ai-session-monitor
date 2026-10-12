use usage_model::settings::{LitellmScope, TlsMode};

use super::*;

pub(super) fn abs(name: &str) -> String {
    std::env::temp_dir().join(name).to_string_lossy().into_owned()
}

fn with_url(url: &str) -> Settings {
    let mut settings = Settings::default();
    settings.litellm.base_url = Some(url.to_owned());
    settings
}

pub(super) fn reject(settings: &Settings, code: ErrorCode, field: &str) {
    let error = validate(settings).expect_err("must be rejected");
    assert_eq!(error.code, code, "field {field}");
    assert!(!error.retryable, "the user has to fix this, retrying cannot help");
    assert_eq!(error.params.len(), 1, "params must carry the field name only: {error:?}");
    assert_eq!(error.params.get("field").map(String::as_str), Some(field));
}

#[test]
fn defaults_and_a_complete_connection_are_valid() {
    assert!(validate(&Settings::default()).is_ok());
    let mut settings = with_url("https://gateway.example.dev/litellm");
    settings.litellm.scope = LitellmScope::Team;
    settings.litellm.entity_id = Some("equipe-ação-🚀".into());
    settings.litellm.auth_header = "X-Api-Key".into();
    settings.litellm.tls = TlsMode::CustomCa { pem_path: abs("corp-ca.pem") };
    settings.claude.config_dir = Some(abs("claude"));
    settings.codex.binary_path = Some(abs("codex.cmd"));
    settings.opencode.data_dir = Some(abs("opencode"));
    assert!(validate(&settings).is_ok());
}

#[test]
fn base_url_accepts_https_with_prefix_and_port_and_http_only_to_loopback() {
    for url in [
        "https://gateway.example.dev",
        "https://gateway.example.dev/",
        "https://gateway.example.dev/litellm",
        "https://gateway.example.dev:8443/a/b",
        "https://10.0.0.5",
        "HTTPS://GATEWAY.EXAMPLE.DEV",
        "http://localhost:4000",
        "http://127.0.0.1:4000/x",
        "http://[::1]:4000",
    ] {
        assert!(validate(&with_url(url)).is_ok(), "{url} should be accepted");
    }
}

#[test]
fn base_url_rejections() {
    let too_long = format!("https://gateway.example.dev/{}", "a".repeat(5000));
    for url in [
        "http://gateway.example.dev",
        "http://localhost.evil.example",
        "http://127.0.0.1.evil.example",
        "ftp://gateway.example.dev",
        "file:///etc/passwd",
        "javascript:alert(1)",
        "gateway.example.dev",
        "https://",
        "",
        "https://user:pw@gateway.example.dev",
        "https://user@gateway.example.dev",
        "https://gateway.example.dev/?key=1",
        "https://gateway.example.dev?",
        "https://gateway.example.dev/#frag",
        "https://gateway.example.dev/#",
        " https://gateway.example.dev",
        "https://gateway.example.dev ",
        "https://gateway.example.dev/\n",
        too_long.as_str(),
    ] {
        reject(&with_url(url), ErrorCode::InvalidUrl, "baseUrl");
    }
}

#[test]
fn an_unset_base_url_is_valid_because_the_tab_then_asks_for_setup() {
    assert!(Settings::default().litellm.base_url.is_none());
    assert!(validate(&Settings::default()).is_ok());
}

#[test]
fn entity_id_is_required_for_user_and_team_scopes_only() {
    for scope in [LitellmScope::User, LitellmScope::Team] {
        for missing in [None, Some(""), Some("   "), Some("\t")] {
            let mut settings = Settings::default();
            settings.litellm.scope = scope;
            settings.litellm.entity_id = missing.map(str::to_owned);
            reject(&settings, ErrorCode::InvalidInput, "entityId");
        }
        let mut settings = Settings::default();
        settings.litellm.scope = scope;
        settings.litellm.entity_id = Some("u-1".into());
        assert!(validate(&settings).is_ok());
    }
    for ignored in [None, Some(""), Some("  ")] {
        let mut settings = Settings::default();
        settings.litellm.entity_id = ignored.map(str::to_owned);
        assert!(validate(&settings).is_ok(), "key scope ignores {ignored:?}");
    }
}

#[test]
fn entity_id_is_bounded_and_free_of_control_characters_whatever_the_scope() {
    for scope in [LitellmScope::Key, LitellmScope::User] {
        let mut settings = Settings::default();
        settings.litellm.scope = scope;
        for bad in ["x".repeat(257), "a\nb".to_owned(), "tab\there".to_owned(), "nul\0".to_owned()]
        {
            settings.litellm.entity_id = Some(bad);
            reject(&settings, ErrorCode::InvalidInput, "entityId");
        }
        settings.litellm.entity_id = Some("x".repeat(256));
        assert!(validate(&settings).is_ok(), "256 characters is the limit");
        settings.litellm.entity_id = Some("ç".repeat(256));
        assert!(validate(&settings).is_ok(), "the limit counts characters, not bytes");
        settings.litellm.entity_id = Some("ç".repeat(257));
        reject(&settings, ErrorCode::InvalidInput, "entityId");
    }
}

#[test]
fn insecure_tls_needs_the_host_of_the_base_url() {
    let insecure = |url: Option<&str>, host: &str| {
        let mut settings = Settings::default();
        settings.litellm.base_url = url.map(str::to_owned);
        settings.litellm.tls = TlsMode::Insecure { host: host.to_owned() };
        settings
    };
    for (url, host) in [
        ("https://gateway.example.dev", "gateway.example.dev"),
        ("https://gateway.example.dev:8443/x", "gateway.example.dev"),
        ("https://Gateway.Example.Dev", "gateway.example.dev"),
        ("https://gateway.example.dev", "GATEWAY.example.dev."),
        ("https://gateway.example.dev", " gateway.example.dev "),
        ("http://[::1]:4000", "::1"),
        ("http://[::1]:4000", "[::1]"),
        ("http://127.0.0.1:4000", "127.0.0.1"),
    ] {
        assert!(validate(&insecure(Some(url), host)).is_ok(), "{host} vs {url}");
    }
    for (url, host) in [
        (Some("https://gateway.example.dev"), ""),
        (Some("https://gateway.example.dev"), "   "),
        (Some("https://gateway.example.dev"), "other.example.dev"),
        (Some("https://gateway.example.dev"), "example.dev"),
        (Some("https://gateway.example.dev"), "gateway.example.dev.evil.net"),
        (Some("https://gateway.example.dev"), "gateway.example.dev:8443"),
        (None, "gateway.example.dev"),
        (None, ""),
    ] {
        reject(&insecure(url, host), ErrorCode::InvalidInput, "host");
    }
}

#[test]
fn an_internationalized_host_is_matched_in_its_ascii_form_like_the_client_does() {
    // The URL parser keeps `münchen.example` as `xn--mnchen-3ya.example`, so that is what the
    // confirmed host has to be; the client compares the same spelling.
    let insecure = |url: &str, host: &str| {
        let mut settings = with_url(url);
        settings.litellm.tls = TlsMode::Insecure { host: host.to_owned() };
        settings
    };
    for url in ["https://xn--mnchen-3ya.example/litellm", "https://münchen.example/litellm"] {
        assert!(validate(&insecure(url, "xn--mnchen-3ya.example")).is_ok(), "{url}");
        assert!(validate(&insecure(url, "XN--MNCHEN-3YA.EXAMPLE.")).is_ok(), "{url}");
        reject(&insecure(url, "münchen.example"), ErrorCode::InvalidInput, "host");
    }
}

#[test]
fn the_first_problem_in_form_order_is_the_one_reported() {
    let mut settings = with_url("http://gateway.example.dev");
    settings.litellm.scope = LitellmScope::Team;
    settings.litellm.auth_header = "Bad Header".into();
    settings.litellm.tls = TlsMode::CustomCa { pem_path: "relative.pem".into() };
    settings.claude.config_dir = Some("relative".into());
    reject(&settings, ErrorCode::InvalidUrl, "baseUrl");
    settings.litellm.base_url = Some("https://gateway.example.dev".into());
    reject(&settings, ErrorCode::InvalidInput, "entityId");
    settings.litellm.entity_id = Some("team-a".into());
    reject(&settings, ErrorCode::InvalidInput, "authHeader");
    settings.litellm.auth_header = "Authorization".into();
    reject(&settings, ErrorCode::InvalidInput, "pemPath");
    settings.litellm.tls = TlsMode::System;
    reject(&settings, ErrorCode::InvalidInput, "configDir");
    settings.claude.config_dir = None;
    assert!(validate(&settings).is_ok());
}

#[test]
fn rejections_never_echo_the_rejected_value() {
    let marker = "zz-sensitive-marker-91";
    let mut cases = Vec::new();
    cases.push(with_url(&format!("https://user:{marker}@gateway.example.dev")));
    cases.push(with_url(&format!("http://{marker}.example.dev")));
    let mut header = Settings::default();
    header.litellm.auth_header = format!("{marker} spaced");
    cases.push(header);
    let mut entity = Settings::default();
    entity.litellm.scope = LitellmScope::User;
    entity.litellm.entity_id = Some(format!("{marker}\n"));
    cases.push(entity);
    let mut pem = Settings::default();
    pem.litellm.tls = TlsMode::CustomCa { pem_path: format!("{marker}.pem") };
    cases.push(pem);
    let mut host = with_url("https://gateway.example.dev");
    host.litellm.tls = TlsMode::Insecure { host: format!("{marker}.example.dev") };
    cases.push(host);
    let mut dir = Settings::default();
    dir.claude.config_dir = Some(format!("{marker}-relative"));
    cases.push(dir);
    for settings in cases {
        let error = validate(&settings).expect_err("every case is invalid");
        assert!(!format!("{error:?}").contains(marker), "{error:?}");
    }
}

#[test]
fn exceeds_chars_counts_characters_and_stops_early_on_huge_input() {
    assert!(!exceeds_chars("", 0));
    assert!(exceeds_chars("a", 0));
    assert!(!exceeds_chars("ççç", 3));
    assert!(exceeds_chars("çççç", 3));
    assert!(exceeds_chars(&"a".repeat(1_000_000), 10));
}
