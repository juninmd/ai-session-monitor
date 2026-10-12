//! Gates run before any I/O: a disabled, unconfigured or invalid poll never reaches the network.

use usage_model::settings::{LitellmScope, LitellmSettings, TlsMode};
use usage_model::{ErrorCode, ViewState};
use usage_support::Secret;

use crate::common::{Fake, NOW, Reply, error_of, param, run, run_with, scoped, settings};

/// A proxy that would answer happily, so only a gate can explain silence.
async fn willing_proxy() -> Fake {
    Fake::routes([("/key/info", Reply::json(fixture!("key_with_budget.json")))]).await
}

#[tokio::test]
async fn a_disabled_source_stays_disabled_and_silent() {
    let fake = willing_proxy().await;
    let view = run(&LitellmSettings { enabled: false, ..settings(&fake.base()) }).await;
    assert_eq!(view.state, ViewState::Disabled);
    assert!(view.windows.is_empty() && view.facts.is_empty() && view.notices.is_empty());
    assert!(view.error.is_none() && view.source.is_none());
    fake.assert_untouched().await;
}

#[tokio::test]
async fn disabled_wins_over_missing_configuration() {
    let view = run(&LitellmSettings { enabled: false, ..LitellmSettings::default() }).await;
    assert_eq!(view.state, ViewState::Disabled);
}

#[tokio::test]
async fn a_missing_or_blank_address_needs_setup() {
    for base in [None, Some(""), Some("   "), Some("\t\r\n")] {
        let settings =
            LitellmSettings { base_url: base.map(str::to_owned), ..LitellmSettings::default() };
        let view = run(&settings).await;
        assert_eq!(view.state, ViewState::NeedsSetup, "{base:?}");
        assert!(view.windows.is_empty() && view.error.is_none());
    }
}

#[tokio::test]
async fn a_missing_or_blank_key_needs_setup_and_the_proxy_is_never_called() {
    let fake = willing_proxy().await;
    let settings = settings(&fake.base());
    let blank = [Secret::new(""), Secret::new("   "), Secret::new("\t\r\n")];
    assert_eq!(run_with(&settings, None, NOW).await.state, ViewState::NeedsSetup);
    for key in &blank {
        let view = run_with(&settings, Some(key), NOW).await;
        assert_eq!(view.state, ViewState::NeedsSetup);
        assert!(view.windows.is_empty() && view.error.is_none());
    }
    fake.assert_untouched().await;
}

#[tokio::test]
async fn invalid_settings_are_refused_before_any_network_access() {
    let fake = willing_proxy().await;
    let base = fake.base();
    let entity = |scope, id: &str| scoped(&base, scope, id);
    let header = |name: &str| LitellmSettings { auth_header: name.to_owned(), ..settings(&base) };
    let tls = |tls| LitellmSettings { tls, ..settings(&base) };
    let ca = |path: &str| tls(TlsMode::CustomCa { pem_path: path.to_owned() });
    let insecure = |host: &str| tls(TlsMode::Insecure { host: host.to_owned() });
    let url = |text: String| settings(&text);
    let userinfo = base.replace("http://", "http://admin:decoy-password@");
    let missing_id = |scope| LitellmSettings { scope, entity_id: None, ..settings(&base) };
    let long_id = "x".repeat(257);

    let input = ErrorCode::InvalidInput;
    let cases = [
        ("user scope without an id", missing_id(LitellmScope::User), input, ("field", "entityId")),
        ("team scope without an id", missing_id(LitellmScope::Team), input, ("field", "entityId")),
        ("blank id", entity(LitellmScope::User, "   "), input, ("field", "entityId")),
        ("id with a newline", entity(LitellmScope::Team, "a\nb"), input, ("field", "entityId")),
        (
            "id over 256 characters",
            entity(LitellmScope::User, &long_id),
            input,
            ("field", "entityId"),
        ),
        ("header with a space", header("X Key"), input, ("field", "authHeader")),
        ("header with a colon", header("X:Key"), input, ("field", "authHeader")),
        ("empty header", header(""), input, ("field", "authHeader")),
        ("blank header", header("   "), input, ("field", "authHeader")),
        ("header with unicode", header("X-Clé"), input, ("field", "authHeader")),
        ("header injection", header("X-Key\r\nEvil: 1"), input, ("field", "authHeader")),
        ("relative CA path", ca("certs/ca.pem"), input, ("field", "pemPath")),
        ("blank CA path", ca("   "), input, ("field", "pemPath")),
        ("insecure without a host", insecure(""), input, ("field", "host")),
        ("insecure for another host", insecure("other.example"), input, ("field", "host")),
        (
            "query in the address",
            url(format!("{base}/?token=x")),
            ErrorCode::InvalidUrl,
            ("host", "127.0.0.1"),
        ),
        ("userinfo in the address", url(userinfo), ErrorCode::InvalidUrl, ("host", "127.0.0.1")),
        (
            "fragment in the address",
            url(format!("{base}#top")),
            ErrorCode::InvalidUrl,
            ("host", "127.0.0.1"),
        ),
        (
            "unsupported scheme",
            url("ftp://127.0.0.1:21".to_owned()),
            ErrorCode::InvalidUrl,
            ("host", "127.0.0.1"),
        ),
        (
            "plain http to a remote host",
            url("http://litellm.corp.example".to_owned()),
            ErrorCode::InvalidUrl,
            ("host", "litellm.corp.example"),
        ),
    ];
    for (name, settings, code, (key, value)) in cases {
        let view = run(&settings).await;
        let error = error_of(&view);
        assert_eq!(error.code, code, "{name}");
        assert!(!error.retryable, "{name}");
        assert_eq!(param(error, key), Some(value), "{name}");
        assert_eq!(error.params.len(), 1, "{name}: only the field or host is reported");
    }
    fake.assert_untouched().await;
}

#[tokio::test]
async fn addresses_that_are_not_urls_are_refused_without_a_host() {
    let texts = [
        "litellm",
        "://nohost",
        "http://",
        "https://",
        "https://:443",
        "https://exa mple.test",
        "javascript:alert(1)",
        "http://127.0.0.1:1/a\nb",
    ];
    for text in texts {
        let view = run(&settings(text)).await;
        let error = error_of(&view);
        assert_eq!(error.code, ErrorCode::InvalidUrl, "{text:?}");
        assert!(error.params.is_empty(), "{text:?}: {error:?}");
    }
}

#[tokio::test]
async fn a_key_that_cannot_travel_in_a_header_is_refused_before_the_network() {
    let fake = willing_proxy().await;
    let settings = settings(&fake.base());
    for text in ["sk-trailing-newline\n", "sk-é-accent", "sk-tab\there", "sk-nul\0byte"] {
        let view = run_with(&settings, Some(&Secret::new(text)), NOW).await;
        let error = error_of(&view);
        assert_eq!(error.code, ErrorCode::InvalidInput, "{text:?}");
        assert_eq!(param(error, "field"), Some("value"));
    }
    fake.assert_untouched().await;
}

#[tokio::test]
async fn managed_header_names_are_refused_before_the_network() {
    let fake = willing_proxy().await;
    for name in ["Host", "Content-Length", "Connection", "Transfer-Encoding"] {
        let settings = LitellmSettings { auth_header: name.to_owned(), ..settings(&fake.base()) };
        let view = run(&settings).await;
        let error = error_of(&view);
        assert_eq!(error.code, ErrorCode::InvalidInput, "{name}");
        assert_eq!(param(error, "field"), Some("authHeader"), "{name}");
    }
    fake.assert_untouched().await;
}

#[tokio::test]
async fn out_of_range_timeouts_are_clamped_not_trusted() {
    let fake = willing_proxy().await;
    for secs in [0, 1, 30, 31, u32::MAX] {
        let settings = LitellmSettings { timeout_secs: secs, ..settings(&fake.base()) };
        let view = run(&settings).await;
        assert_eq!(view.state, ViewState::Ready, "timeout_secs = {secs}");
    }
    assert_eq!(fake.requests().len(), 5);
    assert!(fake.requests().iter().all(|seen| seen.method == "GET"));
}
