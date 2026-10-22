//! Test support and suites. All fixtures, tokens and databases are fabricated; nothing reads a
//! real profile, and nothing leaves the loopback interface.

mod fake_server;
mod flow;
mod gates;
mod http;
mod jwt;
mod parse;
mod paths;
mod store;
mod support;

pub(crate) use support::*;
