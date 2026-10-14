use std::collections::BTreeSet;

use super::*;

const SOURCE: &str = include_str!("../commands.rs");
const LIB: &str = include_str!("../lib.rs");
const CONTRACT: &str = include_str!("../../../contract/ipc-commands.json");
const CAPABILITY: &str = include_str!("../../capabilities/default.json");

/// Names of the `#[tauri::command]` functions in commands.rs; each must be `pub async`.
fn declared() -> BTreeSet<String> {
    let lines: Vec<&str> = SOURCE.lines().map(str::trim).collect();
    lines
        .windows(2)
        .filter(|pair| pair[0] == "#[tauri::command]")
        .map(|pair| {
            let rest = pair[1].strip_prefix("pub async fn ").unwrap_or_else(|| {
                panic!(
                    "a command must be `pub async fn` (sync ones run on the main thread): {}",
                    pair[1]
                )
            });
            rest.split('(').next().unwrap_or_default().to_owned()
        })
        .collect()
}

/// Names passed to `generate_handler!` in lib.rs.
fn registered() -> BTreeSet<String> {
    let marker = "generate_handler![";
    let start = LIB.find(marker).expect("generate_handler! in lib.rs") + marker.len();
    let list = LIB[start..].split(']').next().expect("closing bracket");
    list.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|item| item.strip_prefix("commands::").unwrap_or(item).to_owned())
        .collect()
}

fn json(text: &str) -> serde_json::Value {
    serde_json::from_str(text).expect("valid json")
}

fn strings(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .expect("array")
        .iter()
        .map(|item| item.as_str().expect("string").to_owned())
        .collect()
}

fn contract() -> BTreeSet<String> {
    strings(&json(CONTRACT)["commands"]).into_iter().collect()
}

/// (`allow-*` grants as command names, every other permission).
fn capability() -> (BTreeSet<String>, Vec<String>) {
    let all = strings(&json(CAPABILITY)["permissions"]);
    let (allows, others): (Vec<String>, Vec<String>) =
        all.into_iter().partition(|name| name.starts_with("allow-"));
    let commands = allows.iter().map(|name| name["allow-".len()..].replace('-', "_")).collect();
    (commands, others)
}

#[test]
fn the_declared_commands_are_exactly_the_contract() {
    let contract = contract();
    assert_eq!(contract.len(), 20);
    assert_eq!(declared(), contract);
}

#[test]
fn every_command_is_registered_with_the_handler() {
    assert_eq!(registered(), contract());
}

#[test]
fn the_flyout_may_call_those_commands_and_nothing_else() {
    let (granted, others) = capability();
    assert_eq!(granted, contract());
    assert_eq!(others, ["core:event:allow-listen", "core:event:allow-unlisten"]);
    assert_eq!(json(CAPABILITY)["windows"], serde_json::json!(["flyout"]));
}

#[test]
fn the_flyout_listens_to_events_but_cannot_emit_them() {
    let all = strings(&json(CAPABILITY)["permissions"]);
    // The default set also grants `emit` and `emit_to`, which would let the webview speak as the shell.
    assert!(!all.iter().any(|name| name == "core:event:default"), "core:event:default is back");
    for emit in ["core:event:allow-emit", "core:event:allow-emit-to"] {
        assert!(!all.iter().any(|name| name == emit), "{emit} is granted");
    }
    // `listen` returns an unlisten handle that the UI calls on unmount: both are needed.
    for needed in ["core:event:allow-listen", "core:event:allow-unlisten"] {
        assert!(all.iter().any(|name| name == needed), "{needed} is missing");
    }
}

#[test]
fn a_secret_at_the_limit_is_accepted_and_one_byte_more_is_not() {
    assert!(check_secret(&"k".repeat(MAX_SECRET_BYTES)).is_ok());
    assert_eq!(check_secret(&"k".repeat(MAX_SECRET_BYTES + 1)), Err(invalid("value")));
    // The limit counts bytes: 'ç' takes two.
    assert_eq!(check_secret(&"ç".repeat(MAX_SECRET_BYTES / 2 + 1)), Err(invalid("value")));
    assert!(check_secret("").is_ok(), "emptiness is the service's call, with its own error");
}

#[test]
fn a_flyout_height_must_be_a_positive_finite_number() {
    assert_eq!(check_height(480.5), Ok(480.5));
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(check_height(bad), Err(invalid("height")), "{bad}");
    }
}

#[test]
fn blocking_work_returns_its_value_or_its_own_error() {
    let ok = tauri::async_runtime::block_on(blocking(|| Ok(7)));
    assert_eq!(ok, Ok(7));
    let failed = tauri::async_runtime::block_on(blocking::<(), _>(|| Err(invalid("value"))));
    assert_eq!(failed, Err(invalid("value")));
}

#[test]
fn a_panic_in_blocking_work_is_an_opaque_internal_error() {
    let outcome = tauri::async_runtime::block_on(blocking::<(), _>(|| panic!("boom-canary")));
    let err = outcome.expect_err("the panic must surface as an error");
    assert_eq!(err, internal("task"));
    assert!(!format!("{err:?}").contains("boom-canary"));
}
