//! `GetUserStatus`: the plan and, on older builds, one quota per model.
//!
//! Models of one pool share an allowance, so the pool is reported once, by its most used model. The
//! response states no window length; it is guessed from how far away the reset is and marked as a
//! guess, because the window length steers the pace estimate and a wrong one would raise false alarms.

use serde_json::Value;
use usage_model::HarnessError;

use super::envelope::check_code;
use super::model::{Cadence, Pool, PoolWindow, Read};
use super::values::{clean, reset_at, used_pct};
use crate::errors::endpoint_changed;

const MAX_MODELS: usize = 64;
const MAX_TEXT: usize = 48;
/// A five hour window resets within five hours of being read; the quarter hour absorbs clock skew
/// and a late refresh.
const FIVE_HOUR_REACH: i64 = 5 * 3_600 + 900;
/// Weekly windows reset within a week, with an hour of the same slack.
const WEEKLY_REACH: i64 = 7 * 86_400 + 3_600;
const PLAN_FIELDS: [&str; 5] =
    ["planName", "planDisplayName", "displayName", "productName", "planShortName"];

/// The plan and the pool windows a status response states. `userStatus` with none of its known
/// parts is a server that has nothing to say, which is not a status.
pub(crate) fn parse_status(body: &Value, now: i64) -> Result<Read, HarnessError> {
    check_code(body)?;
    let status = body.get("userStatus").filter(|status| is_known(status));
    let status = status.ok_or_else(endpoint_changed)?;
    Ok(Read { plan: plan_of(status), windows: pool_windows(status, now) })
}

fn is_known(status: &Value) -> bool {
    ["planStatus", "userTier", "cascadeModelConfigData"]
        .into_iter()
        .any(|key| status.get(key).is_some_and(Value::is_object))
}

/// The subscription tier when stated (`userTier.name`, which tells Ultra from Pro), else the plan.
fn plan_of(status: &Value) -> Option<String> {
    let tier = status.get("userTier").and_then(|tier| tier.get("name"));
    let info = status.get("planStatus").and_then(|plan| plan.get("planInfo"));
    let named = PLAN_FIELDS.into_iter().filter_map(|key| info?.get(key));
    tier.into_iter().chain(named).filter_map(Value::as_str).find_map(|text| clean(text, MAX_TEXT))
}

struct Model {
    pool: Pool,
    label: Option<String>,
    used: Option<f64>,
    reset: Option<i64>,
}

fn model_of(config: &Value, now: i64) -> Option<Model> {
    let quota = config.get("quotaInfo").filter(|quota| quota.is_object())?;
    let label = config.get("label").and_then(Value::as_str);
    let id = config.get("modelOrAlias").and_then(|alias| alias.get("model")?.as_str());
    let pool = id.and_then(Pool::of_name).or_else(|| label.and_then(Pool::of_name))?;
    Some(Model {
        pool,
        label: label.and_then(|text| clean(text, MAX_TEXT)),
        used: used_pct(quota),
        reset: reset_at(quota, now),
    })
}

fn pool_windows(status: &Value, now: i64) -> Vec<PoolWindow> {
    let configs = status
        .get("cascadeModelConfigData")
        .and_then(|data| data.get("clientModelConfigs"))
        .and_then(Value::as_array);
    let models: Vec<Model> =
        configs.into_iter().flatten().take(MAX_MODELS).filter_map(|c| model_of(c, now)).collect();
    [Pool::Gemini, Pool::ClaudeGpt]
        .into_iter()
        .filter_map(|pool| pool_window(pool, &models, now))
        .collect()
}

/// One window for the pool, from its most used model; models that state no share only lend their
/// reset time. `None` when the pool has no model with a quota at all.
fn pool_window(pool: Pool, models: &[Model], now: i64) -> Option<PoolWindow> {
    let own: Vec<&Model> = models.iter().filter(|model| model.pool == pool).collect();
    let first = own.first()?;
    let lead = own
        .iter()
        .filter(|model| model.used.is_some())
        .max_by(|a, b| a.used.unwrap_or_default().total_cmp(&b.used.unwrap_or_default()))
        .unwrap_or(first);
    let earliest = own.iter().filter_map(|model| model.reset).min();
    let mut window = PoolWindow::new(pool);
    window.used_pct = lead.used;
    window.resets_at = lead.reset.or(earliest);
    match infer(window.resets_at, now) {
        Some(cadence) => {
            window.cadence = Some(cadence);
            window.inferred = true;
        }
        None => {
            window.label = lead.label.clone();
            window.slug = Some("quota".to_owned());
        }
    }
    Some(window)
}

/// A reset already in the past still belongs to the window that just ended.
fn infer(reset: Option<i64>, now: i64) -> Option<Cadence> {
    let ahead = reset?.saturating_sub(now);
    if ahead <= FIVE_HOUR_REACH {
        Some(Cadence::FiveHour)
    } else if ahead <= WEEKLY_REACH {
        Some(Cadence::Weekly)
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
