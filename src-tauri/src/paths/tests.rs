use std::collections::HashMap;
use std::ffi::OsString;

use super::*;

/// A lookup over fabricated variables, so no test reads or writes the process environment.
fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
    let map: HashMap<String, OsString> =
        pairs.iter().map(|(k, v)| ((*k).to_owned(), OsString::from(*v))).collect();
    move |name| map.get(name).cloned()
}

fn abs(name: &str) -> PathBuf {
    std::env::temp_dir().join(name)
}

#[test]
fn nothing_set_means_no_overrides() {
    let got = Overrides::from_lookup(env(&[])).expect("parse");
    assert_eq!(got, Overrides::default());
    assert!(!got.is_active());
}

#[test]
fn absolute_directories_replace_the_resolved_ones() {
    let (data, home) = (abs("aism-data"), abs("aism-home"));
    let vars = [(DATA_DIR, data.to_str().expect("utf8")), (HOME_DIR, home.to_str().expect("utf8"))];
    let got = Overrides::from_lookup(env(&vars)).expect("parse");
    let paths = got.resolve(PathBuf::from("real-data"), PathBuf::from("real-home"));
    assert_eq!(paths.data_dir, data);
    assert_eq!(paths.home_dir, home);
    assert!(got.is_active());
}

#[test]
fn unset_directories_keep_the_resolved_ones() {
    let paths =
        Overrides::default().resolve(PathBuf::from("real-data"), PathBuf::from("real-home"));
    assert_eq!(paths.data_dir, PathBuf::from("real-data"));
    assert_eq!(paths.home_dir, PathBuf::from("real-home"));
}

#[test]
fn a_data_override_alone_never_exposes_the_real_home() {
    let data = abs("aism-only-data");
    let got = Overrides::from_lookup(env(&[(DATA_DIR, data.to_str().expect("utf8"))])).expect("ok");
    let paths = got.resolve(PathBuf::from("real-data"), PathBuf::from("real-home"));
    assert_eq!(paths.data_dir, data);
    assert_eq!(paths.home_dir, data.join("home"));
}

#[test]
fn a_home_override_alone_never_touches_the_real_data_dir() {
    let home = abs("aism-only-home");
    let got = Overrides::from_lookup(env(&[(HOME_DIR, home.to_str().expect("utf8"))])).expect("ok");
    let paths = got.resolve(PathBuf::from("real-data"), PathBuf::from("real-home"));
    assert_eq!(paths.home_dir, home);
    assert_eq!(paths.data_dir, home.join("aism-data"));
}

#[test]
fn flags_alone_leave_both_directories_real() {
    let got = Overrides::from_lookup(env(&[(SHOW_ON_START, "1")])).expect("ok");
    assert_eq!((got.data_root(), got.home_root()), (None, None));
}

#[test]
fn a_relative_or_empty_directory_is_refused_instead_of_ignored() {
    for bad in ["relative\\dir", "./here", ""] {
        let err = Overrides::from_lookup(env(&[(DATA_DIR, bad)])).expect_err("must refuse");
        assert_eq!(err, OverrideError(DATA_DIR), "{bad:?}");
        let err = Overrides::from_lookup(env(&[(HOME_DIR, bad)])).expect_err("must refuse");
        assert_eq!(err, OverrideError(HOME_DIR), "{bad:?}");
    }
}

#[test]
fn the_error_names_the_variable_but_not_the_value() {
    let err = Overrides::from_lookup(env(&[(DATA_DIR, "secret\\relative")])).expect_err("refuse");
    assert_eq!(err.to_string(), "AISM_DATA_DIR must be an absolute path");
}

#[test]
fn flags_are_on_only_for_exactly_one() {
    for (value, expected) in
        [("1", true), ("0", false), ("true", false), ("", false), ("11", false)]
    {
        let got = Overrides::from_lookup(env(&[(MEMORY_SECRETS, value), (SHOW_ON_START, value)]))
            .expect("parse");
        assert_eq!(got.memory_secrets, expected, "{value:?}");
        assert_eq!(got.show_on_start, expected, "{value:?}");
    }
}

#[test]
fn any_switch_marks_the_run_as_a_test_run() {
    let memory = Overrides::from_lookup(env(&[(MEMORY_SECRETS, "1")])).expect("parse");
    let show = Overrides::from_lookup(env(&[(SHOW_ON_START, "1")])).expect("parse");
    assert!(memory.is_active() && show.is_active());
}

#[test]
fn any_single_switch_keeps_secrets_out_of_the_real_credential_store() {
    let dir = abs("aism-secrets");
    let dir = dir.to_str().expect("utf8");
    let switches = [(DATA_DIR, dir), (HOME_DIR, dir), (SHOW_ON_START, "1"), (MEMORY_SECRETS, "1")];
    for (name, value) in switches {
        let got = Overrides::from_lookup(env(&[(name, value)])).expect("parse");
        assert!(got.uses_memory_secrets(), "{name} alone must keep secrets in memory");
    }
    assert!(!Overrides::default().uses_memory_secrets());
}

#[test]
fn the_process_environment_is_not_consulted_by_the_parser() {
    // The lookup closure is the only input: a variable the closure does not expose is invisible.
    let got = Overrides::from_lookup(|_| None).expect("parse");
    assert_eq!(got, Overrides::default());
}
