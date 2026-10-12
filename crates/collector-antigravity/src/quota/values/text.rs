//! Provider strings are display text from another program: read a bounded part, never trust it.

/// Longest prefix of a string that is looked at.
const MAX_CHARS: usize = 256;
const MAX_TOKENS: usize = 64;
/// A run of digits longer than this is an identifier, not a count.
const MAX_DIGITS: usize = 9;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Token {
    /// Lowercase letters and digits, `gemini` or `5h`.
    Word(String),
    Number(u64),
}

impl Token {
    pub(crate) fn word(self) -> Option<String> {
        match self {
            Self::Word(word) => Some(word),
            Self::Number(_) => None,
        }
    }
}

/// Alphanumeric runs of the text, lowercased: `"Gemini-3 Pro (Low)"` is `gemini`, `3`, `pro`, `low`.
pub(crate) fn tokens(text: &str) -> Vec<Token> {
    let mut found = Vec::new();
    let mut run = String::new();
    // The trailing space closes the last run.
    for ch in text.chars().take(MAX_CHARS).chain(std::iter::once(' ')) {
        if ch.is_alphanumeric() {
            run.extend(ch.to_lowercase());
        } else if !run.is_empty() {
            found.push(token_of(std::mem::take(&mut run)));
            if found.len() >= MAX_TOKENS {
                break;
            }
        }
    }
    found
}

fn token_of(run: String) -> Token {
    let digits = run.len() <= MAX_DIGITS && run.bytes().all(|byte| byte.is_ascii_digit());
    match run.parse() {
        Ok(number) if digits => Token::Number(number),
        _ => Token::Word(run),
    }
}

/// Characters that draw nothing or reorder the text around them.
fn is_invisible(ch: char) -> bool {
    ch.is_control()
        || matches!(ch, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}

/// A display string: trimmed, without invisible characters, at most `max` characters. `None` when
/// nothing is left.
pub(crate) fn clean(text: &str, max: usize) -> Option<String> {
    let kept: String = text.chars().filter(|ch| !is_invisible(*ch)).collect();
    let kept: String = kept.trim().chars().take(max).collect();
    let kept = kept.trim_end().to_owned();
    (!kept.is_empty()).then_some(kept)
}

/// `[a-z0-9_]` identifier of the text, at most `max` characters; `None` when nothing is left.
pub(crate) fn slug(text: &str, max: usize) -> Option<String> {
    let parts: Vec<String> = tokens(text)
        .into_iter()
        .map(|token| match token {
            Token::Word(word) => word,
            Token::Number(number) => number.to_string(),
        })
        .collect();
    let joined: String = parts.join("_").chars().filter(char::is_ascii).take(max).collect();
    let joined = joined.trim_matches('_').to_owned();
    (!joined.is_empty()).then_some(joined)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(text: &str) -> Vec<String> {
        tokens(text).into_iter().filter_map(Token::word).collect()
    }

    #[test]
    fn tokens_split_on_anything_that_is_not_a_letter_or_digit() {
        assert_eq!(words("Gemini-3 Pro (Low)"), ["gemini", "pro", "low"]);
        assert_eq!(words("MODEL_OPENAI_GPT_OSS_120B"), ["model", "openai", "gpt", "oss", "120b"]);
        assert_eq!(
            tokens("in 5 days, 11 hours"),
            [
                Token::Word("in".into()),
                Token::Number(5),
                Token::Word("days".into()),
                Token::Number(11),
                Token::Word("hours".into()),
            ]
        );
    }

    #[test]
    fn a_digit_run_is_a_number_only_when_it_is_short_and_plain() {
        assert_eq!(tokens("007"), [Token::Number(7)]);
        assert_eq!(tokens("1234567890"), [Token::Word("1234567890".into())]);
        assert_eq!(tokens("5h"), [Token::Word("5h".into())]);
        assert_eq!(tokens("\u{661}\u{662}"), [Token::Word("\u{661}\u{662}".into())]);
        assert_eq!(tokens("1.5"), [Token::Number(1), Token::Number(5)]);
    }

    #[test]
    fn unicode_letters_stay_words_and_empty_text_has_no_tokens() {
        assert_eq!(words("Cota \u{c9}pica"), ["cota", "\u{e9}pica"]);
        assert!(tokens("").is_empty());
        assert!(tokens(" -- () _ ").is_empty());
    }

    #[test]
    fn tokens_are_bounded_in_text_and_in_count() {
        let long_word = "a".repeat(10_000);
        assert_eq!(words(&long_word), ["a".repeat(MAX_CHARS)]);
        // The cut counts characters, so a multi-byte letter is never split.
        let wide = "\u{e9}".repeat(1_000);
        assert_eq!(words(&wide), ["\u{e9}".repeat(MAX_CHARS)]);
        let many = "1 ".repeat(MAX_TOKENS * 4);
        assert_eq!(tokens(&many).len(), MAX_TOKENS);
        assert!(tokens(&"\u{1f600}".repeat(1_000)).is_empty());
    }

    #[test]
    fn clean_strips_invisible_characters_and_bounds_the_length() {
        assert_eq!(clean("  Pro \t", 16).as_deref(), Some("Pro"));
        assert_eq!(clean("Go\u{202e}od\u{200b}", 16).as_deref(), Some("Good"));
        assert_eq!(clean("a\u{0}b\nc", 16).as_deref(), Some("abc"));
        assert_eq!(clean("Google AI Ultra", 6).as_deref(), Some("Google"));
        assert_eq!(clean("Google AI Ultra", 7).as_deref(), Some("Google"));
        assert_eq!(clean("\u{200b} \u{202e}", 16), None);
        assert_eq!(clean("", 16), None);
    }

    #[test]
    fn slug_is_a_short_ascii_identifier() {
        assert_eq!(slug("Image Models (beta)", 32).as_deref(), Some("image_models_beta"));
        assert_eq!(slug("gemini-5h", 32).as_deref(), Some("gemini_5h"));
        assert_eq!(slug("A very long group name that goes on", 12).as_deref(), Some("a_very_long"));
        assert_eq!(slug("Cota \u{c9}pica", 32).as_deref(), Some("cota_pica"));
        assert_eq!(slug("--", 32), None);
        assert_eq!(slug("\u{e9}", 32), None);
    }
}
