#![no_main]
use rhai_rowan::parser::Parser;

#[macro_use]
extern crate libfuzzer_sys;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        Parser::new(s).parse();
    }
});
