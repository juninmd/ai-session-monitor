//! `Secret`: a credential held in memory that cannot be printed, logged or serialized by accident.
//!
//! ```compile_fail
//! // No `Display`: a secret cannot be interpolated into a message.
//! fn needs_display<T: std::fmt::Display>(_: &T) {}
//! needs_display(&usage_support::Secret::new("x"));
//! ```
//!
//! ```compile_fail
//! // No `Serialize`: a secret cannot end up in a snapshot or an IPC payload.
//! fn needs_serialize<T: serde::Serialize>(_: &T) {}
//! needs_serialize(&usage_support::Secret::new("x"));
//! ```
//!
//! The controls below prove the two failures above come from the missing impls, not from a typo.
//! They only have to compile, so they are not linked: `cargo test --workspace` hands rustdoc the
//! link paths of every build script, including a stub `msvcrt.lib` from the Tauri shell.
//!
//! ```no_run
//! fn needs_debug<T: std::fmt::Debug>(_: &T) {}
//! needs_debug(&usage_support::Secret::new("x"));
//! fn needs_serialize<T: serde::Serialize>(_: &T) {}
//! needs_serialize(&"x");
//! ```

use std::fmt;

/// A credential held in memory. It cannot be printed, logged or serialized by accident.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// The only way to read the value, so every use site is greppable.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(***)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_output_never_contains_the_value() {
        let secret = Secret::new("sk-live-123");
        assert_eq!(format!("{secret:?}"), "Secret(***)");
        assert_eq!(secret.expose(), "sk-live-123");
    }

    #[test]
    fn pretty_debug_and_containers_stay_masked() {
        let secret = Secret::new("hunter2-fabricated");
        assert_eq!(format!("{secret:#?}"), "Secret(***)");
        let wrapped = Some(vec![secret.clone()]);
        assert!(!format!("{wrapped:?}").contains("hunter2"));
    }

    #[test]
    fn clone_and_equality_compare_the_value() {
        let a = Secret::new("same");
        assert_eq!(a, a.clone());
        assert_ne!(a, Secret::new("other"));
    }

    #[test]
    fn keeps_unicode_and_empty_values_verbatim() {
        assert_eq!(Secret::new("chave-çãõ-🔑").expose(), "chave-çãõ-🔑");
        assert_eq!(Secret::new("").expose(), "");
    }
}
