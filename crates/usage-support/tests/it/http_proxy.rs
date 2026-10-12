//! A proxy named in the environment must never see a request: it would capture the credential.
//! The environment of a running test process cannot be changed safely, so the parent starts this
//! same test binary as a child with hostile variables and checks who got the traffic.

use std::process::Stdio;

use tokio::process::Command;
use usage_support::{ClientOptions, RequestSpec, SafeClient, TlsPolicy};

use crate::common::tls::self_signed;
use crate::common::{Reply, TestServer, client, options};

const PLAIN_URL: &str = "AI_SM_TEST_PROXY_PLAIN_URL";
const TLS_URL: &str = "AI_SM_TEST_PROXY_TLS_URL";

/// The child's job. It is not a check by itself and does nothing unless the parent set the
/// variables; the parent proves it ran by counting the requests that reached the targets.
#[tokio::test]
async fn child_entry_for_the_proxy_test() {
    let (Ok(plain), Ok(tls)) = (std::env::var(PLAIN_URL), std::env::var(TLS_URL)) else {
        return;
    };
    client().get_json(&RequestSpec::new(plain)).await.unwrap();
    let tls_options =
        ClientOptions { tls: TlsPolicy::Insecure { host: "127.0.0.1".into() }, ..options() };
    let insecure = SafeClient::new(tls_options).unwrap();
    insecure.get_json(&RequestSpec::new(tls)).await.unwrap();
}

#[tokio::test]
async fn proxy_variables_in_the_environment_are_ignored() {
    let decoy = TestServer::start(|_| Reply::json(200, r#"{"via":"proxy"}"#)).await;
    let plain = TestServer::start(|_| Reply::json(200, r#"{"via":"direct"}"#)).await;
    let tls = TestServer::start_tls(self_signed(&["127.0.0.1"]), |_| {
        Reply::json(200, r#"{"via":"direct"}"#)
    })
    .await;

    let proxy = format!("http://127.0.0.1:{}", decoy.addr.port());
    let mut child = Command::new(std::env::current_exe().unwrap());
    child
        .args(["--exact", "http_proxy::child_entry_for_the_proxy_test", "--test-threads", "1"])
        .env(PLAIN_URL, plain.url("/"))
        .env(TLS_URL, tls.url("/"))
        .env_remove("NO_PROXY")
        .env_remove("no_proxy")
        .stdin(Stdio::null());
    for name in ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY"] {
        child.env(name, &proxy);
    }
    let output = child.output().await.unwrap();

    assert!(output.status.success(), "child failed: {}", String::from_utf8_lossy(&output.stdout));
    assert_eq!(plain.requests().len(), 1, "the plain request did not reach its target");
    assert_eq!(tls.requests().len(), 1, "the TLS request did not reach its target");
    assert_eq!(decoy.connections(), 0, "a request went to the proxy named in the environment");
}
