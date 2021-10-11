#![allow(dead_code)]

use xshell::cmd;

#[must_use]
pub fn format_rust(src: &str) -> String {
    let mut stdout = cmd!("rustfmt --config fn_single_line=true")
        .stdin(src)
        .read()
        .unwrap();
    if !stdout.ends_with('\n') {
        stdout.push('\n');
    }

    stdout
}
