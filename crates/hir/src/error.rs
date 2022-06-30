use crate::Symbol;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("{kind}")]
pub struct Error {
    pub kind: ErrorKind,
}

#[derive(Debug, Clone, Error)]
pub enum ErrorKind {
    #[error("duplicate function parameter")]
    DuplicateFnParameter {
        duplicate_symbol: Symbol,
        existing_symbol: Symbol,
    },
    #[error(
        "cannot resolve reference{}",
        match &similar_name {
            Some(n) => {
                format!(", did you mean `{}`?", n)
            }
            None => {
                String::from("")
            }
        }
    )]
    UnresolvedReference {
        reference_symbol: Symbol,
        similar_name: Option<String>,
    },
    #[error("unresolved import")]
    UnresolvedImport { import: Symbol },
    #[error("nested functions are not allowed")]
    NestedFunction { function: Symbol },
}
