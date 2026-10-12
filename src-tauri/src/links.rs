//! The only destinations the webview can ask the shell to open. It names a link, never a URL, so
//! a compromised page cannot make the app launch anything else.

use std::io;

use usage_model::{ErrorCode, ExternalLink, HarnessError};

#[cfg(test)]
mod tests;

/// Kimi, z.ai, OpenRouter and DeepSeek land on their key pages; both need a sign-in the vendor's site handles.
///
/// The OpenCode console has no deep link without the user's workspace id, so both console links
/// land on the sign-in page, which forwards to the workspace.
pub const fn url_for(link: ExternalLink) -> &'static str {
    match link {
        ExternalLink::OpencodeGoConsole | ExternalLink::OpencodeBilling => {
            "https://opencode.ai/auth"
        }
        ExternalLink::KimiConsole => "https://www.kimi.com/code/console",
        ExternalLink::ZaiConsole => "https://z.ai/manage-apikey/apikey-list",
        ExternalLink::OpenrouterConsole => "https://openrouter.ai/settings/keys",
        ExternalLink::DeepseekConsole => "https://platform.deepseek.com/api_keys",
        ExternalLink::ProjectReadme => "https://github.com/juninmd/ai-session-monitor#readme",
    }
}

/// Opens `link` through `launch`, which receives the fixed URL.
pub fn open(
    link: ExternalLink,
    launch: &dyn Fn(&str) -> io::Result<()>,
) -> Result<(), HarnessError> {
    launch(url_for(link)).map_err(|_| {
        HarnessError::new(ErrorCode::SpawnFailed, false).with_param("target", "browser")
    })
}

/// Hands the URL to the Windows shell, which opens the default browser. The program is addressed
/// by absolute path so nothing in the app folder or the working directory can stand in for it.
#[cfg(windows)]
pub fn launch_default_browser(url: &str) -> io::Result<()> {
    let root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
    let explorer = std::path::Path::new(&root).join("explorer.exe");
    std::process::Command::new(explorer).arg(url).spawn().map(|_| ())
}

#[cfg(not(windows))]
pub fn launch_default_browser(_url: &str) -> io::Result<()> {
    Err(io::Error::from(io::ErrorKind::Unsupported))
}
