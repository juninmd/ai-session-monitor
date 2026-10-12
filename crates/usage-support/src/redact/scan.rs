//! Byte-level helpers for the redaction rules. Delimiters are ASCII, so every cut lands on a char
//! boundary even in text full of multi-byte characters.

/// Half-open byte range of `input` to replace with the mask.
pub(super) type Span = (usize, usize);

pub(super) fn is_blank(b: u8) -> bool {
    b == b' ' || b == b'\t'
}

/// Characters of an identifier such as `api_key`, `X-Csrf-Token` or `accessToken`.
pub(super) fn is_word(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

/// Characters of base64, base64url, hex and JWT-like credentials.
pub(super) fn is_token(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'~' | b'+' | b'/' | b'=')
}

/// Characters that can follow a credential prefix such as `sk-`.
pub(super) fn is_key_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.')
}

pub(super) fn skip_blanks(b: &[u8], mut i: usize) -> usize {
    while b.get(i).is_some_and(|&c| is_blank(c)) {
        i += 1;
    }
    i
}

pub(super) fn token_end(b: &[u8], start: usize) -> usize {
    start + b.get(start..).map_or(0, |rest| rest.iter().take_while(|&&c| is_token(c)).count())
}

/// Indexes where `needle` occurs, ignoring ASCII case. `needle` must not be empty.
pub(super) fn find_ci<'a>(hay: &'a [u8], needle: &'a [u8]) -> impl Iterator<Item = usize> + 'a {
    hay.windows(needle.len())
        .enumerate()
        .filter(move |(_, w)| w.eq_ignore_ascii_case(needle))
        .map(|(i, _)| i)
}

/// Indexes where `needle` occurs byte for byte. `needle` must not be empty.
pub(super) fn find_exact<'a>(hay: &'a [u8], needle: &'a [u8]) -> impl Iterator<Item = usize> + 'a {
    hay.windows(needle.len()).enumerate().filter(move |(_, w)| *w == needle).map(|(i, _)| i)
}

/// True when the char before `at` is a letter or digit, so a match at `at` is inside a longer word.
pub(super) fn alnum_before(input: &str, at: usize) -> bool {
    input.get(..at).is_none_or(|head| head.chars().next_back().is_some_and(char::is_alphanumeric))
}

/// How a value is wrapped: bare, in quotes, or in JSON-escaped quotes (`\"..\"`).
#[derive(Clone, Copy)]
pub(super) enum Opener {
    Plain,
    Quote(u8),
    Escaped(u8),
}

fn skip_quote(b: &[u8], i: usize) -> usize {
    match (b.get(i), b.get(i + 1)) {
        (Some(b'\\'), Some(b'"' | b'\'')) => i + 2,
        (Some(b'"' | b'\''), _) => i + 1,
        _ => i,
    }
}

/// Where a value starts and how it opens, after an optional closing quote of the key, blanks and
/// one `:` or `=`. `None` when the key is not followed by an assignment.
pub(super) fn value_start(b: &[u8], key_end: usize) -> Option<(usize, Opener)> {
    let i = skip_blanks(b, skip_quote(b, key_end));
    if !matches!(b.get(i), Some(b':' | b'=')) {
        return None;
    }
    Some(open(b, skip_blanks(b, i + 1)))
}

pub(super) fn open(b: &[u8], i: usize) -> (usize, Opener) {
    match (b.get(i), b.get(i + 1)) {
        (Some(b'\\'), Some(&q @ (b'"' | b'\''))) => (i + 2, Opener::Escaped(q)),
        (Some(&q @ (b'"' | b'\'')), _) => (i + 1, Opener::Quote(q)),
        _ => (i, Opener::Plain),
    }
}

/// End of a quoted value: the closing quote, or the end of the line when it is unbalanced.
pub(super) fn quoted_end(b: &[u8], start: usize, quote: u8) -> usize {
    let mut i = start;
    while i < b.len() {
        match b[i] {
            b'\n' | b'\r' => break,
            // A dangling backslash is part of an unbalanced value, so it is masked with it.
            b'\\' if matches!(b.get(i + 1), Some(b'\n' | b'\r') | None) => {
                i += 1;
                break;
            }
            b'\\' => i += 2,
            c if c == quote => break,
            _ => i += 1,
        }
    }
    i.min(b.len())
}

/// End of a value opened with `\"`: the matching `\"`, or the end of the line.
pub(super) fn escaped_end(b: &[u8], start: usize, quote: u8) -> usize {
    let mut i = start;
    while i < b.len() {
        match b[i] {
            b'\n' | b'\r' => break,
            b'\\' if b.get(i + 1) == Some(&quote) => break,
            _ => i += 1,
        }
    }
    i
}

/// End of an unquoted header value: the line, or a quote that closes it (`-H 'Authorization: x'`).
pub(super) fn header_plain_end(b: &[u8], start: usize) -> usize {
    let tail = b.get(start..).unwrap_or_default();
    start + tail.iter().take_while(|&&c| !matches!(c, b'\n' | b'\r' | b'"' | b'\'' | b'`')).count()
}

fn is_plain_stop(c: u8) -> bool {
    c < 0x20
        || c == 0x7f
        || c == b' '
        || matches!(
            c,
            b'"' | b'\''
                | b'`'
                | b'&'
                | b';'
                | b','
                | b')'
                | b']'
                | b'}'
                | b'<'
                | b'>'
                | b'\\'
                | b'#'
        )
}

/// End of an unquoted value: the first delimiter, control character or Unicode whitespace.
pub(super) fn plain_end(input: &str, start: usize) -> usize {
    let b = input.as_bytes();
    let mut i = start;
    while i < b.len() {
        if b[i] < 0x80 {
            if is_plain_stop(b[i]) {
                break;
            }
            i += 1;
            continue;
        }
        match input.get(i..).and_then(|rest| rest.chars().next()) {
            Some(ch) if !ch.is_whitespace() => i += ch.len_utf8(),
            _ => break,
        }
    }
    i
}
