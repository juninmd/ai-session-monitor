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
