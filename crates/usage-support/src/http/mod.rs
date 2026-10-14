//! Hardened HTTP transport for collector JSON and signed GitHub release artifacts.
//!
//! What it guarantees, whatever the caller does:
//! - URLs are `https` (plain `http` only to loopback, and only when enabled), carry no userinfo
//!   or fragment, and are checked before any I/O.
//! - Collector redirects are never followed. Release fetching permits only restricted GitHub
//!   redirects. Proxies are never used, the whole call has a hard deadline and
//!   the body has a size cap that is enforced while streaming.
//! - Certificates are verified by the system store, by exactly one private CA bundle, or not at
//!   all for one named host (see [`TlsPolicy`]).
//! - Errors are `HarnessError`s whose params are only `host`, `httpStatus`, `retryAfterSecs` and
//!   `redirect`; credentials set with [`Secret`](crate::Secret) never appear in them.

mod body;
mod classify;
mod client;
mod options;
mod release;
mod request;
mod url_policy;

pub use client::SafeClient;
pub use options::{Auth, ClientOptions, TlsPolicy};
pub use release::{MAX_RELEASE_BYTES, release_url};
pub use request::RequestSpec;
pub use url_policy::{append_path, append_query, check_url};
