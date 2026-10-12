//! End-to-end tests of the LiteLLM collector against a loopback fake proxy. Every fixture is
//! fabricated from the documented response shapes: no real proxy, key or user profile is touched.
// Test code asserts by panicking, like the `#[test]` functions that clippy.toml already exempts.
#![allow(clippy::unwrap_used, clippy::expect_used)]

/// A fixture compiled into the test binary, so a missing file is a build error and not a skipped
/// test.
macro_rules! fixture {
    ($name:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/", $name))
    };
}

mod common;

mod auth;
mod clock;
mod concurrency;
mod encoding;
mod errors;
mod follow_up;
mod gates;
mod key_scope;
mod leaks;
mod notices;
mod prefix;
mod tls_ca;
mod tls_handshake;
mod user_team;
