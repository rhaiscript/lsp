#![cfg(not(feature = "no_optimize"))]

use rhai::{Engine, EvalAltResult, OptimizationLevel, INT};

#[test]
fn test_optimizer() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    engine.set_optimization_level(OptimizationLevel::Full);

    #[cfg(not(feature = "no_function"))]
    assert_eq!(
        engine.eval::<INT>(
            "
                fn foo(x) { print(x); return; }
                fn foo2(x) { if x > 0 {} return; }
                42
            "
        )?,
        42
    );

    Ok(())
}

#[test]
fn test_optimizer_run() -> Result<(), Box<EvalAltResult>> {
    fn run_test(engine: &mut Engine) -> Result<(), Box<EvalAltResult>> {
        assert_eq!(engine.eval::<INT>("if true { 42 } else { 123 }")?, 42);
        assert_eq!(
            engine.eval::<INT>("if 1 == 1 || 2 > 3 { 42 } else { 123 }")?,
            42
        );
        assert_eq!(
            engine.eval::<INT>(r#"const abc = "hello"; if abc < "foo" { 42 } else { 123 }"#)?,
            123
        );
        Ok(())
    }

    let mut engine = Engine::new();

    engine.set_optimization_level(OptimizationLevel::None);
    run_test(&mut engine)?;

    engine.set_optimization_level(OptimizationLevel::Simple);
    run_test(&mut engine)?;

    engine.set_optimization_level(OptimizationLevel::Full);
    run_test(&mut engine)?;

    // Override == operator
    engine.register_fn("==", |_x: INT, _y: INT| false);

    engine.set_optimization_level(OptimizationLevel::Simple);

    assert_eq!(
        engine.eval::<INT>("if 1 == 1 || 2 > 3 { 42 } else { 123 }")?,
        123
    );

    engine.set_optimization_level(OptimizationLevel::Full);

    assert_eq!(
        engine.eval::<INT>("if 1 == 1 || 2 > 3 { 42 } else { 123 }")?,
        123
    );

    Ok(())
}

#[cfg(not(feature = "no_module"))]
#[cfg(not(feature = "no_position"))]
#[test]
fn test_optimizer_parse() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    engine.set_optimization_level(OptimizationLevel::Simple);

    let ast = engine.compile("{ const DECISION = false; if DECISION { 42 } else { 123 } }")?;

    assert_eq!(
        format!("{:?}", ast),
        "AST { source: None, body: Block[Expr(123 @ 1:53)], functions: Module, resolver: None }"
    );

    let ast = engine.compile("const DECISION = false; if DECISION { 42 } else { 123 }")?;

    assert_eq!(
        format!("{:?}", ast),
        r#"AST { source: None, body: Block[Var(false @ 1:18, "DECISION" @ 1:7, (Constant), 1:1), Expr(123 @ 1:51)], functions: Module, resolver: None }"#
    );

    let ast = engine.compile("if 1 == 2 { 42 }")?;

    assert_eq!(
        format!("{:?}", ast),
        "AST { source: None, body: Block[], functions: Module, resolver: None }"
    );

    engine.set_optimization_level(OptimizationLevel::Full);

    let ast = engine.compile("abs(-42)")?;

    assert_eq!(
        format!("{:?}", ast),
        "AST { source: None, body: Block[Expr(42 @ 1:1)], functions: Module, resolver: None }"
    );

    Ok(())
}
