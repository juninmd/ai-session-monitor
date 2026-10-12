//! Which shell runs the chained command. Claude Code runs status line commands through Git Bash
//! when Git for Windows is installed and through PowerShell otherwise (documented at
//! code.claude.com/docs/en/statusline, "Windows configuration"), so the previous command is run
//! the same way. A bare `bash` on PATH is deliberately not used: on Windows it is usually the WSL
//! launcher, which cannot even see `C:/` paths.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shell {
    /// Git Bash, or `sh` elsewhere: `<shell> -c <command>`.
    Posix(PathBuf),
    /// `pwsh.exe` or Windows PowerShell's `powershell.exe`: `-Command <command>`.
    Pwsh(PathBuf),
    /// Last resort on a machine with neither of the above.
    Cmd,
}

const GIT_BASH_NAMES: [&str; 4] = ["bash.exe", "sh.exe", "bash", "sh"];

pub fn detect(env: &impl Fn(&str) -> Option<OsString>) -> Shell {
    if cfg!(windows) { windows_shell(env) } else { Shell::Posix(PathBuf::from("/bin/sh")) }
}

pub fn windows_shell(env: &impl Fn(&str) -> Option<OsString>) -> Shell {
    if let Some(bash) = find_git_bash(env) {
        return Shell::Posix(bash);
    }
    match find_powershell(env) {
        Some(powershell) => Shell::Pwsh(powershell),
        None => Shell::Cmd,
    }
}

fn find_git_bash(env: &impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    // Claude Code ignores this variable unless it names a real bash/sh, and so does the shim.
    if let Some(configured) = env("CLAUDE_CODE_GIT_BASH_PATH").map(PathBuf::from) {
        let named_like_bash = configured
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| GIT_BASH_NAMES.contains(&name.to_ascii_lowercase().as_str()));
        if named_like_bash && configured.is_file() {
            return Some(configured);
        }
    }
    // git.exe lives in <root>\cmd, <root>\bin or <root>\mingw64\bin; bash is <root>\bin\bash.exe.
    if let Some(git) = find_in_path(env, &["git.exe"])
        && let Some(bash) = git.parent().and_then(bash_near_git)
    {
        return Some(bash);
    }
    ["ProgramFiles", "ProgramW6432", "ProgramFiles(x86)"]
        .iter()
        .filter_map(|name| env(name))
        .map(|dir| PathBuf::from(dir).join("Git"))
        .chain(env("LOCALAPPDATA").map(|dir| PathBuf::from(dir).join("Programs").join("Git")))
        .map(|root| root.join("bin").join("bash.exe"))
        .find(|candidate| candidate.is_file())
}

fn bash_near_git(git_dir: &Path) -> Option<PathBuf> {
    git_dir
        .ancestors()
        .take(3)
        .map(|root| root.join("bin").join("bash.exe"))
        .find(|candidate| candidate.is_file())
}

fn find_powershell(env: &impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    let system = env("SystemRoot").map(|root| {
        PathBuf::from(root)
            .join("System32")
            .join("WindowsPowerShell")
            .join("v1.0")
            .join("powershell.exe")
    });
    find_in_path(env, &["pwsh.exe"])
        .or_else(|| find_in_path(env, &["powershell.exe"]))
        .or(system.filter(|path| path.is_file()))
}

fn find_in_path(env: &impl Fn(&str) -> Option<OsString>, names: &[&str]) -> Option<PathBuf> {
    let path = env("PATH")?;
    std::env::split_paths(&path)
        .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
        .find(|candidate| candidate.is_file())
}

/// The process to spawn for `command`; the caller wires the standard streams.
pub fn command_for(shell: &Shell, command: &str, respect_execution_policy: bool) -> Command {
    let mut process = match shell {
        Shell::Posix(path) => {
            let mut process = Command::new(path);
            process.arg("-c").arg(command);
            process
        }
        Shell::Pwsh(path) => {
            let mut process = Command::new(path);
            process.args(["-NoProfile", "-NonInteractive"]);
            if !respect_execution_policy {
                process.args(["-ExecutionPolicy", "Bypass"]);
            }
            process.arg("-Command").arg(command);
            process
        }
        Shell::Cmd => cmd_command(command),
    };
    hide_window(&mut process);
    process
}

#[cfg(windows)]
fn cmd_command(command: &str) -> Command {
    use std::os::windows::process::CommandExt;
    let mut process = Command::new("cmd.exe");
    // `/S /C "<command>"`: cmd strips exactly one outer pair of quotes and keeps the inner ones.
    process.raw_arg("/S").raw_arg("/C").raw_arg(format!("\"{command}\""));
    process
}

#[cfg(not(windows))]
fn cmd_command(command: &str) -> Command {
    let mut process = Command::new("/bin/sh");
    process.arg("-c").arg(command);
    process
}

/// Without this a console child of a window-less parent opens a visible console on Windows.
#[cfg(windows)]
pub fn hide_window(process: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    process.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
pub fn hide_window(_process: &mut Command) {}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;

    use super::*;

    struct Fake {
        vars: HashMap<String, OsString>,
    }

    impl Fake {
        fn new(vars: &[(&str, &Path)]) -> Self {
            let vars = vars.iter().map(|(k, v)| ((*k).to_owned(), v.as_os_str().to_owned()));
            Self { vars: vars.collect() }
        }

        fn lookup(&self) -> impl Fn(&str) -> Option<OsString> + '_ {
            |name| self.vars.get(name).cloned()
        }
    }

    fn touch(path: &Path) -> PathBuf {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"x").unwrap();
        path.to_path_buf()
    }

    #[test]
    fn git_bash_is_found_from_git_on_path() {
        let root = tempfile::tempdir().unwrap();
        let git = touch(&root.path().join("Git/cmd/git.exe"));
        let bash = touch(&root.path().join("Git/bin/bash.exe"));
        let fake = Fake::new(&[("PATH", git.parent().unwrap())]);
        assert_eq!(windows_shell(&fake.lookup()), Shell::Posix(bash));
    }

    #[test]
    fn git_bash_is_found_when_only_mingw_bin_is_on_path() {
        let root = tempfile::tempdir().unwrap();
        let git = touch(&root.path().join("Git/mingw64/bin/git.exe"));
        let bash = touch(&root.path().join("Git/bin/bash.exe"));
        let fake = Fake::new(&[("PATH", git.parent().unwrap())]);
        assert_eq!(windows_shell(&fake.lookup()), Shell::Posix(bash));
    }

    #[test]
    fn the_configured_path_wins_but_only_when_it_is_a_real_bash() {
        let root = tempfile::tempdir().unwrap();
        let custom = touch(&root.path().join("tools/bash.exe"));
        let program_files = root.path().join("pf");
        let standard = touch(&program_files.join("Git/bin/bash.exe"));
        let fake =
            Fake::new(&[("CLAUDE_CODE_GIT_BASH_PATH", &custom), ("ProgramFiles", &program_files)]);
        assert_eq!(windows_shell(&fake.lookup()), Shell::Posix(custom));

        let not_bash = touch(&root.path().join("tools/notepad.exe"));
        let fake = Fake::new(&[
            ("CLAUDE_CODE_GIT_BASH_PATH", &not_bash),
            ("ProgramFiles", &program_files),
        ]);
        assert_eq!(windows_shell(&fake.lookup()), Shell::Posix(standard.clone()));

        let missing = root.path().join("nope/bash.exe");
        let fake =
            Fake::new(&[("CLAUDE_CODE_GIT_BASH_PATH", &missing), ("ProgramFiles", &program_files)]);
        assert_eq!(windows_shell(&fake.lookup()), Shell::Posix(standard));
    }

    #[test]
    fn a_bare_bash_on_path_is_never_used() {
        let root = tempfile::tempdir().unwrap();
        let wsl = touch(&root.path().join("System32/bash.exe"));
        let fake = Fake::new(&[("PATH", wsl.parent().unwrap())]);
        assert_eq!(windows_shell(&fake.lookup()), Shell::Cmd);
    }

    #[test]
    fn powershell_is_the_fallback_and_pwsh_is_preferred() {
        let root = tempfile::tempdir().unwrap();
        let pwsh = touch(&root.path().join("ps7/pwsh.exe"));
        let legacy = touch(&root.path().join("ps5/powershell.exe"));
        let both =
            std::env::join_paths([pwsh.parent().unwrap(), legacy.parent().unwrap()]).unwrap();
        let vars = HashMap::from([("PATH".to_owned(), both)]);
        assert_eq!(windows_shell(&|name| vars.get(name).cloned()), Shell::Pwsh(pwsh));

        let only_legacy = Fake::new(&[("PATH", legacy.parent().unwrap())]);
        assert_eq!(windows_shell(&only_legacy.lookup()), Shell::Pwsh(legacy));
    }

    #[test]
    fn windows_powershell_is_found_under_system_root_without_path() {
        let root = tempfile::tempdir().unwrap();
        let system = touch(&root.path().join("System32/WindowsPowerShell/v1.0/powershell.exe"));
        let fake = Fake::new(&[("SystemRoot", root.path())]);
        assert_eq!(windows_shell(&fake.lookup()), Shell::Pwsh(system));
    }

    #[test]
    fn nothing_found_falls_back_to_cmd() {
        let fake = Fake::new(&[]);
        assert_eq!(windows_shell(&fake.lookup()), Shell::Cmd);
    }

    #[test]
    fn posix_and_powershell_commands_pass_the_text_as_one_argument() {
        let text = "echo \"a b\" && echo $HOME";
        let args_of = |process: &Command| -> Vec<String> {
            process.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect()
        };
        let posix = command_for(&Shell::Posix(PathBuf::from("bash")), text, false);
        assert_eq!(args_of(&posix), ["-c", text]);

        let powershell = command_for(&Shell::Pwsh(PathBuf::from("pwsh")), text, false);
        let args = args_of(&powershell);
        assert_eq!(args.last().map(String::as_str), Some(text));
        assert!(args.contains(&"Bypass".to_owned()));

        let strict = command_for(&Shell::Pwsh(PathBuf::from("pwsh")), text, true);
        assert!(!args_of(&strict).contains(&"Bypass".to_owned()), "policy respected on request");
    }
}
