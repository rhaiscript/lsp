#![allow(dead_code)]


#[must_use]
pub fn format_rust(src: &str) -> String {
    prettyplease::unparse(&syn::parse_str(src).unwrap())
}
