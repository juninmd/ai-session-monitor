//! The notice table in docs/ipc.md and the codes the collectors emit must be the same set: a code
//! missing from the table has no pt-BR text planned for it, and a row nobody emits is a promise the
//! UI keeps for nothing. Needs the real collector crates, hence the feature gate.
#![cfg(feature = "collectors")]

use std::collections::BTreeSet;
use std::path::Path;

const IPC_DOC: &str = "../docs/ipc.md";

/// Every collector crate with the codes it declares.
fn emitted() -> Vec<(&'static str, &'static [&'static str])> {
    vec![
        ("collector-claude", collector_claude::NOTICE_CODES),
        ("collector-codex", collector_codex::NOTICE_CODES),
        ("collector-cursor", collector_cursor::NOTICE_CODES),
        ("collector-copilot", collector_copilot::NOTICE_CODES),
        ("collector-antigravity", collector_antigravity::NOTICE_CODES),
        ("collector-litellm", collector_litellm::NOTICE_CODES),
        ("collector-opencode", collector_opencode::NOTICE_CODES),
        ("collector-kimi", collector_kimi::NOTICE_CODES),
        ("collector-zai", collector_zai::NOTICE_CODES),
        ("collector-openrouter", collector_openrouter::NOTICE_CODES),
        ("collector-deepseek", collector_deepseek::NOTICE_CODES),
        ("collector-deepinfra", collector_deepinfra::NOTICE_CODES),
        ("collector-moonshot", collector_moonshot::NOTICE_CODES),
        ("collector-novita", collector_novita::NOTICE_CODES),
        ("collector-ollama", collector_ollama::NOTICE_CODES),
        ("collector-minimax", collector_minimax::NOTICE_CODES),
    ]
}

/// The codes in the first column of the table under the "Notice codes" heading: a cell that is
/// exactly one backticked snake_case word. The header and separator rows have none.
fn documented(markdown: &str) -> Vec<String> {
    let mut lines = markdown.lines();
    let heading =
        lines.by_ref().find(|line| line.starts_with('#') && line.contains("Notice codes"));
    assert!(heading.is_some(), "docs/ipc.md has no heading named `Notice codes`");
    lines
        .take_while(|line| !line.starts_with('#'))
        .filter(|line| line.trim_start().starts_with('|'))
        .filter_map(|row| row.trim().trim_start_matches('|').split('|').next())
        .filter_map(|cell| cell.trim().strip_prefix('`')?.strip_suffix('`'))
        .filter(|code| {
            !code.is_empty()
                && code.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        })
        .map(str::to_owned)
        .collect()
}

fn ipc_doc() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(IPC_DOC);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("cannot read docs/ipc.md: {err}"))
}

#[test]
fn every_code_a_collector_emits_is_in_the_notice_table() {
    let table: BTreeSet<String> = documented(&ipc_doc()).into_iter().collect();
    assert!(!table.is_empty(), "the notice table was parsed as empty");
    let missing: Vec<String> = emitted()
        .into_iter()
        .flat_map(|(krate, codes)| codes.iter().map(move |code| (krate, *code)))
        .filter(|(_, code)| !table.contains(*code))
        .map(|(krate, code)| format!("`{code}` (emitted by {krate})"))
        .collect();
    assert!(missing.is_empty(), "missing from the notice table in docs/ipc.md: {missing:?}");
}

#[test]
fn every_code_in_the_notice_table_is_emitted_by_a_collector() {
    let all: BTreeSet<&str> =
        emitted().into_iter().flat_map(|(_, codes)| codes.iter().copied()).collect();
    assert!(!all.is_empty(), "no collector declares any notice code");
    let orphans: Vec<String> =
        documented(&ipc_doc()).into_iter().filter(|code| !all.contains(code.as_str())).collect();
    assert!(orphans.is_empty(), "in docs/ipc.md but emitted by no collector: {orphans:?}");
}

#[test]
fn the_table_parser_reads_only_the_backticked_first_column() {
    let sample = "\
# Doc

### Notice codes (initial catalog)

Prose with a `backticked` word and | a pipe.

| Code | Severity | Meaning |
|---|---|---|
| `alpha_one` | info | Has `inline code` and a `reason` param. |
|`beta2`|warn|No spaces around the cells|
| `Not_Snake` | info | Upper case is not a code. |
| `two`, `words` | info | Two cells' worth in one is skipped. |
| plain | info | No backticks. |
|  | info | Empty first cell. |

## Error codes

| `after_the_section` | x | Must not be read. |
";
    assert_eq!(documented(sample), ["alpha_one", "beta2"]);
}

#[test]
fn the_real_table_is_not_trivially_small() {
    // A heading rename or a reformatted table would otherwise turn both checks above vacuous.
    let table = documented(&ipc_doc());
    let declared: usize = emitted().iter().map(|(_, codes)| codes.len()).sum();
    assert!(table.len() >= declared, "{} documented vs {declared} declared", table.len());
}
