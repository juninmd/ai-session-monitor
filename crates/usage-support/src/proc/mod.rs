//! Child processes with hard limits: run to completion with capped output (`run_capped`), talk to
//! a long-lived child line by line (`LineSession`), and find a binary without trusting the
//! current directory (`resolve_binary`).
//!
//! What it guarantees, whatever the caller does:
//! - Programs start from an argument array, never a shell string, and without a console window.
//! - Output is read through fixed-size buffers and kept only up to the caps, so a chatty child
//!   can neither exhaust memory nor stall on a full pipe.
//! - A timeout, a dropped future or a dropped session ends the child and (on Windows, through
//!   `taskkill /T`) its descendants.
//! - Errors carry no params: paths, arguments and output stay out of them.

mod capture;
mod lines;
mod resolve;
mod run;
mod session;
mod spec;
mod tree;

pub use resolve::{resolve_binary, resolve_binary_from_env};
pub use run::run_capped;
pub use session::{DEFAULT_MAX_LINE, LineSession};
pub use spec::{CommandSpec, ProcOutput};
