use crate::{
    environment::Environment,
    util::{GlobRule, Normalize},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    pub source: SourceConfig,
}

impl Config {
    pub fn prepare(&mut self, e: &impl Environment, base: &Path) -> anyhow::Result<()> {
        self.source.prepare(e, base)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    /// A list of UNIX-style glob patterns
    /// for Rhai files that should be included.
    pub include: Option<Vec<String>>,
    /// A list of UNIX-style glob patterns.
    /// For Rhai files that should be excluded.
    pub exclude: Option<Vec<String>>,

    #[serde(skip)]
    pub file_rule: Option<GlobRule>,
}

impl SourceConfig {
    pub fn prepare(&mut self, e: &impl Environment, base: &Path) -> anyhow::Result<()> {
        self.include = self
            .include
            .take()
            .or_else(|| Some(vec![String::from("**/*.rhai")]));
        self.make_absolute(e, base);

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
