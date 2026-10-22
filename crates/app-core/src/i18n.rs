//! Wording of everything the Rust side shows the user: desktop notifications, the tray tooltip and
//! menu and the startup failure toasts. The language is `settings.general.language`; the webview
//! has its own catalogs (`src/i18n`) and localizes error codes itself.
//!
//! Each language is one self-contained file, `i18n/lang_<tag>.rs`, that fills every field of
//! [`Messages`] with a template. A template holds `{name}` placeholders; [`fill`] replaces them.
//! Nothing here inflects words: a sentence takes a ready-made phrase (a window name, a duration)
//! through a placeholder, so a language can reorder the pieces freely.

use usage_model::HarnessId;
use usage_model::settings::Language;

mod lang_ar;
mod lang_bn;
mod lang_en;
mod lang_es;
mod lang_fr;
mod lang_hi;
mod lang_pt_br;
mod lang_ru;
mod lang_ur;
mod lang_zh_cn;
mod messages;

pub use messages::{Entry, Messages};

#[cfg(test)]
mod tests;

/// The strings of `language`.
pub fn messages(language: Language) -> &'static Messages {
    match language {
        Language::En => &lang_en::MESSAGES,
        Language::ZhCn => &lang_zh_cn::MESSAGES,
        Language::Hi => &lang_hi::MESSAGES,
        Language::Es => &lang_es::MESSAGES,
        Language::Fr => &lang_fr::MESSAGES,
        Language::Ar => &lang_ar::MESSAGES,
        Language::Bn => &lang_bn::MESSAGES,
        Language::PtBr => &lang_pt_br::MESSAGES,
        Language::Ru => &lang_ru::MESSAGES,
        Language::Ur => &lang_ur::MESSAGES,
    }
}

/// Replaces each `{name}` in `template` with its value from `values`. One pass: a value is never
/// scanned again, so text from a remote service cannot inject another placeholder. A `{name}`
/// with no value, and a stray brace, stay as written.
pub fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let tail = &rest[open..];
        let found = tail.find('}').and_then(|close| {
            let name = &tail[1..close];
            let (_, value) = values.iter().find(|(key, _)| *key == name)?;
            Some((close, *value))
        });
        match found {
            Some((close, value)) => {
                out.push_str(value);
                rest = &tail[close + 1..];
            }
            None => {
                out.push('{');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Product names are the same in every language.
pub fn harness_name(id: HarnessId) -> &'static str {
    match id {
        HarnessId::Claude => "Claude Code",
        HarnessId::Codex => "Codex",
        HarnessId::Cursor => "Cursor",
        HarnessId::Copilot => "GitHub Copilot",
        HarnessId::Antigravity => "Antigravity",
        HarnessId::Opencode => "OpenCode",
        HarnessId::Kimi => "Kimi Code",
        HarnessId::Zai => "z.ai",
        HarnessId::Openrouter => "OpenRouter",
        HarnessId::Deepseek => "DeepSeek",
        HarnessId::Deepinfra => "DeepInfra",
        HarnessId::Moonshot => "Moonshot",
        HarnessId::Novita => "Novita",
        HarnessId::Ollama => "Ollama Cloud",
        HarnessId::Minimax => "MiniMax",
        HarnessId::Litellm => "LiteLLM",
    }
}
