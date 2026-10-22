use std::collections::BTreeSet;

use super::*;

const ALL: [Language; 10] = [
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

/// The names between braces, in order of appearance; a stray brace is reported as `?`.
fn placeholders(template: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = template;
    while let Some(open) = rest.find(['{', '}']) {
        if rest.as_bytes()[open] == b'}' {
            found.push("?".to_owned());
            rest = &rest[open + 1..];
            continue;
        }
        let tail = &rest[open + 1..];
        match tail.find(['{', '}']) {
            Some(close) if tail.as_bytes()[close] == b'}' => {
                found.push(tail[..close].to_owned());
                rest = &tail[close + 1..];
            }
            _ => {
                found.push("?".to_owned());
                rest = tail;
            }
        }
    }
    found
}

#[test]
fn every_language_declares_exactly_the_placeholders_of_every_field() {
    for language in ALL {
        for entry in messages(language).entries() {
            let used: BTreeSet<String> = placeholders(entry.template).into_iter().collect();
            let wanted: BTreeSet<String> =
                entry.placeholders.iter().map(|name| (*name).to_owned()).collect();
            assert_eq!(used, wanted, "{language:?} {}: {:?}", entry.field, entry.template);
        }
    }
}

#[test]
fn filling_every_declared_placeholder_leaves_no_brace_behind() {
    for language in ALL {
        for entry in messages(language).entries() {
            let values: Vec<(&str, &str)> =
                entry.placeholders.iter().map(|name| (*name, "7")).collect();
            let filled = fill(entry.template, &values);
            assert!(!filled.contains(['{', '}']), "{language:?} {}: {filled:?}", entry.field);
            assert!(!filled.trim().is_empty(), "{language:?} {} is empty", entry.field);
        }
    }
}

#[test]
fn no_language_leaves_a_field_empty_and_all_have_the_same_fields() {
    let english: Vec<&str> = messages(Language::En).entries().iter().map(|e| e.field).collect();
    for language in ALL {
        let entries = messages(language).entries();
        let fields: Vec<&str> = entries.iter().map(|e| e.field).collect();
        assert_eq!(fields, english, "{language:?}");
        for entry in entries {
            assert!(!entry.template.is_empty(), "{language:?} {}", entry.field);
        }
    }
}

#[test]
fn a_decimal_separator_is_one_character() {
    for language in ALL {
        let separator = messages(language).decimal_separator;
        assert_eq!(separator.chars().count(), 1, "{language:?} {separator:?}");
    }
}

#[test]
fn fill_replaces_named_placeholders_and_keeps_the_rest() {
    assert_eq!(fill("{a} and {b}", &[("a", "1"), ("b", "2")]), "1 and 2");
    assert_eq!(fill("{b}{a}{b}", &[("a", "x"), ("b", "y")]), "yxy");
    assert_eq!(fill("no placeholders", &[]), "no placeholders");
    assert_eq!(fill("{unknown} {a}", &[("a", "1")]), "{unknown} 1");
    assert_eq!(fill("open { brace and } close {a", &[("a", "1")]), "open { brace and } close {a");
    assert_eq!(fill("{{a}}", &[("a", "1")]), "{1}");
    assert_eq!(fill("ação {a} · 日本", &[("a", "é")]), "ação é · 日本");
}

#[test]
fn fill_never_scans_a_value_again() {
    let hostile = "{span} {harness}";
    assert_eq!(
        fill("{window}: {span}", &[("window", hostile), ("span", "5 min")]),
        "{span} {harness}: 5 min"
    );
}
