//! `command.data.groups[].buckets[]` of the `/usage` JSON. The human `response` text is ignored.

use serde_json::Value;
use usage_model::HarnessError;

use crate::errors::endpoint_changed;
use crate::quota::{Read, merge, parse_summary};

/// The windows the CLI reported. A failed command, a missing `command.data` or no recognizable
/// bucket is an endpoint that changed, never a row of zeros.
pub(crate) fn parse_usage(body: &Value, now: i64) -> Result<Read, HarnessError> {
    let succeeded = body
        .get("status")
        .and_then(Value::as_str)
        .is_some_and(|status| status.eq_ignore_ascii_case("success"));
    let data = body.get("command").and_then(|command| command.get("data"));
    match data {
        Some(data) if succeeded => parse_summary(data, now).map(|read| merge(None, Some(read))),
        _ => Err(endpoint_changed()),
    }
}
