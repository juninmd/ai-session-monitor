//! Which records a poll reads. The key scope reads the caller's own key; when the key shows no
//! budget at all it may ask the user it belongs to, then its team, because the cap often lives
//! there. Those follow-ups reuse the same client, credential and host and never exceed two.

use serde_json::Value;
use usage_model::{ErrorCode, HarnessError};

use crate::plan::{Plan, valid_id};
use crate::record::{Kind, Level, Obj};
use crate::transport::Transport;

/// What one poll learned.
pub(crate) struct Resolved {
    /// The record the user configured: it supplies the static caps.
    pub primary: Level,
    /// A user or team record that supplied the budget instead of the key.
    pub followed: Option<(Kind, Level)>,
    /// Follow-ups that failed: the budget may be there, so "no budget" cannot be claimed.
    pub unreadable: Vec<(Kind, HarnessError)>,
}

pub(crate) async fn resolve(
    transport: &Transport<'_>,
    plan: &Plan<'_>,
) -> Result<Resolved, HarnessError> {
    let primary = fetch(transport, plan.scope, plan.entity.as_deref()).await?;
    let mut resolved = Resolved { primary, followed: None, unreadable: Vec::new() };
    if plan.scope != Kind::Key || resolved.primary.has_budget_data() {
        return Ok(resolved);
    }
    let links = [
        (Kind::User, resolved.primary.user_id.clone()),
        (Kind::Team, resolved.primary.team_id.clone()),
    ];
    let key = plan.secret.expose().trim();
    for (kind, id) in links {
        // An id that holds the key would put it in a URL, where proxies log it.
        let Some(id) = id.filter(|id| valid_id(id) && !id.contains(key)) else { continue };
        match fetch(transport, kind, Some(id.as_str())).await {
            Ok(level) if level.has_budget_data() => {
                resolved.followed = Some((kind, level));
                break;
            }
            Ok(_without_budget) => {}
            Err(error) => resolved.unreadable.push((kind, error)),
        }
    }
    Ok(resolved)
}

async fn fetch(
    transport: &Transport<'_>,
    kind: Kind,
    id: Option<&str>,
) -> Result<Level, HarnessError> {
    let body = match (kind, id) {
        (Kind::Key, _) => transport.get(&["key", "info"], &[]).await?,
        (Kind::User, Some(id)) => transport.get(&["user", "info"], &[("user_id", id)]).await?,
        // The default team listing carries every key of the team, which can pass the body cap.
        (Kind::Team, Some(id)) => {
            transport.get(&["team", "info"], &[("team_id", id), ("key_limit", "1")]).await?
        }
        (Kind::User | Kind::Team, None) => {
            return Err(
                HarnessError::new(ErrorCode::InvalidInput, false).with_param("field", "entityId")
            );
        }
    };
    read(kind, id, &body, transport.host())
}

fn container_key(kind: Kind) -> &'static str {
    match kind {
        Kind::Key => "info",
        Kind::User => "user_info",
        Kind::Team => "team_info",
    }
}

fn id_key(kind: Kind) -> Option<&'static str> {
    match kind {
        Kind::Key => None,
        Kind::User => Some("user_id"),
        Kind::Team => Some("team_id"),
    }
}

/// The shape check: a JSON object holding the record under the documented key, and for user and
/// team records an echo of the id that was asked for. Anything else is a changed endpoint.
fn read(kind: Kind, id: Option<&str>, body: &Value, host: &str) -> Result<Level, HarnessError> {
    let changed = |reason: &str| {
        HarnessError::new(ErrorCode::EndpointChanged, false)
            .with_param("host", host)
            .with_param("reason", reason)
    };
    let root = body.as_object().ok_or_else(|| changed("not_an_object"))?;
    let record: &Obj = root
        .get(container_key(kind))
        .and_then(Value::as_object)
        .ok_or_else(|| changed("no_record"))?;
    if let (Some(field), Some(asked)) = (id_key(kind), id) {
        let echoed: Vec<&str> = [root.get(field), record.get(field)]
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        if echoed.is_empty() {
            return Err(changed("id_missing"));
        }
        if echoed.iter().any(|seen| *seen != asked) {
            return Err(changed("id_mismatch"));
        }
    }
    Ok(Level::parse(kind, record))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn reason(error: &HarnessError) -> Option<&str> {
        assert_eq!(error.code, ErrorCode::EndpointChanged);
        assert!(!error.retryable);
        error.params.get("reason").map(String::as_str)
    }

    #[test]
    fn the_record_is_found_under_the_documented_key() {
        let key = json!({ "key": "ignored", "info": { "spend": 1.5, "max_budget": 3.0 } });
        let level = read(Kind::Key, None, &key, "h").expect("key");
        assert_eq!((level.spend, level.max_budget), (Some(1.5), Some(3.0)));
        let user = json!({ "user_id": "u-1", "user_info": { "user_id": "u-1", "spend": 2.0 } });
        assert_eq!(read(Kind::User, Some("u-1"), &user, "h").expect("user").spend, Some(2.0));
        let team = json!({ "team_id": "t-1", "team_info": { "team_id": "t-1", "spend": 4.0 } });
        assert_eq!(read(Kind::Team, Some("t-1"), &team, "h").expect("team").spend, Some(4.0));
    }

    #[test]
    fn a_body_without_the_record_is_a_changed_endpoint() {
        for body in [
            json!([]),
            json!("text"),
            json!(7),
            json!(null),
            json!({}),
            json!({ "info": [] }),
            json!({ "info": null }),
            json!({ "info": 5 }),
            json!({ "user_info": {} }),
        ] {
            let error = read(Kind::Key, None, &body, "h").expect_err("no key record");
            assert!(matches!(reason(&error), Some("not_an_object" | "no_record")), "{body}");
            assert_eq!(error.params.get("host").map(String::as_str), Some("h"));
        }
    }

    #[test]
    fn an_empty_record_is_still_a_record() {
        let level = read(Kind::Key, None, &json!({ "info": {} }), "h").expect("empty record");
        assert!(!level.has_budget_data());
    }

    #[test]
    fn user_and_team_records_must_echo_the_requested_id() {
        let wrong =
            json!({ "user_id": "someone-else", "user_info": { "user_id": "someone-else" } });
        assert_eq!(
            reason(&read(Kind::User, Some("u-1"), &wrong, "h").expect_err("mismatch")),
            Some("id_mismatch")
        );
        let half = json!({ "user_id": "u-1", "user_info": { "user_id": "someone-else" } });
        assert_eq!(
            reason(&read(Kind::User, Some("u-1"), &half, "h").expect_err("half")),
            Some("id_mismatch")
        );
        let outer = json!({ "user_id": "someone-else", "user_info": { "user_id": "u-1" } });
        assert_eq!(
            reason(&read(Kind::User, Some("u-1"), &outer, "h").expect_err("outer")),
            Some("id_mismatch")
        );
        let silent = json!({ "user_info": { "spend": 1.0 } });
        assert_eq!(
            reason(&read(Kind::User, Some("u-1"), &silent, "h").expect_err("silent")),
            Some("id_missing")
        );
        let numeric = json!({ "team_id": 7, "team_info": { "team_id": null } });
        assert_eq!(
            reason(&read(Kind::Team, Some("7"), &numeric, "h").expect_err("numeric")),
            Some("id_missing")
        );
    }

    #[test]
    fn one_echoed_id_is_enough_when_the_other_is_null() {
        let body = json!({ "user_id": null, "user_info": { "user_id": "u-1", "spend": 1.0 } });
        assert!(read(Kind::User, Some("u-1"), &body, "h").is_ok());
    }

    #[test]
    fn ids_compare_exactly_not_case_insensitively() {
        let body = json!({ "team_id": "TEAM-A", "team_info": { "team_id": "TEAM-A" } });
        assert!(read(Kind::Team, Some("team-a"), &body, "h").is_err());
    }
}
