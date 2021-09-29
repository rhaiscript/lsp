use rhai::{Engine, EvalAltResult, INT};

#[test]
fn test_unary_minus() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(engine.eval::<INT>("let x = -5; x")?, -5);

    #[cfg(not(feature = "no_function"))]
    assert_eq!(engine.eval::<INT>("fn neg(x) { -x } neg(5)")?, -5);

    assert_eq!(engine.eval::<INT>("5 - -+ + + - -+-5")?, 0);

    Ok(())
}
