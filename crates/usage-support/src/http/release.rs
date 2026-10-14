//! Public GitHub release downloads. No credentials, bounded redirects, body and deadline.

use reqwest::Url;
use reqwest::header::{ACCEPT, LOCATION};
use usage_model::{ErrorCode, HarnessError};

use super::SafeClient;
use super::classify::{failure, from_status, from_transport};

pub const MAX_RELEASE_BYTES: usize = 128 << 20;
const REPOSITORY: &str = "/juninmd/ai-session-monitor/releases/";

fn refused() -> HarnessError {
    HarnessError::new(ErrorCode::InvalidUrl, false)
}

fn secure(url: &Url) -> bool {
    url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && url.fragment().is_none()
}

/// An initial URL must name this repository, never an arbitrary host or signed CDN query.
pub fn release_url(value: &str) -> Result<Url, HarnessError> {
    let url = Url::parse(value).map_err(|_| refused())?;
    let github = url.host_str() == Some("github.com")
        && url.path().starts_with(REPOSITORY)
        && (url.path().starts_with(&format!("{REPOSITORY}download/"))
            || url.path() == format!("{REPOSITORY}latest/download/latest.json"));
    let api = url.host_str() == Some("api.github.com")
        && url
            .path()
            .strip_prefix("/repos/juninmd/ai-session-monitor/releases/assets/")
            .is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()));
    if !secure(&url) || url.query().is_some() || !(github || api) {
        return Err(refused());
    }
    Ok(url)
}

fn redirect_allowed(url: &Url) -> bool {
    secure(url)
        && match url.host_str() {
            Some("github.com" | "api.github.com") => release_url(url.as_str()).is_ok(),
            Some("release-assets.githubusercontent.com" | "objects.githubusercontent.com") => true,
            _ => false,
        }
}

impl SafeClient {
    /// Downloads a public release with OS-verified TLS. Collector JSON requests still refuse
    /// all redirects. Progress is cumulative bytes; a missing content length stays unknown.
    pub async fn get_release_bytes(
        &self,
        value: &str,
        limit: usize,
        progress: impl FnMut(u64, Option<u64>) + Send,
    ) -> Result<Vec<u8>, HarnessError> {
        let url = release_url(value)?;
        if !self.release_tls || limit == 0 || limit > MAX_RELEASE_BYTES {
            return Err(refused());
        }
        self.release_download(url, limit, redirect_allowed, progress).await
    }

    async fn release_download(
        &self,
        mut url: Url,
        limit: usize,
        allowed: impl Fn(&Url) -> bool + Send,
        mut progress: impl FnMut(u64, Option<u64>) + Send,
    ) -> Result<Vec<u8>, HarnessError> {
        let download = async {
            for redirects in 0..=5 {
                let host = url.host_str().unwrap_or_default().to_owned();
                // No RequestSpec or caller headers: credentials cannot cross a redirect.
                let mut response = self
                    .inner
                    .get(url.clone())
                    .header(ACCEPT, "application/octet-stream")
                    .send()
                    .await
                    .map_err(|err| from_transport(&err, &host))?;
                if response.status().is_redirection() {
                    let next = response
                        .headers()
                        .get(LOCATION)
                        .and_then(|h| h.to_str().ok())
                        .and_then(|location| url.join(location).ok())
                        .ok_or_else(refused)?;
                    if redirects == 5 || !allowed(&next) {
                        return Err(refused());
                    }
                    url = next;
                    continue;
                }
                if let Some(error) = from_status(response.status(), response.headers(), &host) {
                    return Err(error);
                }
                // Partial content is never a complete installer.
                if response.status().as_u16() != 200 {
                    return Err(failure(ErrorCode::BadResponse, false, &host));
                }
                let total = response.content_length();
                if total.is_some_and(|bytes| bytes > limit as u64) {
                    return Err(failure(ErrorCode::ResponseTooLarge, false, &host));
                }
                let mut bytes = Vec::new();
                progress(0, total);
                while let Some(chunk) =
                    response.chunk().await.map_err(|err| from_transport(&err, &host))?
                {
                    if chunk.len() > limit.saturating_sub(bytes.len()) {
                        return Err(failure(ErrorCode::ResponseTooLarge, false, &host));
                    }
                    bytes.extend_from_slice(&chunk);
                    progress(bytes.len() as u64, total);
                }
                if bytes.is_empty() || total.is_some_and(|length| length != bytes.len() as u64) {
                    return Err(failure(ErrorCode::BadResponse, false, &host));
                }
                return Ok(bytes);
            }
            Err(refused())
        };
        tokio::time::timeout(self.timeout, download)
            .await
            .map_err(|_| HarnessError::new(ErrorCode::Timeout, true))?
    }
}

#[cfg(test)]
mod tests;
