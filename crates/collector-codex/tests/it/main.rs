//! The collector against a fake `codex app-server` process and fabricated rollout files. Nothing
//! here touches a real profile: every directory is temporary and every value is invented.
// Helpers outside `#[test]` functions assert by panicking too, which clippy.toml cannot see.
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "tests assert by panicking")]

mod clock;
mod common;
mod errors;
mod fallback;
mod fixtures;
mod gates;
mod hygiene;
mod leaks;
mod mapping;
mod rollout_files;
mod states;
