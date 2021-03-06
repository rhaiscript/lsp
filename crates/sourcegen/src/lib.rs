#![warn(clippy::pedantic)]
#![allow(
    clippy::unused_async,
    clippy::single_match,
    clippy::wildcard_imports,
    clippy::too_many_lines,
    clippy::enum_glob_use,
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::module_name_repetitions,
    clippy::single_match_else,
    clippy::option_option,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]

pub mod syntax;
pub mod util;
