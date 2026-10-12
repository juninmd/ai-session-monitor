use super::*;

#[test]
fn the_environment_value_is_trimmed_and_blank_means_unset() {
    assert_eq!(pick(false, Some("  D:\\x  ".to_owned())).as_deref(), Some("D:\\x"));
    assert_eq!(pick(false, Some("   ".to_owned())), None);
    assert_eq!(pick(false, Some(String::new())), None);
    assert_eq!(pick(false, None), None);
}

#[test]
fn a_test_run_never_follows_the_environment() {
    // The AISM_* switches exist so a smoke run cannot reach the real profile; this variable
    // usually points at it.
    assert_eq!(pick(true, Some("D:\\perfis\\claude".to_owned())), None);
}

#[test]
fn an_active_override_set_disables_the_lookup_itself() {
    let overrides = Overrides { memory_secrets: true, ..Overrides::default() };
    assert!(overrides.is_active());
    assert_eq!(from_env(&overrides), None);
}
