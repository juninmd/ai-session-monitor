//! Integration tests for `usage-support`. One binary keeps link time low with reqwest and rustls.
// Test code asserts by panicking, like the `#[test]` functions that clippy.toml already exempts.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;
mod fsutil;
mod http_limits;
mod http_policy;
mod http_proxy;
mod http_request;
mod http_status;
mod http_tls;
mod http_urls;
mod proc_resolve;
mod proc_run;
mod proc_session;
mod proc_tree;
mod redact;
