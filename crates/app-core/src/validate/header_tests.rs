//! The auth header must be a name every consumer takes. One that passes here and fails there would
//! be saved without a field error and then fail every poll.

use usage_support::{RequestSpec, Secret};

use super::tests::reject;
use super::*;

/// The headers the HTTP client owns, spelled out so a deleted entry in the production list fails.
const MANAGED: [&str; 11] = [
    "Host",
    "Content-Length",
    "Content-Type",
    "Transfer-Encoding",
    "Connection",
    "Keep-Alive",
    "Upgrade",
    "TE",
    "Trailer",
    "Proxy-Authorization",
    "Proxy-Connection",
];

fn with_header(name: &str) -> Settings {
    let mut settings = Settings::default();
    settings.litellm.auth_header = name.to_owned();
    settings
}

/// The rule of the LiteLLM collector (`plan.rs`, `header_name`) and of the settings form
/// (`litellmDraft.ts`, `HEADER_NAME`): letters, digits and hyphen.
fn consumers_accept(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

fn http_client_accepts(name: &str) -> bool {
    let spec = RequestSpec::new("https://gateway.example.dev")
        .secret_header(name, &Secret::new("sk-fabricated"));
    format!("{spec:?}").contains("error: None")
}

#[test]
fn letters_digits_and_hyphens_are_accepted() {
    for good in ["Authorization", "X-Api-Key", "api-key", "x-litellm-api-key", "A", "123", "-"] {
        assert!(validate(&with_header(good)).is_ok(), "{good} should be accepted");
    }
}

#[test]
fn every_other_character_is_rejected_even_when_it_is_a_valid_http_token() {
    // RFC 9110 allows these in a token; the collector and the form do not, so neither may we.
    for bad in ["x_custom.key", "X_Api_Key", "api.key", "a+b", "a~b", "a!b", "a'b", "a|b", "a`b"] {
        reject(&with_header(bad), ErrorCode::InvalidInput, "authHeader");
    }
    for bad in ["", " ", " Authorization", "Authorization ", "Bad Header", "a:b", "tab\tname"] {
        reject(&with_header(bad), ErrorCode::InvalidInput, "authHeader");
    }
    for bad in ["new\nline", "nul\0", "Ключ", "chave-ção", "a\u{200b}b"] {
        reject(&with_header(bad), ErrorCode::InvalidInput, "authHeader");
    }
}

#[test]
fn every_managed_header_is_rejected_whatever_its_case() {
    for name in MANAGED {
        for spelling in [name.to_owned(), name.to_lowercase(), name.to_uppercase()] {
            reject(&with_header(&spelling), ErrorCode::InvalidInput, "authHeader");
        }
    }
}

#[test]
fn the_length_limit_is_128_characters() {
    assert!(validate(&with_header(&"a".repeat(128))).is_ok());
    reject(&with_header(&"a".repeat(129)), ErrorCode::InvalidInput, "authHeader");
}

#[test]
fn nothing_accepted_here_can_fail_in_the_collector_or_the_http_client() {
    let mut probes: Vec<String> = (0x00_u8..0x80)
        .flat_map(|byte| {
            let c = char::from(byte);
            [format!("a{c}b"), c.to_string()]
        })
        .collect();
    probes.extend(MANAGED.iter().flat_map(|m| [m.to_lowercase(), m.to_uppercase()]));
    probes.extend(["Ключ", "X-Api-Key", "x-litellm-api-key", "api_key"].map(str::to_owned));
    let mut accepted = 0;
    for name in &probes {
        if validate(&with_header(name)).is_ok() {
            accepted += 1;
            assert!(consumers_accept(name), "{name:?} is saved but the collector refuses it");
            assert!(http_client_accepts(name), "{name:?} is saved but the HTTP client refuses it");
        }
    }
    assert!(accepted > 60, "the sweep must also cover accepted names: {accepted}");
}

#[test]
fn the_http_client_really_refuses_the_managed_names_and_takes_the_plain_ones() {
    // Control for the parity sweep above: the Debug-based probe tells the two outcomes apart.
    assert!(http_client_accepts("X-Api-Key"));
    for name in MANAGED {
        assert!(!http_client_accepts(name), "{name} is managed by the client");
    }
}
