use std::path::{Path, PathBuf};

use figment::{providers::Serialized, Figment};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    environment::Environment,
    utils::{GlobRule, Normalize},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RhaiConfig {
    pub source: RhaiSourceConfig,
}

impl RhaiConfig {
    pub fn prepare(&mut self, e: &impl Environment, base: &Path) -> anyhow::Result<()> {
        self.source.prepare(e, base)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RhaiSourceConfig {
    /// A list of UNIX-style glob patterns
    /// for Rhai files that should be included.
    pub include: Option<Vec<String>>,
    /// A list of UNIX-style glob patterns.
    /// For Rhai files that should be excluded.
    pub exclude: Option<Vec<String>>,

    #[serde(skip)]
    pub file_rule: Option<GlobRule>,
}

impl RhaiSourceConfig {
    pub fn prepare(&mut self, e: &impl Environment, base: &Path) -> anyhow::Result<()> {
        self.make_absolute(e, base);

        self.include = self
            .include
            .take()
            .or_else(|| Some(vec![String::from("**/*.rhai")]));

        self.file_rule = Some(GlobRule::new(
            self.include.as_deref().unwrap(),
            self.exclude.as_deref().unwrap_or(&[] as &[String]),
        )?);

        Ok(())
    }

    fn make_absolute(&mut self, e: &impl Environment, base: &Path) {
        if let Some(included) = &mut self.include {
            for pat in included {
                if !e.is_absolute(Path::new(pat)) {
                    *pat = base
                        .join(pat.as_str())
                        .normalize()
                        .to_string_lossy()
                        .into_owned();
                }
            }
        }

        if let Some(excluded) = &mut self.exclude {
            for pat in excluded {
                if !e.is_absolute(Path::new(pat)) {
                    *pat = base
                        .join(pat.as_str())
                        .normalize()
                        .to_string_lossy()
                        .into_owned();
                }
            }
        }
    }
}

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
