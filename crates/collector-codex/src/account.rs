//! What `account/read` says about who is signed in. Only the account type and plan are read: the
//! e-mail address in the same answer is personal data and is never touched.

use serde_json::Value;

use crate::plan;

#[derive(Debug, PartialEq)]
pub(crate) enum Account {
    /// No account, and the provider needs one.
    SignedOut,
    /// An API key (or another login without ChatGPT plan windows): there are no limits to read.
    NoLimits,
    Chatgpt {
        plan: Option<String>,
    },
    /// The answer did not say; the rate-limit call decides.
    Unknown,
}

pub(crate) fn parse(result: &Value) -> Account {
    let Some(result) = result.as_object() else { return Account::Unknown };
    let Some(account) = result.get("account") else { return Account::Unknown };
    if account.is_null() {
        // `requiresOpenaiAuth: false` is a custom provider that needs no OpenAI login at all.
        let needs_login = result.get("requiresOpenaiAuth").and_then(Value::as_bool) != Some(false);
        return if needs_login { Account::SignedOut } else { Account::NoLimits };
    }
    match account.get("type").and_then(Value::as_str) {
        Some("chatgpt") => Account::Chatgpt {
            plan: account.get("planType").and_then(Value::as_str).and_then(plan::label),
        },
        Some("apiKey" | "amazonBedrock") => Account::NoLimits,
        _ => Account::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_missing_account_means_signed_out_unless_no_login_is_needed() {
        let out = json!({"account": null, "requiresOpenaiAuth": true, "workspaceRouting": null});
        assert_eq!(parse(&out), Account::SignedOut);
        assert_eq!(parse(&json!({"account": null})), Account::SignedOut);
        assert_eq!(
            parse(&json!({"account": null, "requiresOpenaiAuth": false})),
            Account::NoLimits
        );
    }

    #[test]
    fn api_key_and_bedrock_accounts_have_no_limits() {
        for kind in ["apiKey", "amazonBedrock"] {
            let answer = json!({"account": {"type": kind}, "requiresOpenaiAuth": true});
            assert_eq!(parse(&answer), Account::NoLimits, "{kind}");
        }
    }

    #[test]
    fn a_chatgpt_account_carries_only_its_plan() {
        let answer = json!({"account": {"type": "chatgpt", "email": "x@example.invalid", "planType": "plus"}});
        assert_eq!(parse(&answer), Account::Chatgpt { plan: Some("Plus".into()) });
        let bare = json!({"account": {"type": "chatgpt", "email": null, "planType": "unknown"}});
        assert_eq!(parse(&bare), Account::Chatgpt { plan: None });
    }

    #[test]
    fn anything_else_leaves_the_decision_to_the_rate_limit_call() {
        for answer in [
            json!(null),
            json!([]),
            json!("x"),
            json!({}),
            json!({"account": {}}),
            json!({"account": {"type": "somethingNew"}}),
            json!({"account": {"type": 3}}),
            json!({"account": "chatgpt"}),
        ] {
            assert_eq!(parse(&answer), Account::Unknown, "{answer}");
        }
    }
}
