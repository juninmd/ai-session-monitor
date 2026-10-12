//! Everything the tray app does that does not need a window: settings, secrets, scheduling, the
//! shared snapshot, notification planning and the logic behind every IPC command. `src-tauri` is a
//! thin shell around this crate (tray, flyout window, plugins) and holds no business rules.
//!
//! The public surface below is the contract with the shell: items may be added, never renamed.
//!
//! Behaviour owned here that the shell must not duplicate:
//! - Settings recovery. [`AppService::new`] sets an unreadable or invalid `settings.json` aside as
//!   `settings.corrupt-<epoch>.json` and starts from defaults. [`AppService::run`] announces that
//!   with a [`NotificationKind::SettingsRecovered`] notification before its first snapshot, once.
//! - Supervision. One scheduler runs at a time. If its task dies (a panic in a sink, say) or is
//!   cancelled, the shell restarts polling by spawning [`AppService::run`] again; `shutdown` is final.
//! - Reset notifications. A window shown as out gets one [`NotificationKind::WindowReset`] at its
//!   reset time even if its source stays silent (the Claude statusLine moves only while Claude
//!   Code runs); a reading that later confirms the reset does not notify a second time.

pub mod clock;
pub mod collectors;
pub mod notify;
pub mod paths;
pub mod secrets;
pub mod service;

mod hub;
mod publish;
mod relay;
mod scheduler;
mod shared;
mod store;
mod sync;
mod timing;
mod validate;

pub use clock::{Clock, SystemClock};
pub use collectors::{Adapters, ClaudeShim, CollectFuture, Collector};
pub use notify::{Notification, NotificationKind};
pub use paths::AppPaths;
pub use secrets::{MemorySecretStore, SecretStore};
pub use service::{AppService, EventSink};
