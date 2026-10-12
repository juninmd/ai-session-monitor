//! Lenient reading of one LiteLLM key, user or team record. Every field is optional and judged on
//! its own (see `lenient`), and only budget-relevant fields are ever kept.

use serde_json::{Map, Value};

use crate::entries::{LimitBudget, ModelBudget, limit_budgets, model_budgets};
use crate::lenient::{epoch, non_negative, positive, text};

pub(crate) type Obj = Map<String, Value>;

/// Which LiteLLM record a response describes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Key,
    User,
    Team,
}

impl Kind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Key => "key",
            Self::User => "user",
            Self::Team => "team",
        }
    }

    const fn alias_field(self) -> &'static str {
        match self {
            Self::Key => "key_alias",
            Self::User => "user_alias",
            Self::Team => "team_alias",
        }
    }
}

/// Static caps: not usage, only what the proxy would enforce.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Caps {
    pub rpm: Option<f64>,
    pub tpm: Option<f64>,
    pub parallel: Option<f64>,
    /// Epoch seconds.
    pub expires: Option<i64>,
}

/// The budget-relevant fields of one key, user or team record. Nothing else is ever kept, so
/// token hashes, key names and metadata cannot reach a view.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Level {
    pub alias: Option<String>,
    pub spend: Option<f64>,
    /// Only a positive, finite cap: `null`, `0` and nonsense all mean "no cap configured".
    pub max_budget: Option<f64>,
    pub duration: Option<String>,
    pub resets_at: Option<i64>,
    pub models: Vec<ModelBudget>,
    pub limits: Vec<LimitBudget>,
    pub caps: Caps,
    /// Links from a key to the records that may hold the real cap.
    pub user_id: Option<String>,
    pub team_id: Option<String>,
}

impl Level {
    pub(crate) fn parse(kind: Kind, obj: &Obj) -> Self {
        Self {
            alias: text(obj.get(kind.alias_field())),
            spend: non_negative(obj.get("spend")),
            max_budget: positive(obj.get("max_budget")),
            duration: text(obj.get("budget_duration")),
            resets_at: epoch(obj.get("budget_reset_at")),
            models: model_budgets(obj),
            limits: limit_budgets(obj),
            caps: Caps {
                rpm: positive(obj.get("rpm_limit")),
                tpm: positive(obj.get("tpm_limit")),
                parallel: positive(obj.get("max_parallel_requests")),
                expires: epoch(obj.get("expires")),
            },
            user_id: text(obj.get("user_id")),
            team_id: text(obj.get("team_id")),
        }
    }

    /// Anything that becomes a window; a level without it has no budget to show.
    pub(crate) fn has_budget_data(&self) -> bool {
        self.max_budget.is_some() || !self.models.is_empty() || !self.limits.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn parse(kind: Kind, value: &Value) -> Level {
        Level::parse(kind, value.as_object().expect("object"))
    }

    #[test]
    fn a_full_key_record_is_read_field_by_field() {
        let level = parse(
            Kind::Key,
            &json!({
                "key_alias": "ci-bot", "spend": 12.5, "max_budget": 50.0,
                "budget_duration": "30d", "budget_reset_at": "2026-11-01T00:00:00Z",
                "rpm_limit": 60, "tpm_limit": 200000, "max_parallel_requests": 4,
                "expires": "2027-01-01T00:00:00+00:00", "user_id": "u-1", "team_id": "t-1",
            }),
        );
        assert_eq!(level.alias.as_deref(), Some("ci-bot"));
        assert_eq!((level.spend, level.max_budget), (Some(12.5), Some(50.0)));
        assert_eq!(level.duration.as_deref(), Some("30d"));
        assert_eq!(level.resets_at, Some(1_793_491_200));
        assert_eq!(level.caps.rpm, Some(60.0));
        assert_eq!(level.caps.tpm, Some(200_000.0));
        assert_eq!(level.caps.parallel, Some(4.0));
        assert_eq!(level.caps.expires, Some(1_798_761_600));
        assert_eq!(
            (level.user_id.as_deref(), level.team_id.as_deref()),
            (Some("u-1"), Some("t-1"))
        );
    }

    #[test]
    fn the_alias_field_depends_on_the_record_kind() {
        let body = json!({ "key_alias": "k", "user_alias": "u", "team_alias": "t" });
        assert_eq!(parse(Kind::Key, &body).alias.as_deref(), Some("k"));
        assert_eq!(parse(Kind::User, &body).alias.as_deref(), Some("u"));
        assert_eq!(parse(Kind::Team, &body).alias.as_deref(), Some("t"));
    }

    #[test]
    fn nulls_and_missing_fields_are_unknown() {
        assert_eq!(parse(Kind::Key, &json!({})), Level::default());
        let nulls = json!({
            "key_alias": null, "spend": null, "max_budget": null, "budget_duration": null,
            "budget_reset_at": null, "rpm_limit": null, "tpm_limit": null, "expires": null,
            "model_max_budget_usage": null, "budget_limits": null, "user_id": null,
        });
        assert_eq!(parse(Kind::Key, &nulls), Level::default());
    }

    #[test]
    fn wrong_types_negative_and_out_of_range_numbers_are_unknown_not_clamped() {
        let level = parse(
            Kind::Key,
            &json!({
                "spend": "12.5", "max_budget": -5, "rpm_limit": true, "tpm_limit": [1],
                "max_parallel_requests": -1, "budget_reset_at": 1_793_491_200, "expires": "soon",
                "key_alias": 7, "user_id": {"id": "u"},
            }),
        );
        assert_eq!(level, Level::default());
        let huge =
            parse(Kind::Key, &json!({ "spend": 1e300, "max_budget": 1e16, "rpm_limit": 1e19 }));
        assert_eq!(huge, Level::default());
    }

    #[test]
    fn a_zero_cap_is_no_cap() {
        let level = parse(Kind::Key, &json!({ "max_budget": 0, "spend": 3, "rpm_limit": 0 }));
        assert_eq!((level.max_budget, level.caps.rpm), (None, None));
        assert_eq!(level.spend, Some(3.0));
        assert!(!level.has_budget_data());
    }

    #[test]
    fn budget_data_means_a_cap_a_model_window_or_a_limit_window() {
        assert!(!Level::default().has_budget_data());
        assert!(parse(Kind::Key, &json!({ "max_budget": 1 })).has_budget_data());
        let models =
            json!({ "model_max_budget_usage": { "m": { "current_spend": 1, "budget_limit": 2 } } });
        assert!(parse(Kind::Key, &models).has_budget_data());
        let limits = json!({
            "budget_limits": [{ "budget_duration": "1h", "max_budget": 5.0 }],
            "budget_limits_usage": { "1h": { "current_spend": 1.0 } },
        });
        assert!(parse(Kind::Team, &limits).has_budget_data());
    }

    #[test]
    fn limits_without_any_usage_map_do_not_count_as_budget_data() {
        let body = json!({ "budget_limits": [{ "budget_duration": "1h", "max_budget": 5.0 }] });
        assert!(!parse(Kind::Team, &body).has_budget_data());
    }
}
