//! Integration tests of the public API of `collector-claude`. One binary keeps link time low
//! with reqwest and rustls. Everything runs in temp dirs with fabricated files; nothing here
//! can reach the network (no login file, or the gate returns first).
// Test code asserts by panicking, like the `#[test]` functions that clippy.toml already exempts.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;
mod gates;
mod invariants;
mod lifecycle;
mod oauth_offline;
mod setup;
mod views;
mod views_clock;
mod views_errors;
