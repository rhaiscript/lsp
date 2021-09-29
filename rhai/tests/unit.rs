use rhai::{Engine, EvalAltResult};

#[test]
fn test_unit() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();
    engine.eval::<()>("let x = (); x")?;
    Ok(())
}

#[test]
fn test_unit_eq() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();
    assert_eq!(engine.eval::<bool>("let x = (); let y = (); x == y")?, true);
    Ok(())
}

#[test]
fn test_unit_with_spaces() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();
    engine.eval::<()>("let x = ( ); x")?;
    Ok(())
}
