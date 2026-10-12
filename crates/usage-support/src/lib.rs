//! Shared plumbing for collectors: a hardened HTTP client, atomic file writes, bounded child
//! processes, secret handling and log redaction.

pub mod fsutil;
pub mod http;
pub mod proc;
pub mod redact;
pub mod secret;
pub mod time;

pub use fsutil::{atomic_write, read_capped, read_json_capped};
pub use http::{
    Auth, ClientOptions, RequestSpec, SafeClient, TlsPolicy, append_path, append_query, check_url,
};
pub use proc::{
    CommandSpec, DEFAULT_MAX_LINE, LineSession, ProcOutput, resolve_binary,
    resolve_binary_from_env, run_capped,
};
pub use redact::scrub;
pub use secret::Secret;
pub use time::parse_rfc3339_secs;
