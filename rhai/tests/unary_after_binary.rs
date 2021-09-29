use rhai::{Engine, EvalAltResult, INT};

#[test]
// TODO also add test case for unary after compound
// Hah, turns out unary + has a good use after all!
fn test_unary_after_binary() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(engine.eval::<INT>("10 % +4")?, 2);
    assert_eq!(engine.eval::<INT>("10 << +4")?, 160);
    assert_eq!(engine.eval::<INT>("10 >> +4")?, 0);
    assert_eq!(engine.eval::<INT>("10 & +4")?, 0);
    assert_eq!(engine.eval::<INT>("10 | +4")?, 14);
    assert_eq!(engine.eval::<INT>("10 ^ +4")?, 14);

    Ok(())
}
