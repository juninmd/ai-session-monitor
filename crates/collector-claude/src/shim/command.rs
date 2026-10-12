//! The `statusLine.command` string: how our shim is written into it and recognised in it.
//!
//! On Windows Claude Code runs the command through Git Bash when Git for Windows is installed and
//! through PowerShell otherwise (code.claude.com/docs/en/statusline, "Windows configuration"), and
//! it advises forward slashes and says `~` expands to the home directory. A command that starts
//! with a quoted path is a plain string expression to PowerShell (a parse error once arguments
//! follow; checked on pwsh 7.6 and Windows PowerShell 5.1), so the program path is written
//! without quotes whenever possible:
//!
//! 1. only plain characters (letters of any script, ASCII digits, `/ . _ - :`): written as is,
//!    which Git Bash, PowerShell and cmd all accept;
//! 2. under the home directory: `~/relative/path`, which Git Bash and PowerShell resolve (older
//!    PowerShell passes `~` to programs untouched, so the shim expands it itself). Git Bash reads
//!    `~` from its own HOME, so this form assumes HOME is the Windows profile, which is the default;
//! 3. otherwise double-quoted, which Git Bash and cmd accept but PowerShell does not for the program.

use std::path::Path;

use crate::paths::EXE_STEM;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Unsupported;

pub(crate) struct Args<'a> {
    pub exe: &'a Path,
    pub out: &'a Path,
    pub chain: Option<&'a Path>,
    pub home: &'a Path,
}

/// A recognised command with every path resolved to forward slashes (`~` expanded).
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Parsed {
    pub exe: String,
    pub out: Option<String>,
    pub chain: Option<String>,
}

pub(crate) fn build(args: &Args<'_>) -> Result<String, Unsupported> {
    let home = forward(args.home)?;
    let mut command = word(&forward(args.exe)?, &home)?;
    command.push_str(" --out ");
    command.push_str(&word(&forward(args.out)?, &home)?);
    if let Some(chain) = args.chain {
        command.push_str(" --chain ");
        command.push_str(&word(&forward(chain)?, &home)?);
    }
    Ok(command)
}

/// Forward slashes, no verbatim (`\\?\`) prefix.
pub(crate) fn forward(path: &Path) -> Result<String, Unsupported> {
    let text = path.to_str().ok_or(Unsupported)?.replace('\\', "/");
    if let Some(rest) = text.strip_prefix("//?/UNC/") {
        return Ok(format!("//{rest}"));
    }
    if let Some(rest) = text.strip_prefix("//?/") {
        return Ok(rest.to_owned());
    }
    Ok(text)
}

fn is_plain(text: &str) -> bool {
    !text.is_empty()
        && !text.starts_with('-')
        && text.chars().all(|c| {
            c.is_alphabetic() || c.is_ascii_digit() || matches!(c, '/' | '.' | '_' | '-' | ':')
        })
}

fn word(path: &str, home: &str) -> Result<String, Unsupported> {
    if is_plain(path) {
        return Ok(path.to_owned());
    }
    if let Some(rest) = under(path, home).filter(|rest| is_plain(rest)) {
        return Ok(format!("~/{rest}"));
    }
    if path.chars().any(char::is_control) {
        return Err(Unsupported);
    }
    let mut quoted = String::with_capacity(path.len() + 2);
    quoted.push('"');
    for c in path.chars() {
        if matches!(c, '"' | '\\' | '$' | '`') {
            quoted.push('\\');
        }
        quoted.push(c);
    }
    quoted.push('"');
    Ok(quoted)
}

/// The part of `path` below `home` (both with forward slashes), if it is below it.
fn under(path: &str, home: &str) -> Option<String> {
    let home = home.trim_end_matches('/');
    if home.is_empty() || path.len() <= home.len() {
        return None;
    }
    let head = path.get(..home.len())?;
    let same = if cfg!(windows) { head.eq_ignore_ascii_case(home) } else { head == home };
    let rest = path.get(home.len()..)?.strip_prefix('/')?;
    (same && !rest.is_empty()).then(|| rest.to_owned())
}

/// Whether two forward-slash paths name the same file; Windows paths ignore ASCII case.
pub(crate) fn same_path(resolved: &str, expected: &Path) -> bool {
    let Ok(expected) = forward(expected) else { return false };
    let (a, b) = (resolved.trim_end_matches('/'), expected.trim_end_matches('/'));
    if cfg!(windows) { a.eq_ignore_ascii_case(b) } else { a == b }
}

/// `Some` only for a plain `<shim> --out <file> [--chain <file>]` command; anything with shell
/// syntax beyond quoting is somebody else's.
pub(crate) fn parse(command: &str, home: &Path) -> Option<Parsed> {
    let home = forward(home).ok()?;
    let words = split(command)?;
    let (program, flags) = words.split_first()?;
    let file_name = program.rsplit(['/', '\\']).next()?;
    let ours = [EXE_STEM.to_owned(), format!("{EXE_STEM}.exe")];
    if !ours.iter().any(|name| name.eq_ignore_ascii_case(file_name)) {
        return None;
    }
    let (mut out, mut chain) = (None, None);
    let mut flags = flags.iter();
    while let Some(flag) = flags.next() {
        match flag.as_str() {
            "--out" => out = flags.next().cloned(),
            "--chain" => chain = flags.next().cloned(),
            _ => {}
        }
    }
    let resolve = |word: &str| expand_home(word, &home);
    Some(Parsed {
        exe: resolve(program),
        out: Some(resolve(&out?)),
        chain: chain.as_deref().map(resolve),
    })
}

fn expand_home(word: &str, home: &str) -> String {
    let word = word.replace('\\', "/");
    match word.strip_prefix('~') {
        Some("") => home.to_owned(),
        Some(rest) if rest.starts_with('/') => format!("{}{rest}", home.trim_end_matches('/')),
        _ => word,
    }
}

/// Splits like a shell would for the quoting this module writes. Command substitution, pipes,
/// redirections, other expansions, unquoted backslashes and newlines make it `None`.
fn split(command: &str) -> Option<Vec<String>> {
    let mut words = Vec::new();
    let (mut current, mut open) = (String::new(), false);
    let mut chars = command.chars();
    while let Some(c) = chars.next() {
        match c {
            '\n' | '\r' | '\\' | '|' | '&' | ';' | '<' | '>' | '(' | ')' | '$' | '`' => {
                return None;
            }
            c if c.is_whitespace() => {
                if open {
                    words.push(std::mem::take(&mut current));
                    open = false;
                }
            }
            '\'' => {
                open = true;
                loop {
                    match chars.next()? {
                        '\'' => break,
                        c => current.push(c),
                    }
                }
            }
            '"' => {
                open = true;
                loop {
                    match chars.next()? {
                        '"' => break,
                        '$' | '`' => return None,
                        '\\' => match chars.next()? {
                            escaped @ ('\\' | '"' | '$' | '`') => current.push(escaped),
                            other => {
                                current.push('\\');
                                current.push(other);
                            }
                        },
                        c => current.push(c),
                    }
                }
            }
            c => {
                open = true;
                current.push(c);
            }
        }
    }
    if open {
        words.push(current);
    }
    Some(words)
}

#[cfg(test)]
mod tests;
