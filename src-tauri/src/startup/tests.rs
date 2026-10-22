use std::collections::BTreeSet;

use usage_model::ErrorCode;
use usage_model::settings::Language;

use super::*;

const ALL_LANGUAGES: [Language; 10] = [
    Language::En,
    Language::ZhCn,
    Language::Hi,
    Language::Es,
    Language::Fr,
    Language::Ar,
    Language::Bn,
    Language::PtBr,
    Language::Ru,
    Language::Ur,
];

fn every_reason() -> Vec<Reason> {
    let mut all = vec![Reason::Paths, Reason::DataDir, Reason::Window, Reason::Tray];
    all.extend(ErrorCode::ALL.map(Reason::Service));
    all
}

#[test]
fn a_failed_start_leaves_with_a_failure_code() {
    // 0 would tell the installer, the Run key and a script that nothing went wrong.
    assert_ne!(EXIT_CODE, 0);
}

#[test]
fn every_reason_has_a_notice_in_portuguese_that_names_its_code() {
    for reason in every_reason() {
        let notice = reason.notice(Language::PtBr);
        assert_eq!(notice.title, "AI Session Monitor não iniciou", "{reason:?}");
        assert!(notice.body.contains(&format!("código: {}", reason.code())), "{reason:?}");
        assert!(notice.body.ends_with("O aplicativo vai fechar."), "{reason:?}");
    }
}

#[test]
fn codes_are_snake_case_and_tell_every_reason_apart() {
    let codes: BTreeSet<String> = every_reason().into_iter().map(Reason::code).collect();
    assert_eq!(codes.len(), every_reason().len(), "two reasons share a code");
    for code in &codes {
        assert!(
            !code.is_empty() && code.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'),
            "{code:?}"
        );
    }
}

#[test]
fn a_service_reason_uses_the_wire_name_of_the_error_code_not_its_debug_name() {
    let reason = Reason::Service(ErrorCode::SettingsUnreadable);
    assert_eq!(reason.code(), "settings_unreadable");
    let body = reason.notice(Language::PtBr).body;
    assert!(!body.contains("SettingsUnreadable"), "{body}");
    assert!(body.contains("configurações"), "the user is told which file is the problem: {body}");
}

#[test]
fn every_error_code_maps_to_the_name_the_contract_pins() {
    let pinned: BTreeSet<String> =
        serde_json::from_str(include_str!("../../../contract/error-codes.json")).expect("json");
    let ours: BTreeSet<String> =
        ErrorCode::ALL.into_iter().map(|code| Reason::Service(code).code()).collect();
    assert_eq!(ours, pinned);
}

#[test]
fn a_notice_carries_no_path_value_or_template_leftover() {
    for language in ALL_LANGUAGES {
        for reason in every_reason() {
            let notice = reason.notice(language);
            for text in [notice.title, notice.body.as_str()] {
                for forbidden in ['\\', '/', '{', '}', '"', '\n', '\0'] {
                    assert!(
                        !text.contains(forbidden),
                        "{language:?} {reason:?} has {forbidden:?} in {text:?}"
                    );
                }
            }
        }
    }
}
