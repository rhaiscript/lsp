use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Unknown
}

impl Default for Value {
    fn default() -> Self {
        Self::Unknown
    }
}