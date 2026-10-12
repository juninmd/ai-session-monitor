//! Display text for Codex's `planType` identifiers.

const MAX_PLAN_CHARS: usize = 40;

/// `plus` becomes "Plus", `self_serve_business_prolite` "Self Serve Business Prolite". The
/// identifier is an enum on the wire, so anything that does not look like one (or says `unknown`)
/// is dropped instead of being shown.
pub(crate) fn label(raw: &str) -> Option<String> {
    let raw = raw.trim();
    let identifier = !raw.is_empty()
        && raw.len() <= MAX_PLAN_CHARS
        && raw.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if !identifier || raw.eq_ignore_ascii_case("unknown") {
        return None;
    }
    let words: Vec<String> =
        raw.split(['_', '-']).filter(|word| !word.is_empty()).map(title).collect();
    (!words.is_empty()).then(|| words.join(" "))
}

fn title(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_ascii_uppercase().to_string() + &chars.as_str().to_ascii_lowercase()
    })
}

#[cfg(test)]
mod tests {
    use super::label;

    #[test]
    fn known_identifiers_read_as_words() {
        assert_eq!(label("plus").as_deref(), Some("Plus"));
        assert_eq!(label("PRO").as_deref(), Some("Pro"));
        assert_eq!(label("prolite").as_deref(), Some("Prolite"));
        assert_eq!(
            label("self_serve_business_prolite").as_deref(),
            Some("Self Serve Business Prolite")
        );
        assert_eq!(label("edu-plus").as_deref(), Some("Edu Plus"));
        assert_eq!(label(" plus\n").as_deref(), Some("Plus"));
    }

    #[test]
    fn unknown_empty_and_hostile_values_are_not_shown() {
        for raw in ["", "  ", "unknown", "UNKNOWN", "_", "--", "a b", "<b>pro</b>", "pró"] {
            assert_eq!(label(raw), None, "{raw:?}");
        }
        assert_eq!(label(&"x".repeat(41)), None);
        assert_eq!(label(&"x".repeat(40)).map(|text| text.len()), Some(40));
    }
}
