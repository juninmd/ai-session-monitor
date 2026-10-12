//! Every file the Claude collector reads or writes, derived from the settings and the context.
//! Nothing here touches the disk or the process environment.

use std::path::{Path, PathBuf};

use usage_model::Ctx;
use usage_model::settings::ClaudeSettings;

pub(crate) const EXE_STEM: &str = "statusline-shim";
/// The stable copy of the shim that Claude Code's `statusLine` points at.
pub(crate) const EXE_NAME: &str = if cfg!(windows) { "statusline-shim.exe" } else { EXE_STEM };

#[derive(Clone, Debug)]
pub(crate) struct Paths {
    pub home: PathBuf,
    /// `<config dir>/settings.json`, the file the installer edits.
    pub settings_json: PathBuf,
    /// Written once, before the first edit.
    pub backup: PathBuf,
    /// A second copy of `state` beside the file it describes, so it outlives the app data dir.
    pub state_copy: PathBuf,
    /// Claude Code's own login file; only read when the optional OAuth source is on.
    pub credentials: PathBuf,
    // The five below are per config dir (`profile_key`): two Claude profiles never share one.
    /// Written by the shim, read by the collector.
    pub rate_limits: PathBuf,
    /// The `statusLine` that was configured before the install, for an exact restore.
    pub state: PathBuf,
    /// `{"command": ...}`: what the shim runs after recording.
    pub chain: PathBuf,
    pub oauth_cache: PathBuf,
    /// Throttle and last answer of the `claude -p "/usage"` source.
    pub cli_cache: PathBuf,
    pub exe: PathBuf,
    /// Directories that hold only our files, deepest first; each goes once it is empty.
    pub own_dirs: [PathBuf; 3],
}

impl Paths {
    /// `None` when a directory is unusable: relative (it would depend on the current directory,
    /// and the data dir ends up in the statusLine command, where a relative program path runs
    /// whatever sits in Claude Code's working directory) or holding a NUL byte. Callers report
    /// it as `configDir`, the only one of the three a user can edit.
    pub(crate) fn resolve(settings: &ClaudeSettings, ctx: &Ctx) -> Option<Self> {
        let config_dir = match settings.config_dir.as_deref().map(str::trim) {
            Some(dir) if !dir.is_empty() => PathBuf::from(dir),
            _ => ctx.home_dir.join(".claude"),
        };
        let usable = |dir: &Path| dir.is_absolute() && !dir.to_string_lossy().contains('\0');
        let dirs = [config_dir.as_path(), ctx.data_dir.as_path(), ctx.home_dir.as_path()];
        if !dirs.into_iter().all(usable) {
            return None;
        }
        let (claude_data, bin) = (ctx.data_dir.join("claude"), ctx.data_dir.join("bin"));
        let key = profile_key(&config_dir, &ctx.home_dir);
        let named = |stem: &str| match &key {
            Some(key) => claude_data.join(format!("{stem}-{key}.json")),
            None => claude_data.join(format!("{stem}.json")),
        };
        // The file name stays `statusline-shim[.exe]`: `command::parse` recognises ours by it.
        let exe_dir = key.as_deref().map_or_else(|| bin.clone(), |key| bin.join(key));
        Some(Self {
            home: ctx.home_dir.clone(),
            settings_json: config_dir.join("settings.json"),
            backup: config_dir.join("settings.json.aism-backup"),
            state_copy: config_dir.join("settings.json.aism-state"),
            credentials: config_dir.join(".credentials.json"),
            rate_limits: named("rate_limits"),
            state: named("shim-state"),
            chain: named("chain"),
            oauth_cache: named("oauth-usage"),
            cli_cache: named("cli-usage"),
            exe: exe_dir.join(EXE_NAME),
            own_dirs: [exe_dir, bin, claude_data],
        })
    }
}

/// Which Claude profile the files in the app data dir belong to: `None` for the default
/// `<home>/.claude`, whose files keep their plain names, else 16 hex digits of FNV-1a 64 over the
/// normalised config dir. Written out because `DefaultHasher` may change between Rust releases and
/// these names must not.
fn profile_key(config_dir: &Path, home: &Path) -> Option<String> {
    let dir = normalize(config_dir);
    (dir != normalize(&home.join(".claude"))).then(|| format!("{:016x}", fnv1a64(dir.as_bytes())))
}

/// One spelling per directory: forward slashes, no trailing slash, ASCII-lowercase (Windows paths
/// ignore case; like `command::same_path` this does not fold non-ASCII letters).
fn normalize(dir: &Path) -> String {
    let text = dir.to_string_lossy().replace('\\', "/").to_ascii_lowercase();
    text.trim_end_matches('/').to_owned()
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    bytes.iter().fold(OFFSET_BASIS, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(PRIME))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testenv::Env;

    #[test]
    fn the_config_dir_defaults_to_dot_claude_under_home() {
        let env = Env::new();
        let paths = Paths::resolve(&ClaudeSettings::default(), &env.ctx()).unwrap();
        assert_eq!(paths.settings_json, env.home.join(".claude").join("settings.json"));
        assert_eq!(paths.credentials, env.home.join(".claude").join(".credentials.json"));
        assert_eq!(paths.backup, env.home.join(".claude").join("settings.json.aism-backup"));
    }

    #[test]
    fn an_explicit_config_dir_wins_and_blank_means_default() {
        let env = Env::new();
        let other = env.root().join("outro diretório");
        let padded = format!("  {}  ", other.display());
        let mut settings = ClaudeSettings { config_dir: Some(padded), ..ClaudeSettings::default() };
        let paths = Paths::resolve(&settings, &env.ctx()).unwrap();
        assert_eq!(paths.settings_json, other.join("settings.json"));
        assert_eq!(paths.credentials, other.join(".credentials.json"));

        settings.config_dir = Some("   ".into());
        let paths = Paths::resolve(&settings, &env.ctx()).unwrap();
        assert_eq!(paths.settings_json, env.settings_json());
    }

    #[test]
    fn unusable_config_dirs_are_refused() {
        let env = Env::new();
        let with_nul = format!("{}\0x", env.root().display());
        for bad in ["relative/dir", ".claude", "C:drive-relative", with_nul.as_str()] {
            let settings =
                ClaudeSettings { config_dir: Some(bad.into()), ..ClaudeSettings::default() };
            assert!(Paths::resolve(&settings, &env.ctx()).is_none(), "{bad:?}");
        }
    }

    #[test]
    fn a_relative_data_or_home_dir_is_refused() {
        // The data dir ends up in Claude Code's statusLine command; relative, it would run
        // whatever `bin/statusline-shim` sits in the directory Claude Code was started in.
        let env = Env::new();
        for ctx in [
            Ctx::new(crate::testenv::NOW, "relative/data", env.home.clone()),
            Ctx::new(crate::testenv::NOW, env.data.clone(), "relative/home"),
        ] {
            assert!(Paths::resolve(&ClaudeSettings::default(), &ctx).is_none());
        }
    }

    fn resolve_in(env: &Env, config_dir: &str) -> Paths {
        let settings =
            ClaudeSettings { config_dir: Some(config_dir.to_owned()), ..ClaudeSettings::default() };
        Paths::resolve(&settings, &env.ctx()).unwrap()
    }

    /// What an install owns in the app data dir.
    fn data_files(paths: &Paths) -> [&PathBuf; 6] {
        [
            &paths.rate_limits,
            &paths.state,
            &paths.chain,
            &paths.oauth_cache,
            &paths.cli_cache,
            &paths.exe,
        ]
    }

    #[test]
    fn app_files_live_under_the_data_dir() {
        let env = Env::new();
        let paths = Paths::resolve(&ClaudeSettings::default(), &env.ctx()).unwrap();
        assert_eq!(paths.rate_limits, env.rate_limits());
        assert_eq!(paths.state, env.state_file());
        assert_eq!(paths.chain, env.chain_file());
        assert_eq!(paths.oauth_cache, env.oauth_cache());
        assert_eq!(paths.cli_cache, env.cli_cache());
        assert_eq!(paths.exe, env.shim_copy());
        assert_eq!(paths.home, env.home);
        assert_eq!(paths.state_copy, env.state_copy(), "beside settings.json, not in the data dir");
        assert_eq!(paths.exe.parent(), Some(paths.own_dirs[0].as_path()));
        assert!(paths.own_dirs.iter().all(|dir| dir.starts_with(&env.data) && *dir != env.data));
    }

    #[test]
    fn the_default_config_dir_keeps_the_plain_file_names_however_it_is_spelled() {
        let env = Env::new();
        let default = Paths::resolve(&ClaudeSettings::default(), &env.ctx()).unwrap();
        let dir = env.config_dir().display().to_string();
        let spellings = [
            dir.clone(),
            format!("{dir}{}", std::path::MAIN_SEPARATOR),
            dir.replace('\\', "/"),
            dir.to_ascii_uppercase(),
        ];
        for spelling in spellings {
            let paths = resolve_in(&env, &spelling);
            assert_eq!(data_files(&paths), data_files(&default), "{spelling}");
        }
    }

    #[test]
    fn another_config_dir_gets_files_of_its_own_named_after_a_hash_of_it() {
        let env = Env::new();
        let other = env.root().join("outro perfil");
        let paths = resolve_in(&env, &other.display().to_string());

        let key = format!("{:016x}", fnv1a64(normalize(&other).as_bytes()));
        let claude = env.data.join("claude");
        assert_eq!(paths.rate_limits, claude.join(format!("rate_limits-{key}.json")));
        assert_eq!(paths.state, claude.join(format!("shim-state-{key}.json")));
        assert_eq!(paths.chain, claude.join(format!("chain-{key}.json")));
        assert_eq!(paths.oauth_cache, claude.join(format!("oauth-usage-{key}.json")));
        assert_eq!(paths.exe, env.data.join("bin").join(&key).join(EXE_NAME));
        assert_eq!(paths.state_copy, other.join("settings.json.aism-state"));
        let name = std::ffi::OsStr::new(EXE_NAME);
        assert_eq!(paths.exe.file_name(), Some(name), "ours is recognised by its file name");
        assert_eq!(paths.exe.parent(), Some(paths.own_dirs[0].as_path()));
        assert_eq!(paths.own_dirs[1], env.data.join("bin"), "then the parent, once empty");
        assert!(paths.own_dirs.iter().all(|dir| dir.starts_with(&env.data) && *dir != env.data));
    }

    #[test]
    fn different_config_dirs_never_share_a_file_and_one_dir_spelled_twice_does() {
        let env = Env::new();
        let (a, b) = (env.root().join("perfil a"), env.root().join("perfil b"));
        let (default, a1, b1) = (
            Paths::resolve(&ClaudeSettings::default(), &env.ctx()).unwrap(),
            resolve_in(&env, &a.display().to_string()),
            resolve_in(&env, &b.display().to_string()),
        );
        let all: Vec<_> = [&default, &a1, &b1].into_iter().flat_map(data_files).collect();
        let distinct: std::collections::BTreeSet<_> = all.iter().collect();
        assert_eq!(distinct.len(), all.len(), "{all:?}");

        let again = resolve_in(&env, &format!("{}/", a.display().to_string().to_ascii_uppercase()));
        assert_eq!(data_files(&again), data_files(&a1));
    }

    #[test]
    fn the_key_is_a_fixed_function_of_the_normalised_dir() {
        // Published FNV-1a 64 test vectors: files written by one build must be found by the next.
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);

        let key = |dir: &str| profile_key(Path::new(dir), Path::new("C:/Users/Me"));
        assert_eq!(key("c:/users/me/.claude-work").as_deref(), Some("f5175fb1cad5ff01"));
        assert_eq!(key("C:\\Users\\Me\\.Claude-Work\\\\").as_deref(), Some("f5175fb1cad5ff01"));
        assert_eq!(key("C:/Users/Me/.claude"), None, "the default profile has no key");
        assert_eq!(key("c:\\USERS\\me\\.Claude\\"), None);
    }
}
