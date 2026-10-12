//! JSON-RPC over the lines of a `LineSession`. The wire omits the `"jsonrpc"` member, as the
//! app-server documents. Anything that is not the answer to the request in flight is skipped,
//! except requests the server sends to us: those are refused, so it never waits on a reply.

use serde_json::{Map, Value, json};
use tokio::time::Instant;
use usage_model::{ErrorCode, HarnessError};
use usage_support::LineSession;

const METHOD_NOT_FOUND: i64 = -32601;
/// Enough of an error text to classify it; the rest is noise (some run to several kilobytes).
const MAX_ERROR_CHARS: usize = 2_000;

pub(crate) enum CallError {
    /// The server answered with a JSON-RPC error.
    Rpc { code: i64, message: String },
    /// The conversation itself broke: the clock, a closed pipe, a line that is not JSON-RPC.
    Transport(HarnessError),
}

pub(crate) struct Rpc<'a> {
    session: &'a mut LineSession,
    deadline: Instant,
    next_id: i64,
}

fn protocol_error() -> CallError {
    CallError::Transport(HarnessError::new(ErrorCode::ProtocolError, false))
}

impl<'a> Rpc<'a> {
    pub(crate) fn new(session: &'a mut LineSession, deadline: Instant) -> Self {
        Self { session, deadline, next_id: 1 }
    }

    /// A message that expects no answer.
    pub(crate) async fn notify(&mut self, method: &str, params: Value) -> Result<(), HarnessError> {
        self.send(&json!({ "method": method, "params": params })).await
    }

    /// Sends a request and waits for its answer: the `result`, or `CallError::Rpc`.
    pub(crate) async fn call(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, CallError> {
        let id = self.next_id;
        self.next_id += 1;
        let mut request = Map::new();
        request.insert("id".into(), json!(id));
        request.insert("method".into(), json!(method));
        if let Some(params) = params {
            request.insert("params".into(), params);
        }
        self.send(&Value::Object(request)).await.map_err(CallError::Transport)?;
        loop {
            let line = self.next_line().await.map_err(CallError::Transport)?;
            if line.trim().is_empty() {
                continue;
            }
            let Ok(Value::Object(mut message)) = serde_json::from_str::<Value>(&line) else {
                return Err(protocol_error());
            };
            let is_method = message.get("method").is_some_and(Value::is_string);
            let message_id = message.get("id").filter(|value| !value.is_null()).cloned();
            match (is_method, message_id) {
                (true, Some(server_id)) => {
                    self.refuse(server_id).await.map_err(CallError::Transport)?;
                }
                (false, Some(answered)) if answered == json!(id) => return settle(&mut message),
                // A notification, or an answer to something we are no longer waiting for.
                (true, None) | (false, Some(_)) => {}
                // An error with no id: the server could not even attribute our request.
                (false, None) if message.contains_key("error") => return Err(protocol_error()),
                (false, None) => {}
            }
        }
    }

    async fn send(&mut self, message: &Value) -> Result<(), HarnessError> {
        self.session.send_line(&message.to_string()).await
    }

    async fn refuse(&mut self, server_id: Value) -> Result<(), HarnessError> {
        let reply = json!({
            "id": server_id,
            "error": { "code": METHOD_NOT_FOUND, "message": "not supported by this client" }
        });
        self.send(&reply).await
    }

    async fn next_line(&mut self) -> Result<String, HarnessError> {
        let left = self.deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(HarnessError::new(ErrorCode::Timeout, true));
        }
        self.session.read_line(left).await
    }
}

/// The answer to our request: its `result`, its `error`, or neither (a protocol violation).
fn settle(message: &mut Map<String, Value>) -> Result<Value, CallError> {
    if let Some(error) = message.remove("error").filter(|error| !error.is_null()) {
        let code = error.get("code").and_then(Value::as_i64).unwrap_or(0);
        let text = error.get("message").and_then(Value::as_str).unwrap_or_default();
        let message = text.chars().take(MAX_ERROR_CHARS).collect();
        return Err(CallError::Rpc { code, message });
    }
    message.remove("result").ok_or_else(protocol_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settled(text: &str) -> Result<Value, CallError> {
        let Ok(Value::Object(mut message)) = serde_json::from_str::<Value>(text) else {
            panic!("fixture is not an object")
        };
        settle(&mut message)
    }

    #[test]
    fn a_result_is_returned_whatever_it_holds() {
        assert!(matches!(settled(r#"{"id":1,"result":{"a":1}}"#), Ok(v) if v == json!({"a": 1})));
        assert!(matches!(settled(r#"{"id":1,"result":null}"#), Ok(Value::Null)));
    }

    #[test]
    fn an_error_wins_and_keeps_code_and_a_bounded_message() {
        let long = "x".repeat(10_000);
        let text = format!(r#"{{"error":{{"code":-32600,"message":"{long}"}},"id":1}}"#);
        let Err(CallError::Rpc { code, message }) = settled(&text) else { panic!("expected rpc") };
        assert_eq!(code, -32600);
        assert_eq!(message.len(), MAX_ERROR_CHARS);
        assert!(matches!(
            settled(r#"{"id":1,"error":{"message":"m"},"result":1}"#),
            Err(CallError::Rpc { code: 0, .. })
        ));
    }

    #[test]
    fn an_answer_with_neither_result_nor_error_is_a_protocol_violation() {
        for text in [r#"{"id":1}"#, r#"{"id":1,"error":null}"#] {
            assert!(matches!(
                settled(text),
                Err(CallError::Transport(HarnessError { code: ErrorCode::ProtocolError, .. }))
            ));
        }
    }
}
