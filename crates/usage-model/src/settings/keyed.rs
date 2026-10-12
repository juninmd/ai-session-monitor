//! Settings of the key-only sources: saving the API key is the consent, so none has a flag for it.

use serde::{Deserialize, Serialize};

/// Kimi Code (Moonshot) quota. Saving the API key is the consent, so there is no flag for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct KimiSettings {
    pub enabled: bool,
}

impl Default for KimiSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// z.ai (GLM Coding Plan) quota. Saving the API key is the consent, so there is no flag for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ZaiSettings {
    pub enabled: bool,
}

impl Default for ZaiSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// OpenRouter key limit and credits. Saving the API key is the consent, so there is no flag for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OpenrouterSettings {
    pub enabled: bool,
}

impl Default for OpenrouterSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// DeepSeek account balance. Saving the API key is the consent, so there is no flag for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeepseekSettings {
    pub enabled: bool,
}

impl Default for DeepseekSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// DeepInfra balance, spend limit and month spend. Saving the API key is the consent, so there is no flag for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeepinfraSettings {
    pub enabled: bool,
}

impl Default for DeepinfraSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Moonshot figures. Saving the API key is the consent, so there is no flag for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MoonshotSettings {
    pub enabled: bool,
}

impl Default for MoonshotSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Novita figures. Saving the API key is the consent, so there is no flag for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NovitaSettings {
    pub enabled: bool,
}

impl Default for NovitaSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Ollama Cloud figures. Saving the API key is the consent, so there is no flag for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OllamaSettings {
    pub enabled: bool,
}

impl Default for OllamaSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// MiniMax figures. Saving the API key is the consent, so there is no flag for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MinimaxSettings {
    pub enabled: bool,
}

impl Default for MinimaxSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}
