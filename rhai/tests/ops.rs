use rhai::{Engine, EvalAltResult, Scope, INT};

#[test]
fn test_ops() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(engine.eval::<INT>("60 + 5")?, 65);
    assert_eq!(engine.eval::<INT>("(1 + 2) * (6 - 4) / 2")?, 3);

    Ok(())
}

#[test]
fn test_ops_numbers() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    let mut scope = Scope::new();

    scope.push("x", 42_u16);

    assert!(matches!(
        *engine.eval_with_scope::<bool>(&mut scope, "x == 42").expect_err("should error"),
        EvalAltResult::ErrorFunctionNotFound(f, _) if f.starts_with("== (u16,")
    ));
    #[cfg(not(feature = "no_float"))]
    assert!(matches!(
        *engine.eval_with_scope::<bool>(&mut scope, "x == 42.0").expect_err("should error"),
        EvalAltResult::ErrorFunctionNotFound(f, _) if f.starts_with("== (u16,")
    ));

    assert!(!engine.eval_with_scope::<bool>(&mut scope, r#"x == "hello""#)?);

    Ok(())
}

#[test]
fn test_ops_strings() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert!(engine.eval::<bool>(r#""hello" > 'c'"#)?);
    assert!(engine.eval::<bool>(r#""" < 'c'"#)?);
    assert!(engine.eval::<bool>(r#"'x' > "hello""#)?);
    assert!(engine.eval::<bool>(r#""hello" > "foo""#)?);

    Ok(())
}

#[test]
fn test_ops_precedence() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(
        engine.eval::<INT>("let x = 0; if x == 10 || true { x = 1} x")?,
        1
    );

    Ok(())
}
