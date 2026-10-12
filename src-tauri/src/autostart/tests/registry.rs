//! The Run-value probe against a throwaway subtree of HKCU, never the real Run key.

use std::sync::atomic::{AtomicU32, Ordering};

use windows_registry::{CURRENT_USER, Key};

use super::*;

const NAME: &str = "AI Session Monitor";

static NEXT: AtomicU32 = AtomicU32::new(0);

/// A volatile subtree removed on drop; volatile also drops it at sign-out if a test gets killed.
struct Scratch {
    root: String,
}

impl Scratch {
    fn new() -> Self {
        let n = NEXT.fetch_add(1, Ordering::SeqCst);
        Self { root: format!(r"Software\ai-session-monitor-test-{}-{n}", std::process::id()) }
    }

    fn run_key(&self) -> String {
        format!(r"{}\Run", self.root)
    }

    fn create_run_key(&self) -> Key {
        let mut options = CURRENT_USER.options();
        options.read().write().create().volatile().open(self.run_key()).expect("scratch key")
    }

    fn registered(&self, name: &str) -> bool {
        run_value_exists(CURRENT_USER, &self.run_key(), name).expect("probe")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = CURRENT_USER.remove_tree(&self.root);
    }
}

#[test]
fn a_missing_key_or_value_reads_as_not_registered() {
    let scratch = Scratch::new();
    assert!(!scratch.registered(NAME), "no key");
    let _key = scratch.create_run_key();
    assert!(!scratch.registered(NAME), "key without the value");
}

#[test]
fn only_the_value_it_is_asked_for_counts() {
    let scratch = Scratch::new();
    let key = scratch.create_run_key();
    key.set_string("Other App", r#""C:\Program Files\Other\other.exe" "#).expect("set");
    assert!(!scratch.registered(NAME), "another app's value");
    key.set_string(NAME, r#""C:\Program Files\AI Session Monitor\app.exe" "#).expect("set");
    assert!(scratch.registered(NAME));
    assert!(!scratch.registered("ai session"), "a different name");
}

#[test]
fn it_reads_the_per_user_run_key() {
    // Tests only ever touch a scratch key, so a typo in the real path would go unseen.
    assert_eq!(RUN_KEY, r"Software\Microsoft\Windows\CurrentVersion\Run");
}
