//! Which window a bucket is, from the words the server uses for it.

use super::text::{Token, tokens};
use crate::quota::Cadence;

const HOURS: [&str; 5] = ["h", "hr", "hrs", "hour", "hours"];
const DAYS: [&str; 3] = ["d", "day", "days"];

/// The first text that names exactly one cadence decides (`window`, then `bucketId`, then
/// `displayName`); a text that names none or both defers to the next one.
pub(crate) fn cadence_of(texts: &[Option<&str>]) -> Option<Cadence> {
    texts.iter().flatten().find_map(|text| classify(&tokens(text)))
}

fn classify(found: &[Token]) -> Option<Cadence> {
    let (mut five, mut week) = (false, false);
    for (index, token) in found.iter().enumerate() {
        let next = found.get(index + 1);
        match token {
            Token::Word(word) => match word.as_str() {
                "5h" | "5hr" | "5hrs" | "5hour" | "5hours" | "fivehour" | "session" => five = true,
                "weekly" | "week" | "7d" | "7day" | "7days" => week = true,
                "five" => five |= is_unit(next, &HOURS),
                _ => {}
            },
            Token::Number(5) => five |= is_unit(next, &HOURS),
            Token::Number(7) => week |= is_unit(next, &DAYS),
            Token::Number(_) => {}
        }
    }
    match (five, week) {
        (true, false) => Some(Cadence::FiveHour),
        (false, true) => Some(Cadence::Weekly),
        _ => None,
    }
}

fn is_unit(token: Option<&Token>, units: &[&str]) -> bool {
    matches!(token, Some(Token::Word(word)) if units.contains(&word.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn of(text: &str) -> Option<Cadence> {
        cadence_of(&[Some(text)])
    }

    #[test]
    fn five_hour_buckets_are_recognized_by_any_of_their_names() {
        for text in [
            "5h",
            "5H",
            "5hr",
            "5-hour",
            "5 hours",
            "five-hour",
            "Five Hour Limit",
            "gemini-5h",
            "3p-5h",
            "claude_gpt:5h",
            "Session",
        ] {
            assert_eq!(of(text), Some(Cadence::FiveHour), "{text}");
        }
    }

    #[test]
    fn weekly_buckets_are_recognized_by_any_of_their_names() {
        for text in ["weekly", "Weekly Limit", "gemini-weekly", "3p-weekly", "7d", "7 days", "week"]
        {
            assert_eq!(of(text), Some(Cadence::Weekly), "{text}");
        }
    }

    #[test]
    fn other_or_contradictory_names_say_nothing() {
        for text in [
            "",
            "limit",
            "daily",
            "monthly",
            "50 hours",
            "5 days",
            "7 hours",
            "15h",
            "5h weekly",
            "Five Hour and Weekly",
        ] {
            assert_eq!(of(text), None, "{text}");
        }
    }

    #[test]
    fn the_first_text_that_decides_wins() {
        let pick = |texts: &[Option<&str>]| cadence_of(texts);
        assert_eq!(pick(&[Some("weekly"), Some("gemini-5h")]), Some(Cadence::Weekly));
        assert_eq!(pick(&[None, Some("gemini-5h"), Some("Weekly Limit")]), Some(Cadence::FiveHour));
        assert_eq!(pick(&[Some("5h weekly"), Some("x-weekly")]), Some(Cadence::Weekly));
        assert_eq!(pick(&[Some(""), None, Some("bucket")]), None);
        assert_eq!(pick(&[]), None);
    }
}
