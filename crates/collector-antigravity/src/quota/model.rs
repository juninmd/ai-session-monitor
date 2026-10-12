//! The vocabulary both parsers speak: which pool, how long a window is, and what was read.

use super::values::tokens;

/// A quota pool: the models that draw from one allowance.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Pool {
    /// Gemini Pro and Flash share it.
    Gemini,
    /// Claude and GPT-OSS keep a pool of their own.
    ClaudeGpt,
    /// A group the source names that this collector does not know; its slug is the id.
    Other(String),
}

impl Pool {
    pub(crate) fn id(&self) -> &str {
        match self {
            Self::Gemini => "gemini",
            Self::ClaudeGpt => "claude_gpt",
            Self::Other(slug) => slug,
        }
    }

    /// The pool a group or model name points at (`"Gemini 3 Pro (Low)"`, `"Claude and GPT
    /// models"`); `None` when it names neither side or both.
    pub(crate) fn of_name(text: &str) -> Option<Self> {
        let (mut gemini, mut third_party) = (false, false);
        for word in tokens(text).into_iter().filter_map(|token| token.word()) {
            gemini |= word.starts_with("gemini");
            third_party |= word.starts_with("claude")
                || word.starts_with("gpt")
                || matches!(word.as_str(), "oss" | "openai" | "sonnet" | "opus" | "haiku");
        }
        match (gemini, third_party) {
            (true, false) => Some(Self::Gemini),
            (false, true) => Some(Self::ClaudeGpt),
            _ => None,
        }
    }
}

/// How long a window is. Both lengths are fixed by the plans: Pro and Ultra refresh every five
/// hours until a weekly cap, other plans refresh weekly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Cadence {
    FiveHour,
    Weekly,
}

/// One window of one pool, before it becomes a `UsageWindow`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PoolWindow {
    pub pool: Pool,
    /// `None`: nothing says how long the window is.
    pub cadence: Option<Cadence>,
    /// Guessed from the reset time instead of stated by the source: no length is published for it.
    pub inferred: bool,
    /// Provider name for a window of unknown cadence, which the UI cannot name by kind.
    pub label: Option<String>,
    /// Id part for a window of unknown cadence.
    pub slug: Option<String>,
    /// `None` is unknown, never 0.
    pub used_pct: Option<f64>,
    pub resets_at: Option<i64>,
}

impl PoolWindow {
    pub(crate) fn new(pool: Pool) -> Self {
        Self {
            pool,
            cadence: None,
            inferred: false,
            label: None,
            slug: None,
            used_pct: None,
            resets_at: None,
        }
    }
}

/// What one response said.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Read {
    pub plan: Option<String>,
    pub windows: Vec<PoolWindow>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn models_and_groups_land_in_their_pool() {
        for (name, pool) in [
            ("Gemini 3 Pro (Low)", Pool::Gemini),
            ("Gemini 3 Flash", Pool::Gemini),
            ("Gemini Models", Pool::Gemini),
            ("MODEL_PLACEHOLDER gemini-3-pro", Pool::Gemini),
            ("Claude Sonnet 4.5", Pool::ClaudeGpt),
            ("MODEL_CLAUDE_4_5_SONNET", Pool::ClaudeGpt),
            ("GPT-OSS 120B (Medium)", Pool::ClaudeGpt),
            ("MODEL_OPENAI_GPT_OSS_120B_MEDIUM", Pool::ClaudeGpt),
            ("Claude and GPT models", Pool::ClaudeGpt),
            ("gpt5", Pool::ClaudeGpt),
        ] {
            assert_eq!(Pool::of_name(name), Some(pool), "{name}");
        }
    }

    #[test]
    fn names_that_point_nowhere_or_both_ways_have_no_pool() {
        for name in ["", "Image models", "Gemini and Claude", "crossover", "bossa"] {
            assert_eq!(Pool::of_name(name), None, "{name}");
        }
    }

    #[test]
    fn pools_sort_gemini_then_claude_gpt_then_the_rest() {
        let mut pools =
            vec![Pool::Other("b".into()), Pool::ClaudeGpt, Pool::Other("a".into()), Pool::Gemini];
        pools.sort();
        assert_eq!(
            pools,
            [Pool::Gemini, Pool::ClaudeGpt, Pool::Other("a".into()), Pool::Other("b".into())]
        );
    }
}
