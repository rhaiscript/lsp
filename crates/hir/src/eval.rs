use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
    Unknown
}

impl Default for Value {
    fn default() -> Self {
        Self::Unknown
    }
}