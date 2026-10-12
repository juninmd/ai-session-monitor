use std::path::PathBuf;

use super::*;

fn p(text: &str) -> PathBuf {
    PathBuf::from(text)
}

fn built(exe: &str, out: &str, chain: Option<&str>, home: &str) -> String {
    let chain = chain.map(PathBuf::from);
    build(&Args { exe: &p(exe), out: &p(out), chain: chain.as_deref(), home: &p(home) }).unwrap()
}

#[test]
fn plain_paths_are_written_bare_so_powershell_can_run_them() {
    let command = built(
        "C:\\Tools\\aism\\bin\\statusline-shim.exe",
        "C:\\Tools\\aism\\claude\\rate_limits.json",
        Some("C:\\Tools\\aism\\claude\\chain.json"),
        "C:\\Users\\ana",
    );
    assert_eq!(
        command,
        "C:/Tools/aism/bin/statusline-shim.exe --out C:/Tools/aism/claude/rate_limits.json \
         --chain C:/Tools/aism/claude/chain.json"
    );
}

#[test]
fn spaces_and_accents_under_home_use_the_tilde_form() {
    let command = built(
        "C:/Users/João Silva/AppData/Local/dev.aisessionmonitor.tray/bin/statusline-shim.exe",
        "C:/Users/João Silva/AppData/Local/dev.aisessionmonitor.tray/claude/rate_limits.json",
        None,
        "C:/Users/João Silva",
    );
    assert_eq!(
        command,
        "~/AppData/Local/dev.aisessionmonitor.tray/bin/statusline-shim.exe --out \
         ~/AppData/Local/dev.aisessionmonitor.tray/claude/rate_limits.json"
    );
}

#[test]
fn letters_of_any_script_stay_bare_because_every_shell_runs_them() {
    // Probed on Git Bash, pwsh 7, Windows PowerShell 5.1 and cmd: a bare accented, Cyrillic or CJK
    // path runs in all four, whereas `~` leans on the shell's HOME and a quoted program fails in
    // PowerShell. Only whitespace and shell syntax force the fallbacks.
    for user in ["João", "Иван", "李雷", "Ação-2"] {
        let base = format!("C:/Users/{user}/AppData/Local/aism");
        let (exe, out, chain) = (
            format!("{base}/bin/statusline-shim.exe"),
            format!("{base}/claude/rate_limits.json"),
            format!("{base}/claude/chain.json"),
        );
        let command = built(&exe, &out, Some(&chain), &format!("C:/Users/{user}"));
        assert_eq!(command, format!("{exe} --out {out} --chain {chain}"));
        assert_eq!(parse(&command, &p("C:/h")).unwrap().exe, exe, "{command}");
    }
}

#[test]
fn combining_marks_and_symbols_are_not_trusted_bare() {
    // "a" + U+0303 is decomposed text, `&` and `%` are shell syntax: these take the fallbacks.
    let home = "C:/Users/Ana";
    for name in ["Joa\u{303}o", "R&D", "100%"] {
        let exe = format!("C:/Users/Ana/{name}/statusline-shim.exe");
        let command = built(&exe, "C:/Users/Ana/o.json", None, home);
        assert!(command.starts_with("~/") || command.starts_with('"'), "{name}: {command}");
    }
}

#[test]
fn a_word_that_looks_like_a_flag_is_never_written_bare() {
    // Production paths are absolute; this keeps a relative one from being read as an option.
    let command = built("-x/statusline-shim.exe", "C:/o.json", None, "C:/h");
    assert!(command.starts_with("\"-x/statusline-shim.exe\" --out"), "{command}");
}

#[test]
fn paths_that_are_neither_plain_nor_under_home_are_double_quoted() {
    let command = built(
        "D:/Dados do usuário/bin/statusline-shim.exe",
        "D:/Dados do usuário/claude/rate_limits.json",
        Some("D:/Dados do usuário/claude/chain.json"),
        "C:/Users/ana",
    );
    assert_eq!(
        command,
        "\"D:/Dados do usuário/bin/statusline-shim.exe\" --out \
         \"D:/Dados do usuário/claude/rate_limits.json\" --chain \"D:/Dados do usuário/claude/chain.json\""
    );
}

#[test]
fn the_home_match_needs_a_path_boundary_and_ignores_case_only_on_windows() {
    let home = "C:/Users/Ana Maria";
    let near = built("C:/Users/Ana Mariana/bin/statusline-shim.exe", "C:/o.json", None, home);
    assert!(near.starts_with('"'), "Ana Mariana is not below Ana Maria: {near}");
    let upper = built("c:/USERS/ana maria/bin/statusline-shim.exe", "C:/o.json", None, home);
    assert_eq!(upper.starts_with("~/"), cfg!(windows), "{upper}");
}

#[test]
fn quoting_escapes_what_bash_would_expand() {
    let command = built("D:/we$ird `dir`/\"q\"/statusline-shim.exe", "D:/o.json", None, "C:/h");
    assert_eq!(command, "\"D:/we\\$ird \\`dir\\`/\\\"q\\\"/statusline-shim.exe\" --out D:/o.json");
    assert_eq!(
        parse(&command, &p("C:/h")).unwrap().exe,
        "D:/we$ird `dir`/\"q\"/statusline-shim.exe"
    );
}

#[test]
fn control_characters_cannot_be_written() {
    let exe = p("D:/line\nbreak/statusline-shim.exe");
    let args = Args { exe: &exe, out: &p("D:/o.json"), chain: None, home: &p("C:/h") };
    assert_eq!(build(&args), Err(Unsupported));
}

#[test]
fn verbatim_prefixes_are_dropped() {
    assert_eq!(forward(&p(r"\\?\C:\Users\ana\x.exe")).unwrap(), "C:/Users/ana/x.exe");
    assert_eq!(forward(&p(r"\\?\UNC\server\share\x.exe")).unwrap(), "//server/share/x.exe");
}

#[test]
fn everything_the_builder_writes_parses_back_to_the_same_paths() {
    let home = "C:/Users/João Silva";
    let cases = [
        (
            "C:/Tools/aism/bin/statusline-shim.exe",
            "C:/Tools/aism/rate_limits.json",
            Some("C:/Tools/aism/chain.json"),
        ),
        (
            "C:/Users/João Silva/AppData/a b/bin/statusline-shim.exe",
            "C:/Users/João Silva/AppData/a b/o.json",
            None,
        ),
        (
            "D:/Dados do usuário/bin/statusline-shim.exe",
            "D:/Dados do usuário/o.json",
            Some("D:/Dados do usuário/c.json"),
        ),
    ];
    for (exe, out, chain) in cases {
        let command = built(exe, out, chain, home);
        let parsed = parse(&command, &p(home)).unwrap_or_else(|| panic!("unparsed: {command}"));
        assert_eq!(parsed.exe, exe, "{command}");
        assert_eq!(parsed.out.as_deref(), Some(out), "{command}");
        assert_eq!(parsed.chain.as_deref(), chain, "{command}");
    }
}

#[test]
fn parse_accepts_single_quotes_and_backslash_paths_inside_quotes() {
    let parsed =
        parse(r#"'C:\Tools\statusline-shim.exe' --out "C:\x\o.json""#, &p("C:/h")).unwrap();
    assert_eq!(parsed.exe, "C:/Tools/statusline-shim.exe");
    assert_eq!(parsed.out.as_deref(), Some("C:/x/o.json"));
}

#[test]
fn other_peoples_commands_are_not_ours() {
    let home = p("C:/h");
    for command in [
        "jq -r '.model.display_name'",
        "~/.claude/statusline.sh",
        "powershell -NoProfile -File C:/Users/ana/.claude/statusline.ps1",
        "C:/x/not-statusline-shim.exe --out C:/o.json",
        "C:/x/statusline-shim.exe",
        "",
        "   ",
        "C:/x/statusline-shim.exe --out C:/o.json | tee log",
        "C:/x/statusline-shim.exe --out $(whoami)",
        "C:/x/statusline-shim.exe --out C:/o.json; rm -rf /",
        "C:/x/statusline-shim.exe --out \"unterminated",
        "C:\\x\\statusline-shim.exe --out C:/o.json",
        "C:/x/statusline-shim.exe --out C:/o.json\nsecond line",
    ] {
        assert_eq!(parse(command, &home), None, "{command:?}");
    }
}

#[test]
fn same_path_ignores_trailing_slashes_and_case_on_windows() {
    assert!(same_path("C:/Data/bin/x.exe", &p("C:\\Data\\bin\\x.exe")));
    assert!(same_path("C:/Data/bin/", &p("C:/Data/bin")));
    assert!(!same_path("C:/Data/bin/x.exe", &p("C:/Data/bin/y.exe")));
    assert_eq!(same_path("c:/data/BIN/x.exe", &p("C:/Data/bin/x.exe")), cfg!(windows));
}

#[test]
fn the_tilde_alone_and_tilde_slash_expand_but_other_tildes_do_not() {
    assert_eq!(expand_home("~", "C:/h"), "C:/h");
    assert_eq!(expand_home("~/x/y", "C:/h/"), "C:/h/x/y");
    assert_eq!(expand_home("~other/x", "C:/h"), "~other/x");
    assert_eq!(expand_home("C:/JOAOSI~1/x", "C:/h"), "C:/JOAOSI~1/x");
}
