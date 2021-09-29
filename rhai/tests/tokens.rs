use rhai::{Engine, EvalAltResult, LexError, ParseErrorType, INT};

#[test]
fn test_tokens_disabled() {
    let mut engine = Engine::new();

    engine.disable_symbol("if"); // disable the 'if' keyword

    assert!(matches!(
        *engine
            .compile("let x = if true { 42 } else { 0 };")
            .expect_err("should error")
            .0,
        ParseErrorType::Reserved(err) if err == "if"
    ));

    engine.disable_symbol("+="); // disable the '+=' operator

    assert_eq!(
        *engine
            .compile("let x = 40 + 2; x += 1;")
            .expect_err("should error")
            .0,
        ParseErrorType::UnknownOperator("+=".to_string())
    );

    assert!(matches!(
        *engine.compile("let x = += 0;").expect_err("should error").0,
        ParseErrorType::BadInput(LexError::UnexpectedInput(err)) if err == "+="
    ));
}

#[test]
fn test_tokens_custom_operator_identifiers() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    // Register a custom operator called `foo` and give it
    // a precedence of 160 (i.e. between +|- and *|/).
    engine.register_custom_operator("foo", 160).unwrap();

    // Register a binary function named `foo`
    engine.register_fn("foo", |x: INT, y: INT| (x * y) - (x + y));

    assert_eq!(
        engine.eval_expression::<INT>("1 + 2 * 3 foo 4 - 5 / 6")?,
        15
    );

    #[cfg(not(feature = "no_function"))]
    assert_eq!(
        engine.eval::<INT>(
            "
                fn foo(x, y) { y - x }
                1 + 2 * 3 foo 4 - 5 / 6
            "
        )?,
        -1
    );

    Ok(())
}

#[test]
fn test_tokens_custom_operator_symbol() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    // Register a custom operator `#` and give it
    // a precedence of 160 (i.e. between +|- and *|/).
    engine.register_custom_operator("#", 160).unwrap();

    // Register a binary function named `#`
    engine.register_fn("#", |x: INT, y: INT| (x * y) - (x + y));

    assert_eq!(engine.eval_expression::<INT>("1 + 2 * 3 # 4 - 5 / 6")?, 15);

    // Register a custom operator named `=>`
    assert!(engine.register_custom_operator("=>", 160).is_err());
    engine.disable_symbol("=>");
    engine.register_custom_operator("=>", 160).unwrap();
    engine.register_fn("=>", |x: INT, y: INT| (x * y) - (x + y));
    assert_eq!(engine.eval_expression::<INT>("1 + 2 * 3 => 4 - 5 / 6")?, 15);

    Ok(())
}

#[test]
fn test_tokens_unicode_xid_ident() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();
    let result = engine.eval::<INT>(
        "
            fn すべての答え() { 42 }
            すべての答え()
        ",
    );
    #[cfg(feature = "unicode-xid-ident")]
    assert_eq!(result?, 42);

    #[cfg(not(feature = "unicode-xid-ident"))]
    assert!(result.is_err());

    let result = engine.eval::<INT>(
        "
            fn _1() { 1 }
            _1()
        ",
    );
    assert!(result.is_err());

    Ok(())
}
