use serde::{Deserialize, Serialize};

macro_rules! create_options {
    (
        $(#[$attr:meta])*
        pub struct Options {
            $(
                $(#[$field_attr:meta])*
                pub $name:ident: $ty:ty,
            )+
        }
    ) => {
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        $(#[$attr])*
        pub struct Options {
            $(
                $(#[$field_attr])*
                pub $name: $ty,
            )+
        }

        impl Options {
            pub fn update(&mut self, incomplete: OptionsIncomplete) {
                $(
                    if let Some(v) = incomplete.$name {
                        self.$name = v;
                    }
                )+
            }

            pub fn update_camel(&mut self, incomplete: OptionsIncompleteCamel) {
                $(
                    if let Some(v) = incomplete.$name {
                        self.$name = v;
                    }
                )+
            }

            pub fn update_from_str<S: AsRef<str>, I: Iterator<Item = (S, S)>>(
                &mut self,
                values: I,
            ) -> Result<(), OptionParseError> {
                for (key, val) in values {

                    $(
                        if key.as_ref() == stringify!($name) {
                            self.$name =
                                val.as_ref()
                                    .parse()
                                    .map_err(|error| OptionParseError::InvalidValue {
                                        key: key.as_ref().into(),
                                        error: Box::new(error),
                                    })?;

                            continue;
                        }
                    )+

                    return Err(OptionParseError::InvalidOption(key.as_ref().into()));
                }

                Ok(())
            }
        }

        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        $(#[$attr])*
        #[derive(Default)]
        pub struct OptionsIncomplete {
            $(
                $(#[$field_attr])*
                #[serde(skip_serializing_if = "Option::is_none")]
                pub $name: Option<$ty>,
            )+
        }

        impl OptionsIncomplete {
            pub fn from_options(opts: Options) -> Self {
                let mut o = Self::default();

                $(
                    o.$name = Some(opts.$name);
                )+

                o
            }
        }

        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        $(#[$attr])*
        #[derive(Default)]
        #[serde(rename_all = "camelCase")]
        pub struct OptionsIncompleteCamel {
            $(
                $(#[$field_attr])*
                #[serde(skip_serializing_if = "Option::is_none")]
                pub $name: Option<$ty>,
            )+
        }

        impl OptionsIncompleteCamel {
            pub fn from_options(opts: Options) -> Self {
                let mut o = Self::default();

                $(
                    o.$name = Some(opts.$name);
                )+

                o
            }
        }
    };
}

create_options! {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Options {
        /// Whether to use CRLF for line endings instead of LF.
        pub crlf: bool,
        /// Amount of allowed consecutive empty lines.
        pub max_empty_lines: u64,
        /// Target maximum column width in bytes.
        pub max_width: u64,
        /// String to use for indentation.
        ///
        /// This is typically some amount of spaces or a tab character.
        pub indent_string: String,
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            crlf: false,
            max_width: 80,
            max_empty_lines: 2,
            indent_string: String::from("  "),
        }
    }
}

#[derive(Debug)]
pub enum OptionParseError {
    InvalidOption(String),
    InvalidValue {
        key: String,
        error: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl core::fmt::Display for OptionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid formatting option: {}",
            match self {
                OptionParseError::InvalidOption(k) => {
                    format!(r#"invalid option "{}""#, k)
                }
                OptionParseError::InvalidValue { key, error } => {
                    format!(r#"invalid value for option "{}": {}"#, key, error)
                }
            }
        )
    }
}

impl std::error::Error for OptionParseError {}
