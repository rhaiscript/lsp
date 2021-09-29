use rhai::{Engine, EvalAltResult, INT};

#[cfg(not(feature = "no_float"))]
use rhai::FLOAT;

#[cfg(not(feature = "no_float"))]
const EPSILON: FLOAT = 0.000_001;

#[test]
fn test_power_of() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(engine.eval::<INT>("2 ** 3")?, 8);
    assert_eq!(engine.eval::<INT>("(-2 ** 3)")?, -8);
    assert_eq!(engine.eval::<INT>("2 ** 3 ** 2")?, 512);

    #[cfg(not(feature = "no_float"))]
    {
        assert!(
            (engine.eval::<FLOAT>("2.2 ** 3.3")? - 13.489_468_760_533_386 as FLOAT).abs()
                <= EPSILON
        );
        assert!((engine.eval::<FLOAT>("2.0**-2.0")? - 0.25 as FLOAT).abs() < EPSILON);
        assert!((engine.eval::<FLOAT>("(-2.0**-2.0)")? - 0.25 as FLOAT).abs() < EPSILON);
        assert!((engine.eval::<FLOAT>("(-2.0**-2)")? - 0.25 as FLOAT).abs() < EPSILON);
        assert_eq!(engine.eval::<INT>("4**3")?, 64);
    }

    Ok(())
}

#[test]
fn test_power_of_equals() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(engine.eval::<INT>("let x = 2; x **= 3; x")?, 8);
    assert_eq!(engine.eval::<INT>("let x = -2; x **= 3; x")?, -8);

    #[cfg(not(feature = "no_float"))]
    {
        assert!(
            (engine.eval::<FLOAT>("let x = 2.2; x **= 3.3; x")? - 13.489_468_760_533_386 as FLOAT)
                .abs()
                <= EPSILON
        );
        assert!(
            (engine.eval::<FLOAT>("let x = 2.0; x **= -2.0; x")? - 0.25 as FLOAT).abs() < EPSILON
        );
        assert!(
            (engine.eval::<FLOAT>("let x = -2.0; x **= -2.0; x")? - 0.25 as FLOAT).abs() < EPSILON
        );
        assert!(
            (engine.eval::<FLOAT>("let x = -2.0; x **= -2; x")? - 0.25 as FLOAT).abs() < EPSILON
        );
        assert_eq!(engine.eval::<INT>("let x =4; x **= 3; x")?, 64);
    }

    Ok(())
}
