//! Bounded GETs against the proxy. The client comes from the plan, every call is capped by the
//! time left in the poll, and the credential only travels in the plan's auth header.

use std::io;
use std::path::Path;
use std::time::Duration;

use serde_json::Value;
use tokio::time::Instant;
use usage_model::{ErrorCode, HarnessError};
use usage_support::{Auth, ClientOptions, RequestSpec, SafeClient, TlsPolicy, read_capped};

use crate::plan::{Plan, Tls};

const MAX_BODY_BYTES: usize = 1 << 20;
const MAX_PEM_BYTES: usize = 1 << 20;
/// The scheduler cancels a poll after 30 s; finishing earlier keeps a specific result.
const POLL_BUDGET: Duration = Duration::from_secs(25);
/// Starting a request with less than this left only turns it into a timeout.
const MIN_REQUEST_TIME: Duration = Duration::from_secs(1);
/// A poll makes at most one request per level of the key, user, team walk.
const MAX_REQUESTS: u32 = 3;

pub(crate) struct Transport<'a> {
    client: SafeClient,
    plan: &'a Plan<'a>,
    deadline: Instant,
}

/// Time the whole poll may take: each request gets the configured timeout, all of them together
/// never outlast the scheduler.
fn budget(per_request: Duration) -> Duration {
    per_request.saturating_mul(MAX_REQUESTS).min(POLL_BUDGET)
}

impl<'a> Transport<'a> {
    /// Reads the CA bundle when one is configured and builds the client. The first file or network
    /// access of a poll happens here, after every gate and validation has passed.
    pub(crate) async fn open(plan: &'a Plan<'a>) -> Result<Self, HarnessError> {
        let deadline = Instant::now() + budget(plan.timeout);
        let tls = match tokio::time::timeout_at(deadline, policy(&plan.tls)).await {
            Ok(policy) => policy?,
            Err(_elapsed) => return Err(timed_out(plan)),
        };
        let client = SafeClient::new(ClientOptions {
            timeout: plan.timeout,
            max_body_bytes: MAX_BODY_BYTES,
            tls,
            allow_loopback_http: plan.allow_loopback_http,
            ..ClientOptions::default()
        })
        .map_err(|error| unusable_bundle(error, &plan.tls))?;
        Ok(Self { client, plan, deadline })
    }

    pub(crate) fn host(&self) -> &str {
        &self.plan.host
    }

    /// `GET {base}/{segments...}?{query}` with the key attached. Path prefix and percent-encoding
    /// are the client's job; a refused request never reaches the network.
    pub(crate) async fn get(
        &self,
        segments: &[&str],
        query: &[(&str, &str)],
    ) -> Result<Value, HarnessError> {
        let left = self.deadline.saturating_duration_since(Instant::now());
        if left < MIN_REQUEST_TIME {
            return Err(timed_out(self.plan));
        }
        let mut auth = Auth::new(self.plan.secret);
        auth.header.clone_from(&self.plan.header);
        auth.scheme.clone_from(&self.plan.scheme);
        let mut spec = RequestSpec::new(&self.plan.base).auth(auth);
        for segment in segments {
            spec = spec.segment(*segment);
        }
        for (key, value) in query {
            spec = spec.query(*key, *value);
        }
        match tokio::time::timeout(left, self.client.get_json(&spec)).await {
            Ok(result) => result.map_err(name_the_header),
            Err(_elapsed) => Err(timed_out(self.plan)),
        }
    }
}

fn timed_out(plan: &Plan<'_>) -> HarnessError {
    HarnessError::new(ErrorCode::Timeout, true).with_param("host", plan.host.clone())
}

/// The client reports a CA bundle it cannot use as a bare `TlsError`; say which input it was.
fn unusable_bundle(error: HarnessError, tls: &Tls) -> HarnessError {
    match tls {
        Tls::CustomCa(_) if error.code == ErrorCode::TlsError => {
            error.with_param("reason", "ca_invalid")
        }
        _ => error,
    }
}

/// The key was checked up front, so a header the client refuses is the header name itself.
fn name_the_header(error: HarnessError) -> HarnessError {
    if error.code == ErrorCode::InvalidInput && !error.params.contains_key("field") {
        error.with_param("field", "authHeader")
    } else {
        error
    }
}

async fn policy(tls: &Tls) -> Result<TlsPolicy, HarnessError> {
    Ok(match tls {
        Tls::System => TlsPolicy::System,
        Tls::Insecure(host) => TlsPolicy::Insecure { host: host.clone() },
        Tls::CustomCa(path) => TlsPolicy::CustomCa(read_pem(path).await?),
    })
}

/// The bundle is read on every poll so a renewed CA is picked up without a restart. The read can
/// block on a network share, hence the blocking pool.
async fn read_pem(path: &Path) -> Result<Vec<u8>, HarnessError> {
    let path = path.to_path_buf();
    match tokio::task::spawn_blocking(move || read_capped(&path, MAX_PEM_BYTES)).await {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(error)) => Err(ca_error(&error)),
        Err(_panicked) => Err(HarnessError::new(ErrorCode::Internal, false)),
    }
}

/// Only a reason code: the path is a username-bearing location and stays out of the view.
fn ca_error(error: &io::Error) -> HarnessError {
    let reason = match error.kind() {
        io::ErrorKind::NotFound => "ca_missing",
        io::ErrorKind::FileTooLarge => "ca_too_large",
        _ => "ca_unreadable",
    };
    HarnessError::new(ErrorCode::TlsError, false).with_param("reason", reason)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use usage_model::settings::LitellmSettings;
    use usage_support::Secret;

    use super::*;

    #[test]
    fn the_poll_budget_scales_with_the_timeout_and_stays_under_the_scheduler_limit() {
        let secs = Duration::from_secs;
        assert_eq!(budget(secs(1)), secs(3));
        assert_eq!(budget(secs(8)), secs(24));
        assert_eq!(budget(secs(9)), POLL_BUDGET);
        assert_eq!(budget(secs(30)), POLL_BUDGET);
        assert_eq!(budget(Duration::MAX), POLL_BUDGET);
    }

    #[test]
    fn ca_failures_keep_only_a_reason_code() {
        let kinds = [
            (io::ErrorKind::NotFound, "ca_missing"),
            (io::ErrorKind::FileTooLarge, "ca_too_large"),
            (io::ErrorKind::PermissionDenied, "ca_unreadable"),
            (io::ErrorKind::InvalidData, "ca_unreadable"),
        ];
        for (kind, reason) in kinds {
            let error = ca_error(&io::Error::new(kind, "C:\\Users\\someone\\ca.pem"));
            assert_eq!(error.code, ErrorCode::TlsError);
            assert!(!error.retryable);
            assert_eq!(error.params.get("reason").map(String::as_str), Some(reason));
            assert_eq!(error.params.len(), 1, "no path or message may leak: {error:?}");
        }
    }

    #[test]
    fn only_a_nameless_input_error_is_blamed_on_the_header() {
        let bare = HarnessError::new(ErrorCode::InvalidInput, false);
        assert_eq!(
            name_the_header(bare).params.get("field").map(String::as_str),
            Some("authHeader")
        );
        let named = HarnessError::new(ErrorCode::InvalidInput, false).with_param("field", "value");
        assert_eq!(name_the_header(named).params.get("field").map(String::as_str), Some("value"));
        let other = HarnessError::new(ErrorCode::Network, true);
        assert!(name_the_header(other).params.is_empty());
    }

    #[test]
    fn an_unusable_bundle_is_named_but_other_failures_are_left_alone() {
        let bare = || HarnessError::new(ErrorCode::TlsError, false);
        let ca = Tls::CustomCa("C:\\ca.pem".into());
        let named = unusable_bundle(bare(), &ca);
        assert_eq!(named.params.get("reason").map(String::as_str), Some("ca_invalid"));
        assert!(unusable_bundle(bare(), &Tls::System).params.is_empty());
        assert!(unusable_bundle(bare(), &Tls::Insecure("h".into())).params.is_empty());
        let internal = HarnessError::new(ErrorCode::Internal, false);
        assert!(unusable_bundle(internal, &ca).params.is_empty());
    }

    /// A healthy loopback proxy; the counter is the number of connections it accepted.
    async fn healthy_proxy() -> (String, Arc<AtomicUsize>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let base = format!("http://{}", listener.local_addr().expect("address"));
        let accepted = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&accepted);
        tokio::spawn(async move {
            while let Ok((mut stream, _peer)) = listener.accept().await {
                counter.fetch_add(1, Ordering::SeqCst);
                // Answer only once the whole request head is in, so the client never sees a reset.
                let (mut head, mut chunk) = (Vec::new(), [0_u8; 1024]);
                while !head.windows(4).any(|tail| tail == b"\r\n\r\n") && head.len() < 8192 {
                    match stream.read(&mut chunk).await {
                        Ok(0) | Err(_) => break,
                        Ok(read) => head.extend_from_slice(&chunk[..read]),
                    }
                }
                let body = r#"{"info":{"spend":1.0,"max_budget":2.0}}"#;
                let length = body.len();
                let reply = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
                     Content-Length: {length}\r\nConnection: close\r\n\r\n{body}"
                );
                let _ = stream.write_all(reply.as_bytes()).await;
                let _ = stream.shutdown().await;
            }
        });
        (base, accepted)
    }

    #[tokio::test]
    async fn too_little_poll_time_left_means_no_request_even_to_a_healthy_proxy() {
        let (base, accepted) = healthy_proxy().await;
        let secret = Secret::new("sk-fixture-budget");
        let settings = LitellmSettings {
            base_url: Some(base.clone()),
            timeout_secs: 5,
            ..LitellmSettings::default()
        };
        let plan = Plan::new(&settings, &base, &secret).expect("plan");
        let mut transport = Transport::open(&plan).await.expect("open");

        // Control: with time left the very same proxy answers, so the refusals below are the gate's.
        let body = transport.get(&["key", "info"], &[]).await.expect("control request");
        assert_eq!(body["info"]["max_budget"], 2.0);
        assert_eq!(accepted.load(Ordering::SeqCst), 1);

        for left in [Duration::ZERO, MIN_REQUEST_TIME / 2] {
            transport.deadline = Instant::now() + left;
            let error = transport.get(&["key", "info"], &[]).await.expect_err("not enough time");
            assert_eq!((error.code, error.retryable), (ErrorCode::Timeout, true), "left {left:?}");
            assert_eq!(error.params.get("host").map(String::as_str), Some("127.0.0.1"));
        }
        // Give a stray connection attempt the chance to show up before counting.
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(accepted.load(Ordering::SeqCst), 1, "a refused request must not connect");
    }
}
