//! One read of `codex app-server`: start it, handshake, check the account, read the limits, and
//! always end the whole process tree. Nothing here keeps a process, a file or a socket open
//! after it returns.
//!
//! Confidence: high for the handshake, `account/read` and the signed-out error (documented at
//! developers.openai.com/codex/app-server and observed against an empty throwaway profile); medium
//! for the signed-in `account/rateLimits/read` answer, known from the generated JSON Schema of
//! codex-cli 0.162.1 and the openai/codex sources but never observed with a real account.

use std::path::Path;
use std::time::Duration;

use serde_json::{Value, json};
use tokio::time::{Instant, timeout_at};
use usage_model::HarnessError;
use usage_model::settings::CodexSettings;
use usage_support::{CommandSpec, LineSession};

use crate::account::{self, Account};
use crate::failure::{self, Phase, Verdict};
use crate::limits::{self, Limits};
use crate::rpc::{CallError, Rpc};

/// The whole conversation, from looking up the binary to the last answer, must fit in this. Ending
/// the process tree comes after and takes a moment more (a few seconds at the very most).
pub const APP_SERVER_BUDGET: Duration = Duration::from_secs(15);
/// A rate-limit answer is a few kilobytes; anything near this is not one.
const MAX_LINE: usize = 256 * 1024;
const MAX_BUDGET: Duration = Duration::from_secs(600);

pub(crate) enum Reading {
    Limits(Limits),
    /// Signed in with an API key (or no OpenAI login): there are no plan windows to show.
    NoLimits,
}

pub(crate) async fn read(
    settings: &CodexSettings,
    budget: Duration,
) -> Result<Reading, HarnessError> {
    let deadline = Instant::now() + budget.min(MAX_BUDGET);
    let program = crate::binary::resolve(settings).await?;
    let mut spec = CommandSpec::new(&program);
    spec.args = vec!["app-server".into()];
    // A shim runs the commands it names from here first, so start from the folder that already
    // holds the program we trust, never from whatever directory this app was launched in.
    spec.cwd = program.parent().map(Path::to_path_buf);
    let mut session = LineSession::spawn(spec)?.with_max_line(MAX_LINE);
    let outcome = timeout_at(deadline, converse(&mut session, deadline)).await;
    end(session).await;
    outcome.unwrap_or_else(|_elapsed| Err(failure::timeout()))
}

/// Ends the child and everything it started, whatever happened above. Dropping the session kills
/// the tree while the pipes are still open; `LineSession::kill` closes stdin first, and a server
/// that quits at end-of-input would then be gone before the kill, orphaning its children. Off the
/// async threads because the tree kill waits for `taskkill`, and cancellation-proof because the
/// closure runs to the end even when this future is dropped.
async fn end(session: LineSession) {
    let _ = tokio::task::spawn_blocking(move || drop(session)).await;
}

async fn converse(session: &mut LineSession, deadline: Instant) -> Result<Reading, HarnessError> {
    let mut rpc = Rpc::new(session, deadline);
    handshake(&mut rpc).await?;

    let mut plan = None;
    match rpc.call("account/read", Some(json!({ "refreshToken": false }))).await {
        Ok(answer) => match account::parse(&answer) {
            Account::SignedOut => return Err(failure::not_signed_in()),
            Account::NoLimits => return Ok(Reading::NoLimits),
            Account::Chatgpt { plan: name } => plan = name,
            Account::Unknown => {}
        },
        // An older server may not know the method; the rate-limit call below still decides.
        Err(CallError::Rpc { .. }) => {}
        Err(CallError::Transport(error)) => {
            return Err(failure::from_transport(&error, Phase::Account));
        }
    }

    match rpc.call("account/rateLimits/read", None).await {
        Ok(answer) => parse_limits(&answer, plan),
        Err(CallError::Rpc { code, message }) => {
            match failure::from_rate_limits_error(code, &message) {
                Verdict::NoLimits => Ok(Reading::NoLimits),
                Verdict::Failed(error) => Err(error),
            }
        }
        Err(CallError::Transport(error)) => Err(failure::from_transport(&error, Phase::RateLimits)),
    }
}

async fn handshake(rpc: &mut Rpc<'_>) -> Result<(), HarnessError> {
    let client = json!({
        "clientInfo": {
            "name": "ai-session-monitor",
            "title": "AI Session Monitor",
            "version": env!("CARGO_PKG_VERSION"),
        }
    });
    match rpc.call("initialize", Some(client)).await {
        Ok(Value::Object(_)) => {}
        Ok(_) | Err(CallError::Rpc { .. }) => return Err(failure::handshake_refused()),
        Err(CallError::Transport(error)) => {
            return Err(failure::from_transport(&error, Phase::Initialize));
        }
    }
    rpc.notify("initialized", json!({}))
        .await
        .map_err(|error| failure::from_transport(&error, Phase::Initialize))
}

/// The main bucket of `account/rateLimits/read`, or `EndpointChanged` when the answer has none.
fn parse_limits(answer: &Value, account_plan: Option<String>) -> Result<Reading, HarnessError> {
    let bucket = answer
        .get("rateLimitsByLimitId")
        .and_then(|buckets| buckets.get("codex"))
        .filter(|bucket| bucket.is_object())
        .or_else(|| answer.get("rateLimits"));
    let Some(mut limits) = bucket.and_then(limits::parse_snapshot) else {
        return Err(failure::endpoint_changed(Phase::RateLimits));
    };
    if limits.plan.is_none() {
        limits.plan = account_plan;
    }
    limits.reset_credits = limits::reset_credit_count(answer);
    Ok(Reading::Limits(limits))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits_of(answer: &Value) -> Limits {
        match parse_limits(answer, None) {
            Ok(Reading::Limits(limits)) => limits,
            _ => panic!("expected limits for {answer}"),
        }
    }

    #[test]
    fn the_codex_bucket_wins_over_the_single_bucket_view() {
        let answer = json!({
            "rateLimits": {"limitId": "codex_other", "primary": {"usedPercent": 99, "windowDurationMins": 300}},
            "rateLimitsByLimitId": {
                "codex_other": {"primary": {"usedPercent": 99, "windowDurationMins": 300}},
                "codex": {"primary": {"usedPercent": 11, "windowDurationMins": 300}}
            }
        });
        assert_eq!(limits_of(&answer).windows[0].used_pct, Some(11.0));
    }

    #[test]
    fn the_single_bucket_view_is_used_when_there_is_no_codex_bucket() {
        let answer = json!({
            "rateLimits": {"primary": {"usedPercent": 7, "windowDurationMins": 300}},
            "rateLimitsByLimitId": null
        });
        assert_eq!(limits_of(&answer).windows[0].used_pct, Some(7.0));
    }

    #[test]
    fn the_account_plan_fills_in_when_the_snapshot_has_none() {
        let answer = json!({"rateLimits": {"primary": {"usedPercent": 7}}});
        let Ok(Reading::Limits(limits)) = parse_limits(&answer, Some("Pro".into())) else {
            panic!("expected limits")
        };
        assert_eq!(limits.plan.as_deref(), Some("Pro"));
        let own = json!({"rateLimits": {"planType": "plus"}});
        let Ok(Reading::Limits(limits)) = parse_limits(&own, Some("Pro".into())) else {
            panic!("expected limits")
        };
        assert_eq!(limits.plan.as_deref(), Some("Plus"));
    }

    #[test]
    fn an_answer_without_a_snapshot_is_the_endpoint_changing() {
        for answer in [
            json!(null),
            json!({}),
            json!({"rateLimits": null}),
            json!({"rateLimits": "x"}),
            json!({"limits": {"primary": {}}}),
            json!([1]),
        ] {
            let Err(error) = parse_limits(&answer, None) else { panic!("{answer} was accepted") };
            assert_eq!(error.code, usage_model::ErrorCode::EndpointChanged, "{answer}");
            assert!(!error.retryable);
        }
    }

    #[test]
    fn the_earned_reset_count_comes_from_the_answer_root() {
        let answer = json!({
            "rateLimits": {"primary": {"usedPercent": 7}},
            "rateLimitResetCredits": {"availableCount": 2, "credits": null}
        });
        assert_eq!(limits_of(&answer).reset_credits, Some(2));
        let none =
            json!({"rateLimits": {"primary": {"usedPercent": 7}}, "rateLimitResetCredits": null});
        assert_eq!(limits_of(&none).reset_credits, None);
    }
}
