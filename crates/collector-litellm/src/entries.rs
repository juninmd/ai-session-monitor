//! The lists a record carries besides its own cap: per-model caps and concurrent budget windows.
//! An entry missing either its spend or its limit says nothing usable and is skipped.

use serde_json::Value;

use crate::lenient::{epoch, non_negative, positive, text};
use crate::record::Obj;

/// Entries read from any server-supplied map or list: the body cap alone allows tens of thousands.
pub(crate) const MAX_ENTRIES: usize = 256;

/// A per-model cap with its current-window spend (`model_max_budget_usage`).
#[derive(Debug, PartialEq)]
pub(crate) struct ModelBudget {
    pub name: String,
    pub spend: f64,
    pub limit: f64,
    pub period: Option<String>,
}

/// One concurrent budget window of a key (`budget_limits` joined with `budget_limits_usage`).
#[derive(Debug, PartialEq)]
pub(crate) struct LimitBudget {
    pub duration: String,
    pub spend: f64,
    pub limit: f64,
    pub resets_at: Option<i64>,
}

pub(crate) fn model_budgets(obj: &Obj) -> Vec<ModelBudget> {
    let Some(usage) = obj.get("model_max_budget_usage").and_then(Value::as_object) else {
        return Vec::new();
    };
    usage
        .iter()
        .take(MAX_ENTRIES)
        .filter_map(|(name, entry)| {
            let entry = entry.as_object()?;
            Some(ModelBudget {
                name: name.clone(),
                spend: non_negative(entry.get("current_spend"))?,
                limit: positive(entry.get("budget_limit"))?,
                period: text(entry.get("time_period")),
            })
        })
        .collect()
}

pub(crate) fn limit_budgets(obj: &Obj) -> Vec<LimitBudget> {
    let Some(limits) = obj.get("budget_limits").and_then(Value::as_array) else {
        return Vec::new();
    };
    let usage = obj.get("budget_limits_usage").and_then(Value::as_object);
    limits
        .iter()
        .take(MAX_ENTRIES)
        .filter_map(|entry| {
            let entry = entry.as_object()?;
            let duration = entry.get("budget_duration")?.as_str()?;
            let used = usage?.get(duration)?.as_object()?;
            Some(LimitBudget {
                duration: duration.to_owned(),
                spend: non_negative(used.get("current_spend"))?,
                limit: positive(entry.get("max_budget"))?,
                resets_at: epoch(entry.get("reset_at")),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn obj(value: &Value) -> &Obj {
        value.as_object().expect("object")
    }

    #[test]
    fn model_budgets_need_both_usage_and_a_limit() {
        let body = json!({ "model_max_budget_usage": {
            "gpt-a": { "current_spend": 0.5, "budget_limit": 2.0, "time_period": "30d" },
            "no-limit": { "current_spend": 0.5 },
            "no-spend": { "budget_limit": 2.0 },
            "negative": { "current_spend": -1, "budget_limit": 2.0 },
            "zero-limit": { "current_spend": 1, "budget_limit": 0 },
            "not-an-object": 4,
        }});
        assert_eq!(
            model_budgets(obj(&body)),
            vec![ModelBudget {
                name: "gpt-a".into(),
                spend: 0.5,
                limit: 2.0,
                period: Some("30d".into())
            }]
        );
    }

    #[test]
    fn a_model_without_a_period_keeps_an_unknown_one() {
        let body = json!({ "model_max_budget_usage": {
            "m": { "current_spend": 0, "budget_limit": 1, "time_period": 30 },
        }});
        assert_eq!(model_budgets(obj(&body))[0].period, None);
    }

    #[test]
    fn budget_limits_need_a_limit_and_a_usage_entry_for_their_duration() {
        let body = json!({
            "budget_limits": [
                { "budget_duration": "1h", "max_budget": 5.0, "reset_at": "2026-10-11T13:00:00Z" },
                { "budget_duration": "30d", "max_budget": 100.0 },
                { "budget_duration": "7d", "max_budget": 20.0 },
                { "max_budget": 20.0 },
                "junk",
            ],
            "budget_limits_usage": {
                "1h": { "current_spend": 1.25 },
                "30d": { "current_spend": 40.0 },
                "7d": { "current_spend": "n/a" },
            },
        });
        let limits = limit_budgets(obj(&body));
        assert_eq!(limits.len(), 2);
        assert_eq!(limits[0].duration, "1h");
        assert_eq!((limits[0].spend, limits[0].limit), (1.25, 5.0));
        assert_eq!(limits[0].resets_at, Some(1_791_723_600));
        assert_eq!(limits[1].resets_at, None);
    }

    #[test]
    fn limits_without_a_usage_map_are_not_windows() {
        let body = json!({ "budget_limits": [{ "budget_duration": "1h", "max_budget": 5.0 }] });
        assert!(limit_budgets(obj(&body)).is_empty());
    }

    #[test]
    fn wrong_container_types_give_no_entries() {
        for body in [
            json!({ "model_max_budget_usage": [1, 2], "budget_limits": {"1h": {}} }),
            json!({ "model_max_budget_usage": "x", "budget_limits": "x" }),
            json!({ "model_max_budget_usage": null, "budget_limits": null }),
            json!({}),
        ] {
            assert!(model_budgets(obj(&body)).is_empty(), "{body}");
            assert!(limit_budgets(obj(&body)).is_empty(), "{body}");
        }
    }

    #[test]
    fn oversized_server_collections_are_bounded() {
        let usage: serde_json::Map<String, Value> = (0..10_000)
            .map(|i| (format!("m{i}"), json!({ "current_spend": 1, "budget_limit": 2 })))
            .collect();
        let limits: Vec<Value> = (0..10_000)
            .map(|i| json!({ "budget_duration": format!("{}m", i + 1), "max_budget": 2 }))
            .collect();
        let usage_by_duration: serde_json::Map<String, Value> =
            (0..10_000).map(|i| (format!("{}m", i + 1), json!({ "current_spend": 1 }))).collect();
        let body = json!({
            "model_max_budget_usage": usage,
            "budget_limits": limits,
            "budget_limits_usage": usage_by_duration,
        });
        assert_eq!(model_budgets(obj(&body)).len(), MAX_ENTRIES);
        assert_eq!(limit_budgets(obj(&body)).len(), MAX_ENTRIES);
    }
}
