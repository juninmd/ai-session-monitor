//! The session cookie Cursor's web API expects, built from the access token in `state.vscdb`.
//!
//! The token is a JWT. Only its payload is decoded, to read `sub` (the user id) and `exp`; the
//! signature is never checked (the server does that), and the claims never leave this module
//! except as the cookie. Everything here is defensive: the file is user-writable, so a token
//! that is not plainly a JWT is "no token", never a crash and never part of a header.

use usage_support::Secret;

/// The cookie the web app sends: `WorkosCursorSessionToken=<userId>%3A%3A<jwt>`.
const COOKIE_NAME: &str = "WorkosCursorSessionToken";
/// A token that expires sooner than this is not used: it could lapse mid-request.
pub(crate) const MIN_REMAINING_SECS: i64 = 60;
const MAX_TOKEN_BYTES: usize = 8 * 1024;
const MAX_PAYLOAD_BYTES: usize = 4 * 1024;
const MAX_USER_ID_BYTES: usize = 128;

/// Why a stored token cannot be used. Both mean "sign in to Cursor again".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Unusable {
    /// Empty, not three dot-separated segments, characters outside the JWT alphabet, no `sub`.
    Malformed,
    /// `exp` is less than [`MIN_REMAINING_SECS`] away.
    Expired,
}

/// The cookie header value for `token`, or why the token cannot be used at `now_secs`.
pub(crate) fn cookie(token: &str, now_secs: i64) -> Result<Secret, Unusable> {
    let token = token.trim();
    if token.is_empty() || token.len() > MAX_TOKEN_BYTES || !jwt_shaped(token) {
        return Err(Unusable::Malformed);
    }
    let payload = token.split('.').nth(1).ok_or(Unusable::Malformed)?;
    let claims = decode(payload).ok_or(Unusable::Malformed)?;
    let claims: serde_json::Value =
        serde_json::from_slice(&claims).map_err(|_| Unusable::Malformed)?;
    let user_id = claims.get("sub").and_then(|sub| sub.as_str()).and_then(user_id_of);
    let user_id = user_id.ok_or(Unusable::Malformed)?;
    if let Some(exp) = claims.get("exp").and_then(serde_json::Value::as_f64)
        && exp < now_secs as f64 + MIN_REMAINING_SECS as f64
    {
        return Err(Unusable::Expired);
    }
    Ok(Secret::new(format!("{COOKIE_NAME}={user_id}%3A%3A{token}")))
}

/// Three non-empty segments from the base64url alphabet: also what keeps a header-injection
/// payload (whitespace, `;`, CR/LF) out of the cookie.
fn jwt_shaped(token: &str) -> bool {
    let segments: Vec<&str> = token.split('.').collect();
    segments.len() == 3
        && segments.iter().all(|segment| {
            !segment.is_empty()
                && segment.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        })
}

/// `auth0|user_01ABC` becomes `user_01ABC`: whatever follows the last `|`, if it is a plain id.
fn user_id_of(sub: &str) -> Option<&str> {
    let id = sub.rsplit('|').next()?;
    let plain = !id.is_empty()
        && id.len() <= MAX_USER_ID_BYTES
        && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
    plain.then_some(id)
}

/// Base64url without padding (the JWT form), decoded by hand: 4 characters make 3 bytes. A
/// character outside the alphabet, or a length no encoder produces, is `None`.
pub(crate) fn decode(segment: &str) -> Option<Vec<u8>> {
    let text = segment.trim_end_matches('=');
    if text.len() > MAX_PAYLOAD_BYTES * 4 / 3 + 4 || text.len() % 4 == 1 {
        return None;
    }
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    let (mut buffer, mut bits) = (0u32, 0u32);
    for byte in text.bytes() {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            _ => return None,
        };
        buffer = (buffer << 6) | u32::from(value);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(u8::try_from((buffer >> bits) & 0xff).ok()?);
        }
    }
    Some(out)
}
