//! Command line: `--out <file>` and optional `--chain <file>`. Anything else is ignored, so a
//! newer app can add flags without breaking an older shim that is still configured.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Args {
    pub out: Option<PathBuf>,
    pub chain: Option<PathBuf>,
}

pub fn parse(args: impl IntoIterator<Item = OsString>, home: Option<&OsStr>) -> Args {
    let mut parsed = Args::default();
    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        let slot = match flag.to_str() {
            Some("--out") => &mut parsed.out,
            Some("--chain") => &mut parsed.chain,
            _ => continue,
        };
        *slot =
            args.next().filter(|value| !value.is_empty()).map(|value| expand_tilde(&value, home));
    }
    parsed
}

/// PowerShell 5.1 passes `~` through literally (Git Bash and PowerShell 7 expand it), so the
/// command line the installer writes stays valid in every shell Claude Code may use.
fn expand_tilde(arg: &OsStr, home: Option<&OsStr>) -> PathBuf {
    let (Some(text), Some(home)) = (arg.to_str(), home) else {
        return PathBuf::from(arg);
    };
    match text.strip_prefix('~') {
        Some("") => PathBuf::from(home),
        Some(rest) if rest.starts_with(['/', '\\']) => PathBuf::from(home).join(&rest[1..]),
        _ => PathBuf::from(arg),
    }
}

/// `USERPROFILE` is the Windows home; `HOME` covers Unix and Git Bash overrides.
pub fn home_from_env() -> Option<OsString> {
    let pick = |name: &str| std::env::var_os(name).filter(|value| !value.is_empty());
    if cfg!(windows) { pick("USERPROFILE").or_else(|| pick("HOME")) } else { pick("HOME") }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_strs(args: &[&str], home: Option<&str>) -> Args {
        parse(args.iter().map(OsString::from), home.map(OsStr::new))
    }

    #[test]
    fn reads_out_and_chain() {
        let args = parse_strs(&["--out", "a/b.json", "--chain", "c.json"], None);
        assert_eq!(args.out, Some(PathBuf::from("a/b.json")));
        assert_eq!(args.chain, Some(PathBuf::from("c.json")));
    }

    #[test]
    fn unknown_flags_and_stray_values_are_ignored() {
        let args = parse_strs(&["--future", "--out", "o.json", "stray", "--verbose"], None);
        assert_eq!(args, Args { out: Some(PathBuf::from("o.json")), chain: None });
    }

    #[test]
    fn a_flag_without_a_value_or_with_an_empty_one_is_unset() {
        assert_eq!(parse_strs(&["--out"], None), Args::default());
        assert_eq!(parse_strs(&["--out", "", "--chain"], None), Args::default());
    }

    #[test]
    fn last_occurrence_wins() {
        let args = parse_strs(&["--out", "first", "--out", "second"], None);
        assert_eq!(args.out, Some(PathBuf::from("second")));
    }

    #[test]
    fn tilde_expands_only_as_a_leading_path_segment() {
        let home = Some("/home/ana lima");
        let args = parse_strs(&["--out", "~/x/o.json", "--chain", "~"], home);
        assert_eq!(args.out, Some(PathBuf::from("/home/ana lima").join("x/o.json")));
        assert_eq!(args.chain, Some(PathBuf::from("/home/ana lima")));
        for literal in ["~user/x", "a/~/b", "~x", "x~"] {
            let args = parse_strs(&["--out", literal], home);
            assert_eq!(args.out, Some(PathBuf::from(literal)), "{literal}");
        }
    }

    #[test]
    fn tilde_stays_literal_without_a_home_directory() {
        let args = parse_strs(&["--out", "~/o.json"], None);
        assert_eq!(args.out, Some(PathBuf::from("~/o.json")));
    }

    #[test]
    fn tilde_with_a_backslash_separator_expands_too() {
        let args = parse_strs(&["--out", r"~\AppData\o.json"], Some("home"));
        assert_eq!(args.out, Some(PathBuf::from("home").join(r"AppData\o.json")));
    }
}
