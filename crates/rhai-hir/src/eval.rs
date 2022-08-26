#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
    Unknown,
}

impl core::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(v) => v.fmt(f),
            Value::Float(v) => v.fmt(f),
            Value::Bool(v) => v.fmt(f),
            Value::String(v) => write!(f, r#""{v}""#),
            Value::Char(v) => write!(f, "'{v}'"),
            Value::Unknown => "UNKNOWN VALUE".fmt(f),
        }
    }
}

impl Value {
    /// Returns `true` if the value is [`Int`].
    ///
    /// [`Int`]: Value::Int
    #[must_use]
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(..))
    }

    #[must_use]
    pub fn as_int(&self) -> Option<&i64> {
        if let Self::Int(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the value is [`Float`].
    ///
    /// [`Float`]: Value::Float
    #[must_use]
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(..))
    }

    #[must_use]
    pub fn as_float(&self) -> Option<&f64> {
        if let Self::Float(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the value is [`Bool`].
    ///
    /// [`Bool`]: Value::Bool
    #[must_use]
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(..))
    }

    #[must_use]
    pub fn as_bool(&self) -> Option<&bool> {
        if let Self::Bool(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the value is [`String`].
    ///
    /// [`String`]: Value::String
    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(..))
    }

    #[must_use]
    pub fn as_string(&self) -> Option<&String> {
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the value is [`Char`].
    ///
    /// [`Char`]: Value::Char
    #[must_use]
    pub fn is_char(&self) -> bool {
        matches!(self, Self::Char(..))
    }

    #[must_use]
    pub fn as_char(&self) -> Option<&char> {
        if let Self::Char(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the value is [`Unknown`].
    ///
    /// [`Unknown`]: Value::Unknown
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Unknown
    }
}
