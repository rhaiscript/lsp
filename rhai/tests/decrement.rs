use rhai::{Engine, EvalAltResult, INT};

#[test]
fn test_decrement() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(engine.eval::<INT>("let x = 10; x -= 7; x")?, 3);

    assert_eq!(
        engine.eval::<String>(r#"let s = "test"; s -= 's'; s"#)?,
        "tet"
    );

    Ok(())
}
