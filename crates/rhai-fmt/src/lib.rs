// #![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::derive_partial_eq_without_eq,
    clippy::doc_markdown,
    clippy::enum_glob_use,
    clippy::items_after_statements,
    clippy::match_like_matches_macro,
    clippy::match_same_arms,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::unused_self,
    clippy::vec_init_then_push,
    clippy::wildcard_imports,
    clippy::cast_possible_truncation
)]

mod algorithm;
mod comments;
mod cst;
mod expr;
mod item;
mod path;
mod ring;
mod source;
mod util;

pub mod options;

pub use options::Options;
use rhai_rowan::{parser::Parser, syntax::SyntaxElement};

pub use algorithm::Formatter;

#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn format_syntax(cst: impl Into<SyntaxElement>, options: Options) -> String {
    let mut s = Vec::new();
    Formatter::new_with_options(&mut s, options)
        .format(cst)
        .unwrap();
    // SAFETY: we only print valid UTF-8.
    unsafe { String::from_utf8_unchecked(s) }
}

#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn format_source(src: &str, options: Options) -> String {
    let mut s = Vec::new();

    let cst = if rhai_rowan::util::is_rhai_def(src) {
        Parser::new(src).parse_def().into_syntax()
    } else {
        Parser::new(src).parse_script().into_syntax()
    };

    Formatter::new_with_options(&mut s, options)
        .format(cst)
        .unwrap();
    // SAFETY: we only print valid UTF-8.
    unsafe { String::from_utf8_unchecked(s) }
}
