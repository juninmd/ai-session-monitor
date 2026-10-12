//! Readers for single values of a response. Each is total: unreadable input is `None`.

mod cadence;
mod fraction;
mod reset;
mod text;

pub(super) use cadence::cadence_of;
pub(super) use fraction::used_pct;
pub(super) use reset::reset_at;
pub(super) use text::{clean, slug, tokens};
