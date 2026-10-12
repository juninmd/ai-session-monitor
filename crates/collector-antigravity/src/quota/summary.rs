//! `RetrieveUserQuotaSummary`: groups of models, each with a five hour and a weekly bucket.
//!
//! Every field is optional. Group and bucket names decide the pool and the cadence; a bucket that
//! says nothing about either is still listed under its own name instead of being guessed.

use serde_json::Value;
use usage_model::HarnessError;

use super::envelope::check_code;
use super::model::{Pool, PoolWindow, Read};
use super::values::{cadence_of, clean, reset_at, slug, used_pct};
use crate::errors::endpoint_changed;

const MAX_GROUPS: usize = 16;
const MAX_BUCKETS: usize = 16;
const MAX_NAME: usize = 48;
const MAX_SLUG: usize = 32;

/// The windows the summary lists. A body with no recognizable group or bucket is an endpoint that
/// no longer answers the way it used to.
pub(crate) fn parse_summary(body: &Value, now: i64) -> Result<Read, HarnessError> {
    check_code(body)?;
    let groups = groups_of(body).ok_or_else(endpoint_changed)?;
    let windows: Vec<PoolWindow> = groups
        .iter()
        .take(MAX_GROUPS)
        .enumerate()
        .flat_map(|(index, group)| group_windows(group, index, now))
        .collect();
    if windows.is_empty() {
        return Err(endpoint_changed());
    }
    Ok(Read { plan: None, windows })
}

/// Under `response`, under `summary`, or at the root, in that order.
fn groups_of(body: &Value) -> Option<&Vec<Value>> {
    let wrapped = ["response", "summary"].into_iter().filter_map(|key| body.get(key));
    wrapped.chain(std::iter::once(body)).find_map(|root| root.get("groups")?.as_array())
}

fn name_of(object: &Value, keys: [&str; 2]) -> Option<String> {
    keys.into_iter()
        .filter_map(|key| object.get(key)?.as_str())
        .find_map(|text| clean(text, MAX_NAME))
}

fn group_windows(group: &Value, index: usize, now: i64) -> Vec<PoolWindow> {
    let Some(buckets) = group.get("buckets").and_then(Value::as_array) else {
        return Vec::new();
    };
    let pool = pool_of(group, index);
    buckets
        .iter()
        .take(MAX_BUCKETS)
        .enumerate()
        .filter_map(|(position, bucket)| bucket_window(bucket, &pool, position, now))
        .collect()
}

/// By the group name, then by the models its description lists; otherwise a pool of its own.
fn pool_of(group: &Value, index: usize) -> Pool {
    let name = name_of(group, ["displayName", "name"]);
    let listed = group.get("description").and_then(Value::as_str);
    name.as_deref()
        .and_then(Pool::of_name)
        .or_else(|| listed.and_then(Pool::of_name))
        .unwrap_or_else(|| {
            let id = name.as_deref().and_then(|text| slug(text, MAX_SLUG));
            Pool::Other(id.unwrap_or_else(|| format!("group_{index}")))
        })
}

fn bucket_window(bucket: &Value, pool: &Pool, position: usize, now: i64) -> Option<PoolWindow> {
    if bucket.get("disabled").and_then(Value::as_bool) == Some(true) {
        return None;
    }
    let id = name_of(bucket, ["bucketId", "id"]);
    let name = name_of(bucket, ["displayName", "name"]);
    let stated = bucket.get("window").and_then(Value::as_str);
    let mut window = PoolWindow::new(pool.clone());
    window.cadence = cadence_of(&[stated, id.as_deref(), name.as_deref()]);
    window.used_pct = used_pct(bucket);
    window.resets_at = reset_at(bucket, now);
    // A bucket without a cadence, a share and a reset time has nothing to show.
    if window.cadence.is_none() && window.used_pct.is_none() && window.resets_at.is_none() {
        return None;
    }
    if window.cadence.is_none() {
        let own = id.as_deref().or(name.as_deref()).and_then(|text| slug(text, MAX_SLUG));
        window.slug = Some(own.unwrap_or_else(|| format!("bucket_{position}")));
        window.label = name.or(id);
    }
    Some(window)
}

#[cfg(test)]
mod tests;
