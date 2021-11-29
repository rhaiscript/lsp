use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhaiConfig {
    pub include_directories: Option<Vec<PathBuf>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoRhaiConfig {
    pub project: Option<CargoMetadata>,
    pub workspace: Option<CargoMetadata>,
    pub package: Option<CargoMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoMetadata {
    pub rhai: Option<RhaiConfig>,
}
