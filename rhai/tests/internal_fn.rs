#![cfg(not(feature = "no_function"))]

use rhai::{Engine, EvalAltResult, ParseErrorType, INT};

#[test]
fn test_internal_fn() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(
        engine.eval::<INT>("fn add_me(a, b) { a+b } add_me(3, 4)")?,
        7
    );

    assert_eq!(
        engine.eval::<INT>("fn add_me(a, b,) { a+b } add_me(3, 4,)")?,
        7
    );

    assert_eq!(engine.eval::<INT>("fn bob() { return 4; 5 } bob()")?, 4);

    assert_eq!(engine.eval::<INT>("fn add(x, n) { x + n } add(40, 2)")?, 42);

    assert_eq!(
        engine.eval::<INT>("fn add(x, n,) { x + n } add(40, 2,)")?,
        42
    );

    assert_eq!(
        engine.eval::<INT>("fn add(x, n) { x + n } let a = 40; add(a, 2); a")?,
        40
    );

    #[cfg(not(feature = "no_object"))]
    assert_eq!(
        engine.eval::<INT>("fn add(n) { this + n } let x = 40; x.add(2)")?,
        42
    );

    #[cfg(not(feature = "no_object"))]
    assert_eq!(
        engine.eval::<INT>("fn add(n) { this += n; } let x = 40; x.add(2); x")?,
        42
    );

    assert_eq!(engine.eval::<INT>("fn mul2(x) { x * 2 } mul2(21)")?, 42);

    assert_eq!(
        engine.eval::<INT>("fn mul2(x) { x *= 2 } let a = 21; mul2(a); a")?,
        21
    );

    #[cfg(not(feature = "no_object"))]
    assert_eq!(
        engine.eval::<INT>("fn mul2() { this * 2 } let x = 21; x.mul2()")?,
        42
    );

    #[cfg(not(feature = "no_object"))]
    assert_eq!(
        engine.eval::<INT>("fn mul2() { this *= 2; } let x = 21; x.mul2(); x")?,
        42
    );

    Ok(())
}

#[test]
fn test_internal_fn_big() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(
        engine.eval::<INT>(
            "
                fn math_me(a, b, c, d, e, f) {
                    a - b * c + d * e - f
                }
                math_me(100, 5, 2, 9, 6, 32)
            ",
        )?,
        112
    );

    Ok(())
}

#[test]
fn test_internal_fn_overloading() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(
        engine.eval::<INT>(
            "
                fn abc(x,y,z) { 2*x + 3*y + 4*z + 888 }
                fn abc(x,y) { x + 2*y + 88 }
                fn abc() { 42 }
                fn abc(x) { x - 42 }

                abc() + abc(1) + abc(1,2) + abc(1,2,3)
            "
        )?,
        1002
    );

    assert_eq!(
        *engine
            .compile(
                "
                    fn abc(x) { x + 42 }
                    fn abc(x) { x - 42 }
                "
            )
            .expect_err("should error")
            .0,
        ParseErrorType::FnDuplicatedDefinition("abc".to_string(), 1)
    );

    Ok(())
}

#[test]
fn test_internal_fn_params() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    // Expect duplicated parameters error
    assert_eq!(
        *engine
            .compile("fn hello(x, x) { x }")
            .expect_err("should be error")
            .0,
        ParseErrorType::FnDuplicatedParam("hello".to_string(), "x".to_string())
    );

    Ok(())
}

#[test]
fn test_function_pointers() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(engine.eval::<String>(r#"type_of(Fn("abc"))"#)?, "Fn");

    assert_eq!(
        engine.eval::<INT>(
            r#"
                fn foo(x) { 40 + x }

                let f = Fn("foo");
                call(f, 2)
            "#
        )?,
        42
    );

    #[cfg(not(feature = "no_object"))]
    assert_eq!(
        engine.eval::<INT>(
            r#"
                fn foo(x) { 40 + x }

                let fn_name = "f";
                fn_name += "oo";

                let f = Fn(fn_name);
                f.call(2)
            "#
        )?,
        42
    );

    #[cfg(not(feature = "no_object"))]
    assert!(matches!(
        *engine.eval::<INT>(r#"let f = Fn("abc"); f.call(0)"#).expect_err("should error"),
        EvalAltResult::ErrorFunctionNotFound(f, _) if f.starts_with("abc (")
    ));

    #[cfg(not(feature = "no_object"))]
    assert_eq!(
        engine.eval::<INT>(
            r#"
                fn foo(x) { 40 + x }

                let x = #{ action: Fn("foo") };
                x.action.call(2)
            "#
        )?,
        42
    );

    #[cfg(not(feature = "no_object"))]
    assert_eq!(
        engine.eval::<INT>(
            r#"
                fn foo(x) { this.data += x; }

                let x = #{ data: 40, action: Fn("foo") };
                x.action(2);
                x.data
            "#
        )?,
        42
    );

    Ok(())
}

#[test]
#[cfg(not(feature = "no_closure"))]
fn test_internal_fn_captures() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(
        engine.eval::<INT>(
            "
                fn foo(y) { x += y; x }

                let x = 41;
                let y = 999;

                foo!(1) + x
            "
        )?,
        83
    );

    assert!(engine
        .eval::<INT>(
            "
                fn foo(y) { x += y; x }

                let x = 41;
                let y = 999;

                foo(1) + x
            "
        )
        .is_err());

    #[cfg(not(feature = "no_object"))]
    assert!(matches!(
        *engine
            .compile(
                "
                    fn foo() { this += x; }

                    let x = 41;
                    let y = 999;

                    y.foo!();
                "
            )
            .expect_err("should error")
            .0,
        ParseErrorType::MalformedCapture(_)
    ));

    Ok(())
}

#[test]
fn test_internal_fn_is_def() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert!(engine.eval::<bool>(
        r#"
            fn foo(x) { x + 1 }
            is_def_fn("foo", 1)
        "#
    )?);
    assert!(!engine.eval::<bool>(
        r#"
            fn foo(x) { x + 1 }
            is_def_fn("bar", 1)
        "#
    )?);
    assert!(!engine.eval::<bool>(
        r#"
            fn foo(x) { x + 1 }
            is_def_fn("foo", 0)
        "#
    )?);

    Ok(())
}
