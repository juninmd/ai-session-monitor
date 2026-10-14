//! IO-free vocabulary of the monitor: harness ids, what collectors emit (`view`), the user's
//! non-secret configuration (`settings`) and the pace/status rules (`status`).

pub mod ctx;
pub mod harness;
pub mod history;
pub mod ipc;
pub mod settings;
pub mod status;
pub mod update;
pub mod view;

pub use ctx::Ctx;
pub use harness::HarnessId;
pub use history::{HarnessHistory, HistoryEpoch, HistoryPoint};
pub use ipc::{ExternalLink, SecretKind, SecretStatus, ShimState, ShimStatus};
pub use settings::Settings;
pub use status::{Pace, Status, Summary};
pub use update::{UpdateFailure, UpdatePhase, UpdateStatus};
pub use view::{
    Amount, AmountUnit, ErrorCode, Fact, HarnessError, HarnessView, Notice, Severity, Snapshot,
    Source, UsageWindow, ViewState, WindowKind,
};
