//! Parsing of one request head. Strict on purpose: anything ambiguous is a 400, because an
//! ambiguity between this parser and a browser's is where request smuggling lives.

/// The head (request line, headers and the blank line) may not be longer than this.
pub(super) const MAX_HEAD_BYTES: usize = 8 * 1024;
/// More header lines than this is refused; a real client sends a handful.
pub(super) const MAX_HEADERS: usize = 64;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Request {
    pub method: String,
    /// The target without its query string.
    pub path: String,
    pub host: Option<String>,
    pub has_origin: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Malformed {
    /// 400
    BadRequest,
    /// 431
    TooLarge,
}

/// `head` ends with the blank line. Bytes of a body are never read: a request that announces one
/// is refused.
pub(super) fn parse(head: &[u8]) -> Result<Request, Malformed> {
    if head.len() > MAX_HEAD_BYTES {
        return Err(Malformed::TooLarge);
    }
    let text = std::str::from_utf8(head).map_err(|_| Malformed::BadRequest)?;
    let text = text.strip_suffix("\r\n\r\n").ok_or(Malformed::BadRequest)?;
    // The CRLFs are split off first: any CR or LF left inside a line is a bare one.
    if !text.is_ascii() || text.split("\r\n").any(|line| line.bytes().any(is_forbidden_byte)) {
        return Err(Malformed::BadRequest);
    }
    let mut lines = text.split("\r\n");
    let (method, path) = request_line(lines.next().unwrap_or_default())?;
    let mut request = Request { method, path, host: None, has_origin: false };
    for (count, line) in lines.enumerate() {
        if count >= MAX_HEADERS {
            return Err(Malformed::TooLarge);
        }
        header(&mut request, line)?;
    }
    Ok(request)
}

/// Controls other than tab, which includes a bare CR or LF.
fn is_forbidden_byte(byte: u8) -> bool {
    (byte < 0x20 && byte != b'\t') || byte == 0x7f
}

fn request_line(line: &str) -> Result<(String, String), Malformed> {
    let mut parts = line.split(' ');
    let (Some(method), Some(target), Some(version), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(Malformed::BadRequest);
    };
    if !is_token(method) || !matches!(version, "HTTP/1.1" | "HTTP/1.0") {
        return Err(Malformed::BadRequest);
    }
    // Origin-form only: no absolute URL, no `*`, no authority.
    if !target.starts_with('/') || target.starts_with("//") {
        return Err(Malformed::BadRequest);
    }
    let path = target.split(['?', '#']).next().unwrap_or_default();
    Ok((method.to_owned(), path.to_owned()))
}

fn header(request: &mut Request, line: &str) -> Result<(), Malformed> {
    // A line that starts with whitespace is an obsolete fold; a name with a space before the
    // colon is a classic smuggling shape. Both fail `is_token`.
    let (name, value) = line.split_once(':').ok_or(Malformed::BadRequest)?;
    if !is_token(name) {
        return Err(Malformed::BadRequest);
    }
    let value = value.trim_matches([' ', '\t']);
    match name.to_ascii_lowercase().as_str() {
        "host" => {
            if request.host.replace(value.to_owned()).is_some() {
                return Err(Malformed::BadRequest);
            }
        }
        "origin" => request.has_origin = true,
        "content-length" if value != "0" => return Err(Malformed::BadRequest),
        "transfer-encoding" => return Err(Malformed::BadRequest),
        _ => {}
    }
    Ok(())
}

fn is_token(text: &str) -> bool {
    !text.is_empty()
        && text.bytes().all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b))
}
