use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Options {
    pub crlf: bool,
    pub max_empty_lines: u64,
    pub max_width: u64,
    #[serde(with = "serde_indent_str")]
    pub indent_string: Arc<str>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            crlf: false,
            max_width: 80,
            max_empty_lines: 2,
            indent_string: Arc::from("  "),
        }
    }
}

mod serde_indent_str {
    use super::*;

    pub fn serialize<S: serde::ser::Serializer>(rc: &Arc<str>, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&**rc)
    }

    pub fn deserialize<'de, S: serde::de::Deserializer<'de>>(de: S) -> Result<Arc<str>, S::Error> {
        let s = <&str>::deserialize(de)?;
        Ok(Arc::from(s))
    }
}
