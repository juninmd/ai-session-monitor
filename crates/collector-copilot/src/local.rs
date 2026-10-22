//! The GitHub OAuth token the Copilot extension keeps locally. Read only with consent, only a few
//! fixed files, never the whole profile: `apps.json` then `hosts.json` under the extension's own
//! directory. The token is held as a [`Secret`] and never reaches a log, a view or an error.

use std::path::{Path, PathBuf};

use serde_json::Value;
use usage_support::{Secret, read_json_capped};

/// Both files are a few hundred bytes; a bigger one is not what the extension writes.
const MAX_FILE_BYTES: usize = 64 * 1024;
const FILES: [&str; 2] = ["apps.json", "hosts.json"];
const DIR: &str = "github-copilot";
/// OAuth tokens are about 40 characters; the header must stay one clean value.
const MAX_TOKEN_LEN: usize = 512;

/// Why no token was found. Fixed identifiers only: shown as the error's `reason` param.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    /// Neither file exists.
    NotFound,
    /// A file exists but could not be read or is not the JSON object the extension writes.
    Unreadable,
    /// Files were read, but none holds a usable `github.com` token.
    NoToken,
}

impl Reason {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::Unreadable => "unreadable",
            Self::NoToken => "no_token",
        }
    }
}

/// Windows keeps the extension's data under `%LOCALAPPDATA%` (`<home>\AppData\Local`), the other
/// systems under `~/.config`. The process environment is deliberately never consulted.
pub(crate) fn candidate_dirs(home: &Path, windows: bool) -> Vec<PathBuf> {
    if windows {
        vec![home.join("AppData").join("Local").join(DIR)]
    } else {
        vec![home.join(".config").join(DIR)]
    }
}

pub(crate) async fn find_token(home: PathBuf) -> Result<Secret, Reason> {
    let dirs = candidate_dirs(&home, cfg!(windows));
    // Blocking file reads stay off the async workers; a lost task reads as "not readable".
    tokio::task::spawn_blocking(move || find_in(&dirs)).await.unwrap_or(Err(Reason::Unreadable))
}

pub(crate) fn find_in(dirs: &[PathBuf]) -> Result<Secret, Reason> {
    let mut seen = Reason::NotFound;
    for dir in dirs {
        for file in FILES {
            match read_json_capped::<Value>(&dir.join(file), MAX_FILE_BYTES) {
                Ok(document) => match token_of(&document) {
                    Some(token) => return Ok(token),
                    None => seen = seen.max_of(Reason::NoToken),
                },
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => seen = seen.max_of(Reason::Unreadable),
            }
        }
    }
    Err(seen)
}

impl Reason {
    /// The more informative of two reasons: unreadable beats no token beats not found.
    fn max_of(self, other: Self) -> Self {
        let rank = |reason: Self| match reason {
            Self::NotFound => 0,
            Self::NoToken => 1,
            Self::Unreadable => 2,
        };
        if rank(other) > rank(self) { other } else { self }
    }
}

/// The `oauth_token` of the first entry keyed `github.com` or `github.com:<app id>`. Enterprise
/// hosts are skipped on purpose: the token is only ever sent to api.github.com.
fn token_of(document: &Value) -> Option<Secret> {
    document
        .as_object()?
        .iter()
        .filter(|(key, _)| *key == "github.com" || key.starts_with("github.com:"))
        .find_map(|(_, entry)| {
            let token = entry.get("oauth_token")?.as_str()?.trim();
            usable(token).then(|| Secret::new(token))
        })
}

/// Printable ASCII without spaces and not absurdly long: nothing that could split a header.
fn usable(token: &str) -> bool {
    !token.is_empty() && token.len() <= MAX_TOKEN_LEN && token.bytes().all(|b| b.is_ascii_graphic())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directories_follow_the_platform_and_start_at_the_injected_home() {
        let home = Path::new("home");
        assert_eq!(
            candidate_dirs(home, true),
            [home.join("AppData").join("Local").join("github-copilot")]
        );
        assert_eq!(candidate_dirs(home, false), [home.join(".config").join("github-copilot")]);
    }

    #[test]
    fn only_github_com_keys_count_and_tokens_must_be_header_safe() {
        let doc = |text: &str| serde_json::from_str::<Value>(text).unwrap();
        let token = |text: &str| token_of(&doc(text)).map(|t| t.expose().to_owned());
        assert_eq!(
            token(r#"{"github.com:Iv1.x":{"oauth_token":"gho_a"}}"#).as_deref(),
            Some("gho_a")
        );
        assert_eq!(token(r#"{"github.com":{"oauth_token":" gho_b\n"}}"#).as_deref(), Some("gho_b"));
        for hostile in [
            r#"{"ghe.example.com":{"oauth_token":"gho_c"}}"#,
            r#"{"github.com.evil.test":{"oauth_token":"gho_c"}}"#,
            r#"{"github.com":{"oauth_token":"has space"}}"#,
            r#"{"github.com":{"oauth_token":"a\r\nX: y"}}"#,
            r#"{"github.com":{"oauth_token":""}}"#,
            r#"{"github.com":{"oauth_token":7}}"#,
            r#"{"github.com":"gho_d"}"#,
            r#"[{"github.com":{"oauth_token":"gho_e"}}]"#,
        ] {
            assert_eq!(token(hostile), None, "{hostile}");
        }
        let long = format!(r#"{{"github.com":{{"oauth_token":"{}"}}}}"#, "a".repeat(600));
        assert_eq!(token(&long), None);
    }
}
