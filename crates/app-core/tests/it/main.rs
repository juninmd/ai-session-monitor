//! The whole service through its public API only: fake collectors and shim, a recording sink, a
//! temp dir and tokio's paused clock, so minutes of scheduling run in milliseconds.

mod collector;
mod doubles;
mod fake_shim;
mod rig;
mod sink;
mod views;

mod adaptive;
mod backoff;
mod bad_collectors;
mod blocking;
mod cadence;
mod claude_shim;
mod coalescing;
mod concurrency;
mod cooldown;
mod expiry;
mod failures;
mod first_snapshot;
mod forced;
mod history;
mod live_settings;
mod notifications;
mod recovery;
mod refresh;
mod save_failure;
mod secret_flow;
mod sections;
mod service_api;
mod shutdown;
mod silent_reset;
mod supervision;
mod threshold_alerts;
