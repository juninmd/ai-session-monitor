//! Masks credentials in free text before it reaches a log line, an error param or a notice.
//!
//! The rules are heuristic and deliberately over-mask: a false positive costs a word in a message,
//! a miss costs a key. They cover `Authorization`/cookie header values, `Bearer` tokens, `sk-`,
//! `oc_sk_`, Google, GitHub and JWT tokens, URL userinfo, and the value of anything assigned to a
//! key-like name (`api_key=`, `"token":`, `--csrf-token`, `?password=`).

mod rules;
mod scan;

use scan::Span;

const MASK: &str = "***";
/// Passes after the first; a mask next to leftover text can fold that text in on the next pass.
const SETTLE_PASSES: usize = 3;

/// Returns `input` with every credential-looking substring replaced by `***`. Scrubbing already
/// scrubbed text changes nothing, the cost is linear in the input size, and no input panics.
pub fn scrub(input: &str) -> String {
    let Some(mut current) = scrub_once(input) else { return input.to_owned() };
    for _ in 0..SETTLE_PASSES {
        match scrub_once(&current) {
            Some(next) if next != current => current = next,
            _ => break,
        }
    }
    current
}

/// One pass of every rule; `None` when nothing matched.
fn scrub_once(input: &str) -> Option<String> {
    let mut spans = Vec::new();
    rules::userinfo(input, &mut spans);
    rules::header_values(input, &mut spans);
    rules::bearer(input, &mut spans);
    rules::assignments(input, &mut spans);
    rules::prefixed_tokens(input, &mut spans);
    apply(input, spans)
}

fn apply(input: &str, mut spans: Vec<Span>) -> Option<String> {
    spans.retain(|&(start, end)| {
        start < end && input.is_char_boundary(start) && input.is_char_boundary(end)
    });
    if spans.is_empty() {
        return None;
    }
    spans.sort_unstable();
    let mut merged: Vec<Span> = Vec::with_capacity(spans.len());
    for (start, end) in spans {
        match merged.last_mut() {
            Some(last) if start <= last.1 => last.1 = last.1.max(end),
            _ => merged.push((start, end)),
        }
    }
    let mut out = String::with_capacity(input.len());
    let mut cursor = 0;
    for (start, end) in merged {
        out.push_str(&input[cursor..start]);
        out.push_str(MASK);
        cursor = end;
    }
    out.push_str(&input[cursor..]);
    Some(out)
}
