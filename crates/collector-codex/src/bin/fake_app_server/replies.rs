//! The canned answers. Every value is invented. Strings containing `sentinel` exist so a test can
//! prove that nothing the server says is copied into a view.

use serde_json::{Value, json};

use crate::mode::Mode;

pub enum Reply {
    Result(Value),
    Error { code: i64, message: String },
}

fn error(code: i64, message: &str) -> Reply {
    Reply::Error { code, message: message.to_owned() }
}

pub fn initialize() -> Value {
    json!({
        "userAgent": "fake-app-server/0.0.0 sentinel-user-agent",
        "codexHome": "C:\\Users\\sentinel-codex-home\\.codex",
        "platformFamily": "windows",
        "platformOs": "windows"
    })
}

pub fn account(mode: Mode) -> Reply {
    let signed_in = |plan: &str| {
        Reply::Result(json!({
            "account": {"type": "chatgpt", "email": "sentinel-person@example.invalid", "planType": plan},
            "requiresOpenaiAuth": true
        }))
    };
    match mode {
        Mode::ApiKey => {
            Reply::Result(json!({"account": {"type": "apiKey"}, "requiresOpenaiAuth": true}))
        }
        Mode::NoAuthProvider => {
            Reply::Result(json!({"account": null, "requiresOpenaiAuth": false}))
        }
        Mode::NotSignedIn => Reply::Result(json!({"account": null, "requiresOpenaiAuth": true})),
        // A server too old to know the method: the limits request has to decide.
        Mode::ApiKeyRpc | Mode::NotSignedInRpc => error(-32601, "Method not found"),
        Mode::ProNoPrimary => signed_in("pro"),
        _ => signed_in("plus"),
    }
}

pub fn rate_limits(mode: Mode) -> Reply {
    match mode {
        Mode::PlusOnly => snapshot(Some(five_hour(42)), None, "plus", None),
        Mode::ProNoPrimary => snapshot(None, Some(weekly(17)), "pro", None),
        Mode::BadNumbers => bad_numbers(),
        Mode::BadShape => Reply::Result(json!({"rateLimits": "not an object"})),
        Mode::ApiKey | Mode::ApiKeyRpc => {
            error(-32600, "chatgpt authentication required to read rate limits")
        }
        Mode::NotSignedIn | Mode::NotSignedInRpc | Mode::NoAuthProvider => {
            error(-32600, "codex account authentication required to read rate limits")
        }
        Mode::Backend(status) => error(-32603, &backend_text(status)),
        Mode::Offline => error(
            -32603,
            "failed to fetch codex rate limits: error sending request for url (https://chatgpt.invalid/backend-api/wham/usage)",
        ),
        _ => snapshot(Some(five_hour(42)), Some(weekly(17)), "plus", Some(json!("12.50"))),
    }
}

fn window(used: Value, minutes: Value, resets_at: Value) -> Value {
    json!({"usedPercent": used, "windowDurationMins": minutes, "resetsAt": resets_at})
}

// NOW + 8100 s and NOW + 349200 s for the tests' fixed clock of 2026-10-10T20:00:00Z.
fn five_hour(used: u32) -> Value {
    window(json!(used), json!(300), json!(1_791_670_500))
}

fn weekly(used: u32) -> Value {
    window(json!(used), json!(10_080), json!(1_792_011_600))
}

/// Values no real server sends: negative, absurd, and a millisecond timestamp.
fn bad_numbers() -> Reply {
    let negative = window(json!(-5), json!(300), json!(1_791_670_500));
    let absurd = window(json!(100_000), json!(10_080), json!(99_999_999_999_999_i64));
    snapshot(Some(negative), Some(absurd), "plus", Some(json!("NaN")))
}

fn snapshot(
    primary: Option<Value>,
    secondary: Option<Value>,
    plan: &str,
    balance: Option<Value>,
) -> Reply {
    let credits =
        balance.map(|balance| json!({"hasCredits": true, "unlimited": false, "balance": balance}));
    let bucket = json!({
        "limitId": "codex",
        "limitName": null,
        "primary": primary,
        "secondary": secondary,
        "credits": credits,
        "planType": plan,
        "rateLimitReachedType": null
    });
    Reply::Result(json!({"rateLimits": bucket.clone(), "rateLimitsByLimitId": {"codex": bucket}}))
}

/// The wording the real backend client uses for a bad status, with a body to leak.
fn backend_text(status: u16) -> String {
    format!(
        "failed to fetch codex rate limits: GET https://chatgpt.invalid/backend-api/wham/usage failed: {status} Sentinel Reason; content-type=application/json; body={{\"detail\":\"sentinel-backend-body\"}}"
    )
}
