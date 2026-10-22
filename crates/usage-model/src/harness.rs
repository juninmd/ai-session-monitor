use serde::{Deserialize, Serialize};

/// One AI tool ("agent harness") the monitor can read. Declaration order is the tab order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HarnessId {
    Claude,
    Codex,
    Cursor,
    Copilot,
    Antigravity,
    Opencode,
    Kimi,
    Zai,
    Openrouter,
    Deepseek,
    Deepinfra,
    Moonshot,
    Novita,
    Ollama,
    Minimax,
    Litellm,
}

impl HarnessId {
    pub const ALL: [Self; 16] = [
        Self::Claude,
        Self::Codex,
        Self::Cursor,
        Self::Copilot,
        Self::Antigravity,
        Self::Opencode,
        Self::Kimi,
        Self::Zai,
        Self::Openrouter,
        Self::Deepseek,
        Self::Deepinfra,
        Self::Moonshot,
        Self::Novita,
        Self::Ollama,
        Self::Minimax,
        Self::Litellm,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Cursor => "cursor",
            Self::Copilot => "copilot",
            Self::Antigravity => "antigravity",
            Self::Opencode => "opencode",
            Self::Kimi => "kimi",
            Self::Zai => "zai",
            Self::Openrouter => "openrouter",
            Self::Deepseek => "deepseek",
            Self::Deepinfra => "deepinfra",
            Self::Moonshot => "moonshot",
            Self::Novita => "novita",
            Self::Ollama => "ollama",
            Self::Minimax => "minimax",
            Self::Litellm => "litellm",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_as_lowercase_id_matching_as_str() {
        for id in HarnessId::ALL {
            let json = serde_json::to_string(&id).expect("serialize");
            assert_eq!(json, format!("\"{}\"", id.as_str()));
        }
    }
}
