use rhai::{Engine, EvalAltResult, Scope, INT};
use std::sync::{Arc, RwLock};

#[cfg(not(feature = "only_i32"))]
#[cfg(not(feature = "only_i64"))]
#[test]
fn test_to_string() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    let mut scope = Scope::new();
    scope.push("x", 42_u8);
    scope.push("y", 42_i32);
    scope.push("z", 42_i16);

    assert_eq!(
        engine.eval_with_scope::<String>(&mut scope, "to_string(x)")?,
        "42"
    );
    assert_eq!(
        engine.eval_with_scope::<String>(&mut scope, "to_string(x)")?,
        "42"
    );
    assert_eq!(
        engine.eval_with_scope::<String>(&mut scope, "to_string(x)")?,
        "42"
    );

    Ok(())
}

#[test]
fn test_print_debug() -> Result<(), Box<EvalAltResult>> {
    let logbook = Arc::new(RwLock::new(Vec::<String>::new()));

    // Redirect print/debug output to 'log'
    let log1 = logbook.clone();
    let log2 = logbook.clone();

    let mut engine = Engine::new();

    engine
        .on_print(move |s| log1.write().unwrap().push(format!("entry: {}", s)))
        .on_debug(move |s, src, pos| {
            log2.write().unwrap().push(format!(
                "DEBUG of {} at {:?}: {}",
                src.unwrap_or("unknown"),
                pos,
                s
            ))
        });

    // Evaluate script
    engine.run("print(40 + 2)")?;
    let mut ast = engine.compile(r#"let x = "hello!"; debug(x)"#)?;
    ast.set_source("world");
    engine.run_ast(&ast)?;

    // 'logbook' captures all the 'print' and 'debug' output
    assert_eq!(logbook.read().unwrap().len(), 2);
    assert_eq!(logbook.read().unwrap()[0], "entry: 42");
    assert_eq!(
        logbook.read().unwrap()[1],
        if cfg!(not(feature = "no_position")) {
            r#"DEBUG of world at 1:19: "hello!""#
        } else {
            r#"DEBUG of world at none: "hello!""#
        }
    );

    for entry in logbook.read().unwrap().iter() {
        println!("{}", entry);
    }

    Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
struct MyStruct {
    field: INT,
}

impl std::fmt::Display for MyStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "hello: {}", self.field)
    }
}

#[cfg(not(feature = "no_object"))]
#[test]
fn test_print_custom_type() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    engine
        .register_type_with_name::<MyStruct>("MyStruct")
        .register_fn("to_debug", |x: &mut MyStruct| x.to_string())
        .register_fn("debug", |x: &mut MyStruct| x.to_string())
        .register_fn("new_ts", || MyStruct { field: 42 });

    engine.run("let x = new_ts(); debug(x);")?;

    #[cfg(not(feature = "no_index"))]
    assert_eq!(
        engine.eval::<String>(
            r#"
                let x = [ 123, true, (), "world", new_ts() ];
                x.to_string()
            "#
        )?,
        r#"[123, true, (), "world", hello: 42]"#
    );

    assert!(engine
        .eval::<String>(
            r#"
                let x = #{ a:123, b:true, c:(), d:"world", e:new_ts() };
                x.to_string()
            "#
        )?
        .contains(r#""e": hello: 42"#));
    Ok(())
}
