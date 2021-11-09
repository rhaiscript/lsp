use crate::Symbol;
use rhai_rowan::TextRange;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("{kind}")]
pub struct Error {
    pub text_range: Option<TextRange>,
    pub kind: ErrorKind,
}

#[derive(Debug, Clone, Error)]
pub enum ErrorKind {
    #[error("duplicate parameter `{name}` for function `{fn_name}`")]
    DuplicateFnParameter {
        name: String,
        fn_name: String,
        duplicate_symbol: Symbol,
        duplicate_range: Option<TextRange>,
        existing_symbol: Symbol,
        existing_range: Option<TextRange>,
    },

    #[error(
        "cannot resolve reference `{reference_name}`{}",
        match &similar_name {
            Some(n) => {
                format!(" maybe you wanted `{}`?", n)
            }
            None => {
                String::from("")
            }
        }
    )]
    UnresolvedReference {
        reference_name: String,
        similar_name: Option<String>,
        reference_symbol: Symbol,
        reference_range: Option<TextRange>,
    }
}
