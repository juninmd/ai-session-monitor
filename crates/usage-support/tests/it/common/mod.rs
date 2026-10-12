//! Shared test support: a loopback HTTP server, throwaway certificates, client options and the
//! peer process.

pub mod peer;
mod reply;
pub mod server;
pub mod tls;

use std::time::Duration;

pub use reply::Reply;
pub use server::TestServer;

/// Options for tests: loopback `http` allowed and a deadline that fails fast on a hang.
pub fn options() -> usage_support::ClientOptions {
    usage_support::ClientOptions {
        timeout: Duration::from_secs(5),
        allow_loopback_http: true,
        ..usage_support::ClientOptions::default()
    }
}

pub fn client() -> usage_support::SafeClient {
    usage_support::SafeClient::new(options()).unwrap()
}
