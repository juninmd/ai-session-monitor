//! What the language server's command line says about itself: the CSRF tokens, the ports it was
//! told to use and which app started it. Works on an already split argument list (the OS rules for
//! quoting have been applied by then), so a value with spaces or quotes arrives as one element.

use std::path::Path;

use usage_support::Secret;

/// Longest token accepted; a UUID is 36 bytes, anything near this is not a token.
const MAX_TOKEN_BYTES: usize = 256;
/// The server takes about 15 flags; more than this is not the process we are looking for.
const MAX_ARGS: usize = 128;
/// The flags we read, spelled with `_`; builds differ in `-` versus `_`, so both match.
const NAMES: [&str; 6] = [
    "csrf_token",
    "extension_server_csrf_token",
    "https_server_port",
    "lsp_port",
    "extension_server_port",
    "app_data_dir",
];

/// Values of the flags we use. Tokens are kept as `Secret`, so `Debug` never prints them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ServerFlags {
    pub csrf_token: Option<Secret>,
    /// Windows builds also hand the IDE's extension server its own token.
    pub extension_csrf_token: Option<Secret>,
    /// Port the HTTPS endpoint was asked to use; `0` (pick any) is not recorded.
    pub https_port: Option<u16>,
    /// A JSON-RPC-over-TCP port, not HTTP: never probed.
    pub lsp_port: Option<u16>,
    pub extension_port: Option<u16>,
    pub app_data_dir: Option<String>,
}

impl ServerFlags {
    /// Go's `flag` package takes one or two dashes and `=` or a space between name and value; the
    /// last occurrence wins. A value that starts with a dash is the next flag, not a value.
    pub(crate) fn parse(args: &[String]) -> Self {
        let args = &args[..args.len().min(MAX_ARGS)];
        let mut flags = Self::default();
        let mut index = 0;
        while index < args.len() {
            let arg = &args[index];
            index += 1;
            let Some(body) = arg.strip_prefix("--").or_else(|| arg.strip_prefix('-')) else {
                continue;
            };
            let (name, inline) = match body.split_once('=') {
                Some((name, value)) => (name, Some(value)),
                None => (body, None),
            };
            let name = name.to_ascii_lowercase().replace('-', "_");
            if !NAMES.contains(&name.as_str()) {
                continue;
            }
            let value = match inline {
                Some(value) => value,
                None => match args.get(index).filter(|next| !next.starts_with('-')) {
                    Some(next) => {
                        index += 1;
                        next.as_str()
                    }
                    None => continue,
                },
            };
            flags.set(&name, value);
        }
        flags
    }

    fn set(&mut self, name: &str, value: &str) {
        if value.is_empty() {
            return;
        }
        match name {
            "csrf_token" => self.csrf_token = Some(Secret::new(value)),
            "extension_server_csrf_token" => self.extension_csrf_token = Some(Secret::new(value)),
            "https_server_port" => self.https_port = port(value).or(self.https_port),
            "lsp_port" => self.lsp_port = port(value).or(self.lsp_port),
            "extension_server_port" => self.extension_port = port(value).or(self.extension_port),
            "app_data_dir" => self.app_data_dir = Some(value.to_owned()),
            _ => {}
        }
    }

    pub(crate) fn has_usable_token(&self) -> bool {
        usable(self.csrf_token.as_ref()).is_some()
            || usable(self.extension_csrf_token.as_ref()).is_some()
    }

    /// The app is Antigravity, not another Codeium fork that ships the same server binary. An
    /// explicit `--app_data_dir` decides; without one the install path has to say so.
    pub(crate) fn is_antigravity(&self, exe: Option<&Path>, args: &[String]) -> bool {
        if let Some(dir) = &self.app_data_dir {
            return dir.to_ascii_lowercase().starts_with("antigravity");
        }
        let in_path = |path: &Path| {
            path.components().any(|part| is_antigravity_dir(&part.as_os_str().to_string_lossy()))
        };
        exe.is_some_and(in_path) || args.first().is_some_and(|first| in_path(Path::new(first)))
    }
}

/// A folder named like the app, not any folder that mentions it (a user may be called that).
fn is_antigravity_dir(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let bare = lower.strip_suffix(".app").unwrap_or(&lower);
    matches!(bare, "antigravity" | "antigravity ide" | "antigravity-ide" | "antigravity_ide")
}

fn port(value: &str) -> Option<u16> {
    value.trim().parse::<u16>().ok().filter(|port| *port != 0)
}

/// The token, but only when it can travel as an HTTP header value: visible ASCII, bounded, no
/// spaces. Anything else counts as no token at all.
pub(crate) fn usable(token: Option<&Secret>) -> Option<Secret> {
    let token = token?;
    let text = token.expose();
    let fits = !text.is_empty() && text.len() <= MAX_TOKEN_BYTES;
    (fits && text.bytes().all(|byte| byte.is_ascii_graphic())).then(|| token.clone())
}

#[cfg(test)]
mod tests;
