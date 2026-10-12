//! What the language server's answers mean: JSON in, pool windows out. Nothing here does I/O.

mod envelope;
mod merge;
mod model;
mod status;
mod summary;
mod values;

pub(crate) use merge::merge;
#[cfg(test)]
pub(crate) use model::Pool;
pub(crate) use model::{Cadence, PoolWindow, Read};
pub(crate) use status::parse_status;
pub(crate) use summary::parse_summary;
