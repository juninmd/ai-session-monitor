//! The `statusLine.command` forms the installer writes (module docs of collector-claude's
//! `shim/command.rs`), run through the real shells Claude Code may pick, against a fake home in a
//! temp dir. The strings are spelled out here because this crate cannot depend on the installer:
//! its unit tests pin the exact text, this file proves each form really starts the shim and hands
//! it its paths intact. A cell a shell cannot run by design (a quoted program path in PowerShell,
//! `~` in cmd) is not exercised, and every form must run in at least one shell.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stderr,
    reason = "tests assert by panicking and say which shells ran"
)]

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use common::run;
use serde_json::{Value, json};

const SHIM: &str = env!("CARGO_BIN_EXE_aism-statusline-shim");
const PAYLOAD: &str = r#"{"cwd":"/secret","rate_limits":{"five_hour":{"used_percentage":12.5,"resets_at":1800000000}}}"#;

#[derive(Clone, Copy, Debug)]
enum Form {
    /// Plain characters only, written as is.
    Bare,
    /// `~/relative/path` under the home directory.
    Home,
    /// Double-quoted: spaces, accents, anything else.
    Quoted,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code, reason = "each platform detects a different subset of the shells")]
enum Kind {
    Posix,
    PowerShell,
    Cmd,
}

struct Shell {
    name: &'static str,
    kind: Kind,
    program: PathBuf,
}

/// The support matrix of `command.rs`: PowerShell reads a quoted program path as a string
/// expression, and cmd does not expand `~`.
fn supports(kind: Kind, form: Form) -> bool {
    !matches!((kind, form), (Kind::PowerShell, Form::Quoted) | (Kind::Cmd, Form::Home))
}

fn responds(program: &Path, probe: &[&str]) -> bool {
    Command::new(program).args(probe).output().is_ok_and(|out| out.status.success())
}

#[cfg(windows)]
fn detect() -> Vec<Shell> {
    let var = |name: &str| std::env::var_os(name).map(PathBuf::from);
    let system_root = var("SystemRoot").unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    let mut found = vec![Shell { name: "cmd", kind: Kind::Cmd, program: "cmd.exe".into() }];
    let powershell = system_root.join(r"System32\WindowsPowerShell\v1.0\powershell.exe");
    if powershell.is_file() {
        found.push(Shell { name: "powershell 5.1", kind: Kind::PowerShell, program: powershell });
    }
    if responds(Path::new("pwsh"), &["-NoProfile", "-Command", "exit 0"]) {
        found.push(Shell { name: "pwsh", kind: Kind::PowerShell, program: "pwsh".into() });
    }
    let git_bash = ["ProgramFiles", "ProgramW6432", "ProgramFiles(x86)"]
        .into_iter()
        .filter_map(var)
        .map(|root| root.join(r"Git\bin\bash.exe"))
        .find(|bash| bash.is_file());
    if let Some(program) = git_bash {
        found.push(Shell { name: "git bash", kind: Kind::Posix, program });
    }
    found
}

#[cfg(not(windows))]
fn detect() -> Vec<Shell> {
    ["sh", "bash"]
        .into_iter()
        .filter(|name| responds(Path::new(name), &["-c", "exit 0"]))
        .map(|name| Shell { name, kind: Kind::Posix, program: name.into() })
        .collect()
}

fn shells() -> &'static [Shell] {
    static SHELLS: OnceLock<Vec<Shell>> = OnceLock::new();
    SHELLS.get_or_init(detect)
}

fn invocation(shell: &Shell, text: &str) -> Command {
    let mut command = Command::new(&shell.program);
    match shell.kind {
        Kind::Posix => {
            command.arg("-c").arg(text);
        }
        Kind::PowerShell => {
            command.args(["-NoProfile", "-NonInteractive", "-Command"]).arg(text);
        }
        Kind::Cmd => cmd_arguments(&mut command, text),
    }
    command
}

#[cfg(windows)]
fn cmd_arguments(command: &mut Command, text: &str) {
    use std::os::windows::process::CommandExt;
    // `/S` strips exactly the outer pair of quotes, so the inner ones reach cmd untouched.
    command.args(["/D", "/S", "/C"]).raw_arg(format!("\"{text}\""));
}

#[cfg(not(windows))]
fn cmd_arguments(_: &mut Command, _: &str) {
    unreachable!("cmd is only detected on Windows");
}

fn is_plain(c: char) -> bool {
    c.is_alphanumeric() || "/._-:~".contains(c)
}

fn forward(path: &Path) -> String {
    path.to_str().expect("utf-8 temp path").replace('\\', "/")
}

fn exe_name() -> String {
    format!("statusline-shim{}", std::env::consts::EXE_SUFFIX)
}

/// Copies the shim to `<dir>/bin` and prepares `<dir>/claude`: the layout the installer makes.
fn place_shim(dir: &Path) -> (PathBuf, PathBuf) {
    let exe = dir.join("bin").join(exe_name());
    std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
    std::fs::create_dir_all(dir.join("claude")).unwrap();
    std::fs::copy(SHIM, &exe).unwrap();
    (exe, dir.join("claude").join("rate_limits.json"))
}

/// A temp root whose "home" is inside it, so no form can reach the real profile.
struct World {
    root: tempfile::TempDir,
    home: PathBuf,
}

impl World {
    fn new() -> Self {
        Self::at(tempfile::tempdir().unwrap())
    }

    /// A world under a root the bare form can address unquoted, if the machine has one.
    fn in_plain_root() -> Option<Self> {
        let roots = [std::env::temp_dir(), PathBuf::from(env!("CARGO_TARGET_TMPDIR"))];
        let root = roots.into_iter().find(|r| forward(r).chars().all(is_plain))?;
        std::fs::create_dir_all(&root).ok()?;
        Some(Self::at(tempfile::tempdir_in(root).unwrap()))
    }

    fn at(root: tempfile::TempDir) -> Self {
        let home = root.path().join("Usuário Ação");
        for dir in [&home, &root.path().join("roaming"), &root.path().join("local")] {
            std::fs::create_dir_all(dir).unwrap();
        }
        Self { root, home }
    }

    fn dir(&self, name: &str) -> PathBuf {
        self.root.path().join(name)
    }

    fn command(&self, shell: &Shell, text: &str) -> Command {
        let mut command = invocation(shell, text);
        // The working directory is the temp root, so a stray file shows up there.
        command.current_dir(self.root.path());
        command.env("HOME", &self.home).env("USERPROFILE", &self.home);
        command.env("APPDATA", self.dir("roaming")).env("LOCALAPPDATA", self.dir("local"));
        let home = self.home.to_string_lossy();
        // Windows PowerShell derives `$HOME` from these two rather than from USERPROFILE.
        if let (Some(drive), Some(path)) =
            (home.get(..2).filter(|d| d.ends_with(':')), home.get(2..))
        {
            command.env("HOMEDRIVE", drive).env("HOMEPATH", path);
        }
        command
    }

    /// Runs `text` in every shell that is meant to support `form` and expects the recording;
    /// returns the shells it ran in.
    fn exercise(
        &self,
        form: Form,
        text: &str,
        out: &Path,
        only: impl Fn(Kind) -> bool,
    ) -> Vec<&'static str> {
        let mut ran = Vec::new();
        for shell in shells().iter().filter(|s| supports(s.kind, form) && only(s.kind)) {
            let _ = std::fs::remove_file(out);

            let outcome = run(self.command(shell, text), PAYLOAD.as_bytes());

            let recorded: Option<Value> =
                std::fs::read(out).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok());
            let used =
                recorded.as_ref().map(|doc| &doc["rateLimits"]["five_hour"]["used_percentage"]);
            assert_eq!(
                used,
                Some(&json!(12.5)),
                "{form:?} form in {}: exit {:?}, stderr: {}",
                shell.name,
                outcome.code,
                String::from_utf8_lossy(&outcome.stderr).trim()
            );
            assert_eq!(outcome.code, Some(0), "{form:?} form in {}", shell.name);
            ran.push(shell.name);
        }
        eprintln!("{form:?} form ran in {ran:?}");
        ran
    }
}

fn at_least_one(form: Form, ran: &[&str]) {
    assert!(!ran.is_empty(), "no shell on this machine could run the {form:?} form");
}

#[test]
fn a_bare_path_with_letters_of_any_script_starts_the_shim_in_every_shell() {
    let Some(world) = World::in_plain_root() else {
        assert!(
            std::env::var_os("CI").is_none(),
            "no plain temp root: the bare form went unexercised"
        );
        eprintln!("skipped: no plain temp root on this machine");
        return;
    };
    let (exe, out) = place_shim(&world.dir("Иван-李雷"));
    let (exe, out_text) = (forward(&exe), forward(&out));

    let ran = world.exercise(Form::Bare, &format!("{exe} --out {out_text}"), &out, |_| true);

    at_least_one(Form::Bare, &ran);
}

#[test]
fn a_home_relative_path_starts_the_shim_in_every_shell_that_expands_the_tilde() {
    let world = World::new();
    let data = world.home.join("AppData").join("Local").join("ai-session-monitor");
    let (_, out) = place_shim(&data);
    let base = "~/AppData/Local/ai-session-monitor";
    let command = format!("{base}/bin/{} --out {base}/claude/rate_limits.json", exe_name());

    let ran = world.exercise(Form::Home, &command, &out, |_| true);

    at_least_one(Form::Home, &ran);
}

#[test]
fn a_quoted_path_with_spaces_and_accents_starts_the_shim_in_every_shell_that_accepts_it() {
    let world = World::new();
    let (exe, out) = place_shim(&world.dir("Dados do usuário (cópia)"));
    let command = format!("\"{}\" --out \"{}\"", forward(&exe), forward(&out));

    let ran = world.exercise(Form::Quoted, &command, &out, |_| true);

    at_least_one(Form::Quoted, &ran);
}

#[test]
fn shell_syntax_in_a_quoted_path_is_data_not_code() {
    let world = World::new();
    // Legal directory names on Windows and Unix alike. The installer escapes `$` and the backtick.
    let name = "x$(touch CANARY) `touch CANARY2` $HOME & 'q' ;";
    let escaped = r"x\$(touch CANARY) \`touch CANARY2\` \$HOME & 'q' ;";
    let (_, out) = place_shim(&world.dir(name));
    let base = forward(world.root.path());
    let command = format!(
        "\"{base}/{escaped}/bin/{}\" --out \"{base}/{escaped}/claude/rate_limits.json\"",
        exe_name()
    );

    // Only POSIX shells read `$` and the backtick inside double quotes; none at all is a skip.
    world.exercise(Form::Quoted, &command, &out, |kind| kind == Kind::Posix);

    for canary in ["CANARY", "CANARY2"] {
        assert!(!world.dir(canary).exists(), "{canary}: the path was run as shell code");
    }
}
