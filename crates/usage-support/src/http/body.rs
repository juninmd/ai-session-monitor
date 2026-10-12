//! Reads a response body under a size cap and decodes it as JSON.

use reqwest::Response;
use serde_json::Value;
use usage_model::{ErrorCode, HarnessError};

use super::classify::{failure, from_transport};

const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";
/// Largest buffer reserved up front; the declared length is only a hint from the server.
const PREALLOCATE_LIMIT: u64 = 64 * 1024;

/// Rejects a declared length over the cap, then enforces the cap while streaming: the read stops
/// and the connection is dropped at the first chunk that would cross it.
pub(super) async fn read_json(
    response: &mut Response,
    max_bytes: usize,
    host: &str,
) -> Result<Value, HarnessError> {
    let declared = response.content_length();
    if declared.is_some_and(|len| len > u64::try_from(max_bytes).unwrap_or(u64::MAX)) {
        return Err(failure(ErrorCode::ResponseTooLarge, false, host));
    }
    let hint = declared.map_or(0, |len| len.min(PREALLOCATE_LIMIT));
    let mut body = Vec::with_capacity(usize::try_from(hint).unwrap_or(0));
    while let Some(chunk) = response.chunk().await.map_err(|e| from_transport(&e, host))? {
        if chunk.len() > max_bytes - body.len() {
            return Err(failure(ErrorCode::ResponseTooLarge, false, host));
        }
        body.extend_from_slice(&chunk);
    }
    let json = body.strip_prefix(UTF8_BOM).unwrap_or(&body);
    serde_json::from_slice(json).map_err(|_| failure(ErrorCode::BadResponse, false, host))
}
