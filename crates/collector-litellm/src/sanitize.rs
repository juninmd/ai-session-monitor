//! Names the proxy supplies (aliases, model names, durations) made safe to show in the UI.

use usage_support::scrub;

const MAX_CHARS: usize = 80;

/// `raw` without control characters, trimmed and bounded. `None` when nothing is left or when it
/// looks like a credential: an admin can type anything into an alias, including a key.
pub(crate) fn display_text(raw: &str, secret: &str) -> Option<String> {
    // A key pasted with edge spaces is echoed by a server without them.
    let needle = secret.trim();
    // Cleaned first, so a key split by control characters is still found.
    let cleaned: String = raw.chars().filter(|c| !c.is_control()).collect();
    let cleaned = cleaned.trim();
    let leaks = !needle.is_empty() && cleaned.contains(needle);
    if cleaned.is_empty() || leaks || scrub(cleaned) != cleaned {
        return None;
    }
    let mut chars = cleaned.chars();
    let shown: String = chars.by_ref().take(MAX_CHARS).collect();
    Some(if chars.next().is_some() { format!("{shown}…") } else { shown })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "lk-fixture-secret-0001";

    #[test]
    fn plain_names_pass_through_trimmed() {
        assert_eq!(display_text("  ci-bot  ", SECRET).as_deref(), Some("ci-bot"));
        assert_eq!(display_text("équipe α/β", SECRET).as_deref(), Some("équipe α/β"));
        assert_eq!(display_text("gpt-5.6-terra", SECRET).as_deref(), Some("gpt-5.6-terra"));
    }

    #[test]
    fn empty_and_control_only_text_is_dropped() {
        for raw in ["", "   ", "\n\t", "\u{0}\u{7}"] {
            assert_eq!(display_text(raw, SECRET), None, "{raw:?}");
        }
    }

    #[test]
    fn control_characters_are_removed_not_rendered() {
        assert_eq!(display_text("a\u{0}b\nc\u{1b}[31md", SECRET).as_deref(), Some("abc[31md"));
    }

    #[test]
    fn long_text_is_cut_on_a_character_boundary() {
        let long = "é".repeat(200);
        let shown = display_text(&long, SECRET).expect("shown");
        assert_eq!(shown.chars().count(), MAX_CHARS + 1);
        assert!(shown.ends_with('…'));
        let exact = "x".repeat(MAX_CHARS);
        assert_eq!(display_text(&exact, SECRET).as_deref(), Some(exact.as_str()));
    }

    #[test]
    fn the_configured_secret_is_never_shown_even_inside_a_longer_name() {
        assert_eq!(display_text(SECRET, SECRET), None);
        assert_eq!(display_text(&format!("team {SECRET} prod"), SECRET), None);
    }

    #[test]
    fn a_key_stored_with_edge_spaces_is_caught_without_them() {
        assert_eq!(display_text("sk-abc", " sk-abc \t"), None);
        assert_eq!(display_text("team sk-abc prod", " sk-abc "), None);
        assert_eq!(display_text("team", " sk-abc "), Some("team".to_owned()));
        assert_eq!(display_text("anything", "   ").as_deref(), Some("anything"));
    }

    #[test]
    fn a_secret_split_by_control_characters_is_still_caught() {
        let split = format!("{}\u{0}{}", &SECRET[..8], &SECRET[8..]);
        assert_eq!(display_text(&split, SECRET), None);
    }

    #[test]
    fn credential_shaped_text_is_dropped() {
        for raw in ["sk-abcdefghijklmnopqrstuvwxyz0123456789", "Bearer abcdef0123456789abcdef"] {
            assert_eq!(display_text(raw, SECRET), None, "{raw:?}");
        }
    }

    #[test]
    fn an_empty_secret_does_not_make_every_name_a_leak() {
        assert_eq!(display_text("anything", "").as_deref(), Some("anything"));
    }
}
