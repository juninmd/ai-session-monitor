//! The individual redaction rules. Each one only records spans; `scrub` merges and applies them,
//! so rules never see each other's output and their order does not matter. Every rule skips past
//! what it already masked, which keeps hostile input (`key=key=key=...`) linear.

use super::scan::{
    Opener, Span, alnum_before, escaped_end, find_ci, find_exact, header_plain_end, is_blank,
    is_key_char, is_word, open, plain_end, quoted_end, skip_blanks, token_end, value_start,
};

const HEADER_NAMES: [&[u8]; 4] =
    [b"authorization", b"proxy-authorization", b"set-cookie", b"cookie"];

/// A word that ends with one of these names a secret (`api_key`, `accessToken`, `CLIENT_SECRET`).
const SECRET_WORDS: [&[u8]; 9] = [
    b"key",
    b"token",
    b"secret",
    b"password",
    b"passwd",
    b"csrf",
    b"credential",
    b"credentials",
    b"apikey",
];

/// Well-known credential prefixes with the minimum run length that must follow them.
const PREFIXES: [(&[u8], usize); 10] = [
    (b"sk-", 6),
    (b"oc_sk_", 4),
    (b"AIza", 30),
    (b"ya29.", 20),
    (b"ghp_", 20),
    (b"gho_", 20),
    (b"ghu_", 20),
    (b"ghs_", 20),
    (b"ghr_", 20),
    (b"github_pat_", 20),
];

/// `scheme://user:password@host` -> `scheme://***@host`.
pub(super) fn userinfo(input: &str, spans: &mut Vec<Span>) {
    let b = input.as_bytes();
    for at in find_exact(b, b"://") {
        if at == 0 || !(b[at - 1].is_ascii_alphanumeric() || b"+.-".contains(&b[at - 1])) {
            continue;
        }
        let start = at + 3;
        let len = b[start..].iter().take_while(|&&c| !ends_authority(c)).count();
        // The last `@` separates userinfo from host, even when a password contains a raw `@`.
        if let Some(rel) = b[start..start + len].iter().rposition(|&c| c == b'@')
            && rel > 0
        {
            spans.push((start, start + rel));
        }
    }
}

fn ends_authority(c: u8) -> bool {
    c <= b' ' || matches!(c, b'/' | b'?' | b'#' | b'\\' | b'"' | b'\'' | b'<' | b'>' | b'`')
}

/// `Authorization: <anything>` and the other credential-carrying headers: the whole value.
pub(super) fn header_values(input: &str, spans: &mut Vec<Span>) {
    let b = input.as_bytes();
    for name in HEADER_NAMES {
        let mut skip_to = 0;
        for at in find_ci(b, name) {
            if at < skip_to || (at > 0 && is_word(b[at - 1])) {
                continue;
            }
            let Some((start, opener)) = value_start(b, at + name.len()) else { continue };
            let end = match opener {
                Opener::Quote(q) => quoted_end(b, start, q),
                Opener::Escaped(q) => escaped_end(b, start, q),
                Opener::Plain => header_plain_end(b, start),
            };
            if end > start {
                spans.push((start, end));
                skip_to = end;
            }
        }
    }
}

/// `Bearer <token>` anywhere in the text.
pub(super) fn bearer(input: &str, spans: &mut Vec<Span>) {
    let b = input.as_bytes();
    for at in find_ci(b, b"bearer") {
        let after = at + "bearer".len();
        if alnum_before(input, at) || !b.get(after).is_some_and(|&c| is_blank(c)) {
            continue;
        }
        let start = skip_blanks(b, after);
        let end = token_end(b, start);
        if looks_like_credential(&b[start..end]) {
            spans.push((start, end));
        }
    }
}

/// Short plain words ("Bearer token missing") are prose; anything token-shaped is masked.
fn looks_like_credential(run: &[u8]) -> bool {
    run.len() >= 8 || run.iter().any(|c| !c.is_ascii_alphabetic())
}

/// `key=...`, `"api_key": "..."`, `?token=...`, `--csrf-token ...` and `key <long opaque run>`.
pub(super) fn assignments(input: &str, spans: &mut Vec<Span>) {
    let b = input.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if !is_word(b[i]) {
            i += 1;
            continue;
        }
        let start = i;
        while i < b.len() && is_word(b[i]) {
            i += 1;
        }
        if names_a_secret(&b[start..i])
            && let Some(end) = assigned_value(input, start, i, spans)
        {
            i = i.max(end);
        }
    }
}

/// True when the word ends with a secret name that is a word of its own: `api_key`, `apiKey`,
/// `CSRF-TOKEN`, but not `monkey`, `tokens` or `keyboard`.
fn names_a_secret(word: &[u8]) -> bool {
    SECRET_WORDS.iter().any(|name| {
        let Some(split) = word.len().checked_sub(name.len()) else { return false };
        if !word[split..].eq_ignore_ascii_case(name) {
            return false;
        }
        if split == 0 {
            return true;
        }
        let (prev, first) = (word[split - 1], word[split]);
        prev == b'_'
            || prev == b'-'
            || (first.is_ascii_uppercase() && (prev.is_ascii_lowercase() || prev.is_ascii_digit()))
            || !word.iter().any(u8::is_ascii_lowercase)
    })
}

/// Records the span of the value assigned to the secret-named word and returns where it ends.
fn assigned_value(
    input: &str,
    word_start: usize,
    word_end: usize,
    spans: &mut Vec<Span>,
) -> Option<usize> {
    let b = input.as_bytes();
    if let Some((start, opener)) = value_start(b, word_end) {
        return push_value(input, start, opener, spans);
    }
    // No `=` or `:`. A CLI flag takes the next argument; prose only counts when it looks like a key.
    let next = skip_blanks(b, word_end);
    if next == word_end {
        return None;
    }
    let (start, opener) = open(b, next);
    if b[word_start] == b'-' {
        return push_value(input, start, opener, spans);
    }
    let end = token_end(b, start);
    if looks_opaque(&b[start..end]) {
        spans.push((start, end));
        return Some(end);
    }
    None
}

fn looks_opaque(run: &[u8]) -> bool {
    run.len() >= 32 || (run.len() >= 16 && run.iter().any(u8::is_ascii_digit))
}

fn push_value(input: &str, start: usize, opener: Opener, spans: &mut Vec<Span>) -> Option<usize> {
    let b = input.as_bytes();
    let end = match opener {
        Opener::Quote(q) => quoted_end(b, start, q),
        Opener::Escaped(q) => escaped_end(b, start, q),
        // A JSON object or list after the key is structure, not a value to mask.
        Opener::Plain if matches!(b.get(start), Some(b'{' | b'[' | b'(')) => return None,
        Opener::Plain => plain_end(input, start),
    };
    if end > start {
        spans.push((start, end));
        return Some(end);
    }
    None
}

/// `sk-...`, `oc_sk_...`, Google and GitHub keys, and JWTs, wherever they appear.
pub(super) fn prefixed_tokens(input: &str, spans: &mut Vec<Span>) {
    let b = input.as_bytes();
    for (prefix, min_run) in PREFIXES {
        let mut skip_to = 0;
        for at in find_exact(b, prefix) {
            if at < skip_to || alnum_before(input, at) {
                continue;
            }
            let from = at + prefix.len();
            let run = b[from..].iter().take_while(|&&c| is_key_char(c)).count();
            if run >= min_run {
                spans.push((at, from + run));
                skip_to = from + run;
            }
        }
    }
    jwts(input, spans);
}

fn is_base64url(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'-'
}

fn base64url_end(b: &[u8], start: usize) -> usize {
    start + b.get(start..).map_or(0, |rest| rest.iter().take_while(|&&c| is_base64url(c)).count())
}

/// `header.payload.signature`, three base64url segments; the header always starts with `eyJ`.
fn jwts(input: &str, spans: &mut Vec<Span>) {
    let b = input.as_bytes();
    let mut skip_to = 0;
    for at in find_exact(b, b"eyJ") {
        if at < skip_to || alnum_before(input, at) {
            continue;
        }
        let header_end = base64url_end(b, at);
        skip_to = header_end;
        if header_end - at < 8 || b.get(header_end) != Some(&b'.') {
            continue;
        }
        let payload_end = base64url_end(b, header_end + 1);
        if payload_end - header_end - 1 < 4 || b.get(payload_end) != Some(&b'.') {
            continue;
        }
        let end = base64url_end(b, payload_end + 1);
        spans.push((at, end));
        skip_to = end;
    }
}
