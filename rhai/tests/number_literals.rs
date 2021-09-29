use rhai::{Engine, EvalAltResult, INT};

#[test]
fn test_number_literal() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(engine.eval::<INT>("42")?, 42);

    #[cfg(not(feature = "no_object"))]
    assert_eq!(
        engine.eval::<String>("42.type_of()")?,
        if cfg!(feature = "only_i32") {
            "i32"
        } else {
            "i64"
        }
    );

    Ok(())
}

#[test]
fn test_hex_literal() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(engine.eval::<INT>("let x = 0xf; x")?, 15);
    assert_eq!(engine.eval::<INT>("let x = 0Xf; x")?, 15);
    assert_eq!(engine.eval::<INT>("let x = 0xff; x")?, 255);

    Ok(())
}

#[test]
fn test_octal_literal() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(engine.eval::<INT>("let x = 0o77; x")?, 63);
    assert_eq!(engine.eval::<INT>("let x = 0O77; x")?, 63);
    assert_eq!(engine.eval::<INT>("let x = 0o1234; x")?, 668);

    Ok(())
}

#[test]
fn test_binary_literal() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(engine.eval::<INT>("let x = 0b1111; x")?, 15);
    assert_eq!(engine.eval::<INT>("let x = 0B1111; x")?, 15);
    assert_eq!(
        engine.eval::<INT>("let x = 0b0011_1100_1010_0101; x")?,
        15525
    );

    Ok(())
}
