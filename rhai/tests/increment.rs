use rhai::{Engine, EvalAltResult, INT};

#[test]
fn test_increment() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(engine.eval::<INT>("let x = 1; x += 2; x")?, 3);

    assert_eq!(
        engine.eval::<String>(r#"let s = "test"; s += "ing"; s"#)?,
        "testing"
    );

    Ok(())
}
