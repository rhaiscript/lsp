use rhai::{Engine, EvalAltResult, Position, Scope, INT};

#[test]
fn test_var_scope() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();
    let mut scope = Scope::new();

    engine.eval_with_scope::<()>(&mut scope, "let x = 4 + 5")?;
    assert_eq!(engine.eval_with_scope::<INT>(&mut scope, "x")?, 9);
    engine.eval_with_scope::<()>(&mut scope, "x += 1; x += 2;")?;
    assert_eq!(engine.eval_with_scope::<INT>(&mut scope, "x")?, 12);

    scope.set_value("x", 42 as INT);
    assert_eq!(engine.eval_with_scope::<INT>(&mut scope, "x")?, 42);

    engine.eval_with_scope::<()>(&mut scope, "{let x = 3}")?;
    assert_eq!(engine.eval_with_scope::<INT>(&mut scope, "x")?, 42);

    Ok(())
}

#[test]
fn test_var_is_def() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert!(engine.eval::<bool>(
        r#"
            let x = 42;
            is_def_var("x")
        "#
    )?);
    assert!(!engine.eval::<bool>(
        r#"
            let x = 42;
            is_def_var("y")
        "#
    )?);
    assert!(engine.eval::<bool>(
        r#"
            const x = 42;
            is_def_var("x")
        "#
    )?);

    Ok(())
}

#[test]
fn test_scope_eval() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    // First create the state
    let mut scope = Scope::new();

    // Then push some initialized variables into the state
    // NOTE: Remember the default numbers used by Rhai are INT and f64.
    //       Better stick to them or it gets hard to work with other variables in the script.
    scope.push("y", 42 as INT);
    scope.push("z", 999 as INT);

    // First invocation
    engine
        .eval_with_scope::<()>(&mut scope, " let x = 4 + 5 - y + z; y = 1;")
        .expect("variables y and z should exist");

    // Second invocation using the same state
    let result = engine.eval_with_scope::<INT>(&mut scope, "x")?;

    println!("result: {}", result); // should print 966

    // Variable y is changed in the script
    assert_eq!(
        scope
            .get_value::<INT>("y")
            .expect("variable y should exist"),
        1
    );

    Ok(())
}

#[test]
fn test_var_resolver() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    let mut scope = Scope::new();
    scope.push("innocent", 1 as INT);
    scope.push("chameleon", 123 as INT);
    scope.push("DO_NOT_USE", 999 as INT);

    engine.on_var(|name, _, context| {
        match name {
            "MYSTIC_NUMBER" => Ok(Some((42 as INT).into())),
            // Override a variable - make it not found even if it exists!
            "DO_NOT_USE" => {
                Err(EvalAltResult::ErrorVariableNotFound(name.to_string(), Position::NONE).into())
            }
            // Silently maps 'chameleon' into 'innocent'.
            "chameleon" => context
                .scope()
                .get_value("innocent")
                .map(Some)
                .ok_or_else(|| {
                    EvalAltResult::ErrorVariableNotFound(name.to_string(), Position::NONE).into()
                }),
            // Return Ok(None) to continue with the normal variable resolution process.
            _ => Ok(None),
        }
    });

    assert_eq!(
        engine.eval_with_scope::<INT>(&mut scope, "MYSTIC_NUMBER")?,
        42
    );
    assert_eq!(engine.eval_with_scope::<INT>(&mut scope, "chameleon")?, 1);
    assert!(
        matches!(*engine.eval_with_scope::<INT>(&mut scope, "DO_NOT_USE").expect_err("should error"),
        EvalAltResult::ErrorVariableNotFound(n, _) if n == "DO_NOT_USE")
    );

    Ok(())
}
