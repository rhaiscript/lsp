#![cfg(not(feature = "no_function"))]
use rhai::{Dynamic, Engine, EvalAltResult, FnPtr, Func, FuncArgs, Scope, INT};
use std::{any::TypeId, iter::once};

#[test]
fn test_call_fn() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();
    let mut scope = Scope::new();

    scope.push("foo", 42 as INT);

    let ast = engine.compile(
        "
            fn hello(x, y) {
                x + y
            }
            fn hello(x) {
                x *= foo;
                foo = 1;
                x
            }
            fn hello() {
                41 + foo
            }
            fn define_var() {
                let bar = 21;
                bar * 2
            }
        ",
    )?;

    let r: INT = engine.call_fn(&mut scope, &ast, "hello", (42 as INT, 123 as INT))?;
    assert_eq!(r, 165);

    let r: INT = engine.call_fn(&mut scope, &ast, "hello", (123 as INT,))?;
    assert_eq!(r, 5166);

    let r: INT = engine.call_fn(&mut scope, &ast, "hello", ())?;
    assert_eq!(r, 42);

    let r: INT = engine.call_fn(&mut scope, &ast, "define_var", ())?;
    assert_eq!(r, 42);

    assert!(!scope.contains("bar"));

    assert_eq!(
        scope
            .get_value::<INT>("foo")
            .expect("variable foo should exist"),
        1
    );

    Ok(())
}

struct Options {
    pub foo: bool,
    pub bar: String,
    pub baz: INT,
}

impl FuncArgs for Options {
    fn parse<C: Extend<Dynamic>>(self, container: &mut C) {
        container.extend(once(self.foo.into()));
        container.extend(once(self.bar.into()));
        container.extend(once(self.baz.into()));
    }
}

#[test]
fn test_call_fn_args() -> Result<(), Box<EvalAltResult>> {
    let options = Options {
        foo: false,
        bar: "world".to_string(),
        baz: 42,
    };

    let engine = Engine::new();
    let mut scope = Scope::new();

    let ast = engine.compile(
        "
            fn hello(x, y, z) {
                if x { `hello ${y}` } else { y + z }
            }
        ",
    )?;

    let result: String = engine.call_fn(&mut scope, &ast, "hello", options)?;

    assert_eq!(result, "world42");

    Ok(())
}

#[test]
fn test_call_fn_private() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();
    let mut scope = Scope::new();

    let ast = engine.compile("fn add(x, n) { x + n }")?;

    let r: INT = engine.call_fn(&mut scope, &ast, "add", (40 as INT, 2 as INT))?;
    assert_eq!(r, 42);

    let ast = engine.compile("private fn add(x, n, ) { x + n }")?;

    let r: INT = engine.call_fn(&mut scope, &ast, "add", (40 as INT, 2 as INT))?;
    assert_eq!(r, 42);

    Ok(())
}

#[test]
#[cfg(not(feature = "no_object"))]
fn test_fn_ptr_raw() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    #[allow(deprecated)]
    engine
        .register_fn("mul", |x: &mut INT, y: INT| *x *= y)
        .register_raw_fn(
            "bar",
            &[
                TypeId::of::<INT>(),
                TypeId::of::<FnPtr>(),
                TypeId::of::<INT>(),
            ],
            move |context, args| {
                let fp = std::mem::take(args[1]).cast::<FnPtr>();
                let value = args[2].clone();
                let this_ptr = args.get_mut(0).unwrap();

                fp.call_dynamic(&context, Some(this_ptr), [value])
            },
        );

    assert_eq!(
        engine.eval::<INT>(
            r#"
                fn foo(x) { this += x; }

                let x = 41;
                x.bar(Fn("foo"), 1);
                x
            "#
        )?,
        42
    );

    assert_eq!(
        engine.eval::<INT>(
            r#"
                fn foo(x, y) { this += x + y; }

                let x = 40;
                let v = 1;
                x.bar(Fn("foo").curry(v), 1);
                x
            "#
        )?,
        42
    );

    assert_eq!(
        engine.eval::<INT>(
            r#"
                private fn foo(x) { this += x; }

                let x = 41;
                x.bar(Fn("foo"), 1);
                x
            "#
        )?,
        42
    );

    assert_eq!(
        engine.eval::<INT>(
            r#"
                let x = 21;
                x.bar(Fn("mul"), 2);
                x
            "#
        )?,
        42
    );

    Ok(())
}

#[test]
fn test_anonymous_fn() -> Result<(), Box<EvalAltResult>> {
    let calc_func = Func::<(INT, INT, INT), INT>::create_from_script(
        Engine::new(),
        "fn calc(x, y, z,) { (x + y) * z }",
        "calc",
    )?;

    assert_eq!(calc_func(42, 123, 9)?, 1485);

    let calc_func = Func::<(INT, String, INT), INT>::create_from_script(
        Engine::new(),
        "fn calc(x, y, z) { (x + len(y)) * z }",
        "calc",
    )?;

    assert_eq!(calc_func(42, "hello".to_string(), 9)?, 423);

    let calc_func = Func::<(INT, String, INT), INT>::create_from_script(
        Engine::new(),
        "private fn calc(x, y, z) { (x + len(y)) * z }",
        "calc",
    )?;

    assert_eq!(calc_func(42, "hello".to_string(), 9)?, 423);

    let calc_func = Func::<(INT, &str, INT), INT>::create_from_script(
        Engine::new(),
        "fn calc(x, y, z) { (x + len(y)) * z }",
        "calc",
    )?;

    assert_eq!(calc_func(42, "hello", 9)?, 423);

    Ok(())
}
