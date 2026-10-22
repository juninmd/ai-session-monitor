//! The optional read-only local API: the current readings as JSON on loopback, for scripts, status
//! bars and widgets. It is the only place the app listens on a port, so it is deliberately small
//! and strict. Off until `general.local_api` is on; the shell starts and stops it as that setting
//! changes (`LocalApi` stops when dropped).
//!
//! Security model, each point pinned by a test:
//! - Bound to `127.0.0.1` only, on [`LOCAL_API_PORT`]; no other interface, no IPv6.
//! - Only `GET`. A `Host` other than `127.0.0.1:<port>` / `localhost:<port>` is refused (DNS
//!   rebinding), and so is any request with an `Origin` header (a web page must not read it). No
//!   CORS headers are ever sent, so a browser cannot read a response either way.
//! - Bounded: request head size and header count, one overall read deadline per connection (a
//!   slow client cannot pin a thread), a write timeout and a cap on concurrent connections.
//! - The body is built from the same `Snapshot` the webview gets, narrowed to presentation data:
//!   no errors, notices, meta, labels or paths. Nothing about a request is ever logged.

mod body;
mod connection;
mod request;
mod response;
mod server;

#[cfg(test)]
mod tests;

pub use server::{LocalApi, Provider, Reading, StartError};
pub use usage_model::settings::LOCAL_API_PORT;
