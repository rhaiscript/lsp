///! This example registers a variety of functions that operate on strings.
///! Remember to use `ImmutableString` or `&str` instead of `String` as parameters.
use rhai::{Engine, EvalAltResult, ImmutableString, Scope, INT};
use std::io::{stdin, stdout, Write};

/// Trim whitespace from a string. The original string argument is changed.
///
/// This version uses `&mut ImmutableString`
fn trim_string(s: &mut ImmutableString) {
    *s = s.trim().into();
}

/// Notice this is different from the built-in Rhai 'len' function for strings
/// which counts the actual number of Unicode _characters_ in a string.
/// This version simply counts the number of _bytes_ in the UTF-8 representation.
///
/// This version uses `&str`.
fn count_string_bytes(s: &str) -> INT {
    s.len() as INT
}

/// This version uses `ImmutableString` and `&str`.
fn find_substring(s: ImmutableString, sub: &str) -> INT {
    s.find(sub).map(|x| x as INT).unwrap_or(-1)
}

fn main() -> Result<(), Box<EvalAltResult>> {
    // Create a `raw` Engine with no built-in string functions.
    let mut engine = Engine::new_raw();

    // Register string functions
    engine
        .register_fn("trim", trim_string)
        .register_fn("len", count_string_bytes)
        .register_fn("index_of", find_substring)
        // Register string functions using closures
        .register_fn("display", |label: &str, value: INT| {
            println!("{}: {}", label, value)
        })
        .register_fn("display", |label: ImmutableString, value: &str| {
            println!(r#"{}: "{}""#, label, value) // Quote the input string
        });

    let mut scope = Scope::new();
    let mut input = String::new();

    loop {
        scope.clear();

        println!("Type something. Press Ctrl-C to exit.");
        print!("strings> ");
        stdout().flush().expect("couldn't flush stdout");

        input.clear();

        if let Err(err) = stdin().read_line(&mut input) {
            panic!("input error: {}", err);
        }

        scope.push("x", input.clone());

        println!("Line: {}", input.replace('\r', "\\r").replace('\n', "\\n"));

        engine.run_with_scope(
            &mut scope,
            r#"
                display("Length", x.len());
                x.trim();
                display("Trimmed", x);
                display("Trimmed Length", x.len());
                display("Index of \"!!!\"", x.index_of("!!!"));
            "#,
        )?;

        println!();
    }
}
