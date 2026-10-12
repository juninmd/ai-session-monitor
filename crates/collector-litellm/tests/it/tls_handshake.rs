//! Certificate checks against real TLS servers on loopback with throwaway certificates. Each
//! refusal is paired with a control that is accepted, and a refused handshake must never have
//! carried the key.

use std::fs;
use std::sync::Arc;
use std::time::Duration;

use rustls::ServerConfig;
use tempfile::TempDir;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use usage_model::ErrorCode;
use usage_model::settings::{LitellmSettings, TlsMode};

use crate::common::tls::{issued_by_new_ca, self_signed};
use crate::common::{Fake, Reply, assert_ready, error_of, meta, param, run, settings};

async fn https_proxy(config: Arc<ServerConfig>) -> Fake {
    Fake::start_tls(config, |_| Reply::json(fixture!("key_with_budget.json"))).await
}

fn trusting(base: &str, dir: &TempDir, ca_pem: &str) -> LitellmSettings {
    let path = dir.path().join("ca.pem");
    fs::write(&path, ca_pem).unwrap();
    let pem_path = path.to_string_lossy().into_owned();
    LitellmSettings { tls: TlsMode::CustomCa { pem_path }, ..settings(base) }
}

fn insecure(base: &str, host: &str) -> LitellmSettings {
    LitellmSettings { tls: TlsMode::Insecure { host: host.to_owned() }, ..settings(base) }
}

fn assert_refused(view: &usage_model::HarnessView, fake: &Fake) {
    let error = error_of(view);
    assert_eq!((error.code, error.retryable), (ErrorCode::TlsError, false), "{error:?}");
    assert_eq!(param(error, "host"), Some("127.0.0.1"));
    assert!(
        fake.requests().is_empty(),
        "a request, and the key with it, reached an unverified server"
    );
}

#[tokio::test]
async fn a_custom_ca_trusts_the_proxy_it_issued_and_the_key_travels() {
    let pki = issued_by_new_ca(&["127.0.0.1"]);
    let fake = https_proxy(pki.server).await;
    let dir = TempDir::new().unwrap();
    let view = run(&trusting(&fake.base(), &dir, &pki.ca_pem)).await;
    assert_ready(&view);
    assert_eq!(meta(&view).get("host"), Some(&"127.0.0.1"));
    let bearer = format!("Bearer {}", crate::common::SECRET);
    assert_eq!(fake.requests()[0].header("authorization"), Some(bearer.as_str()));
}

#[tokio::test]
async fn another_ca_is_refused_and_the_same_server_is_fine_for_its_own() {
    let (served, other) = (issued_by_new_ca(&["127.0.0.1"]), issued_by_new_ca(&["127.0.0.1"]));
    let fake = https_proxy(served.server).await;
    let dir = TempDir::new().unwrap();
    assert_refused(&run(&trusting(&fake.base(), &dir, &other.ca_pem)).await, &fake);
    assert_ready(&run(&trusting(&fake.base(), &dir, &served.ca_pem)).await);
}

#[tokio::test]
async fn a_trusted_ca_still_has_to_name_the_host() {
    let pki = issued_by_new_ca(&["other.example"]);
    let fake = https_proxy(pki.server).await;
    let dir = TempDir::new().unwrap();
    assert_refused(&run(&trusting(&fake.base(), &dir, &pki.ca_pem)).await, &fake);
}

#[tokio::test]
async fn the_system_roots_refuse_unknown_and_self_signed_certificates() {
    let unknown = https_proxy(issued_by_new_ca(&["127.0.0.1"]).server).await;
    assert_refused(&run(&settings(&unknown.base())).await, &unknown);
    let own = https_proxy(self_signed(&["127.0.0.1"])).await;
    assert_refused(&run(&settings(&own.base())).await, &own);
}

#[tokio::test]
async fn the_bundle_in_use_is_the_one_on_disk_at_poll_time() {
    let (old, renewed) = (issued_by_new_ca(&["127.0.0.1"]), issued_by_new_ca(&["127.0.0.1"]));
    let fake = https_proxy(renewed.server).await;
    let dir = TempDir::new().unwrap();
    let settings = trusting(&fake.base(), &dir, &old.ca_pem);
    assert_refused(&run(&settings).await, &fake);
    fs::write(dir.path().join("ca.pem"), &renewed.ca_pem).unwrap();
    assert_ready(&run(&settings).await);
}

#[tokio::test]
async fn insecure_mode_accepts_an_unverifiable_certificate_for_the_confirmed_host() {
    let fake = https_proxy(self_signed(&["other.example"])).await;
    for confirmed in ["127.0.0.1", " 127.0.0.1 ", "127.0.0.1."] {
        let view = run(&insecure(&fake.base(), confirmed)).await;
        assert_ready(&view);
        assert_eq!(meta(&view).get("host"), Some(&"127.0.0.1"));
    }
    assert_eq!(fake.requests().len(), 3);
}

#[tokio::test]
async fn insecure_mode_for_another_host_is_refused_before_connecting() {
    let fake = https_proxy(self_signed(&["127.0.0.1"])).await;
    for confirmed in ["localhost", "127.0.0.2", "", "   ", "127.0.0.1.evil.test"] {
        let view = run(&insecure(&fake.base(), confirmed)).await;
        let error = error_of(&view);
        assert_eq!(error.code, ErrorCode::InvalidInput, "{confirmed:?}");
        assert_eq!(param(error, "field"), Some("host"), "{confirmed:?}");
    }
    fake.assert_untouched().await;
}

#[tokio::test]
async fn plain_http_behind_an_https_address_is_a_tls_error() {
    // Answers the first bytes it receives with plain HTTP, which is what a misconfigured proxy
    // does when the address says https; the handshake can never complete, so no key is sent.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut tcp, _)) = listener.accept().await {
            let _ = tcp.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}").await;
            let _ = tcp.flush().await;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
    let view = run(&settings(&format!("https://127.0.0.1:{port}"))).await;
    let error = error_of(&view);
    assert_eq!((error.code, error.retryable), (ErrorCode::TlsError, false), "{error:?}");
}
