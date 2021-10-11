use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Type {
    Object{},
    Unknown,
}

impl Default for Type {
    fn default() -> Self {
        Self::Unknown
    }
}
