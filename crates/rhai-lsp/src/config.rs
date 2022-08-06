use figment::{providers::Serialized, Figment};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitConfig {
    pub cache_path: Option<PathBuf>,
    #[serde(default = "default_configuration_section")]
    pub configuration_section: String,
}

impl Default for InitConfig {
    fn default() -> Self {
        Self {
            cache_path: Default::default(),
            configuration_section: default_configuration_section(),
        }
    }
}

fn default_configuration_section() -> String {
    String::from("rhai")
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspConfig {
    pub syntax: SyntaxConfig,
}

impl LspConfig {
    pub fn update_from_json(&mut self, json: &Value) -> Result<(), anyhow::Error> {
        *self = Figment::new()
            .merge(Serialized::defaults(&self))
            .merge(Serialized::defaults(json))
            .extract()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyntaxConfig {
    pub semantic_tokens: bool,
}

impl Default for SyntaxConfig {
    fn default() -> Self {
        Self {
            semantic_tokens: true,
        }
    }
}
