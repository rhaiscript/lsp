use rhai::{Engine, EvalAltResult, INT};

#[test]
fn test_mismatched_op() {
    let engine = Engine::new();

    assert!(matches!(
        *engine.eval::<INT>(r#""hello, " + "world!""#).expect_err("expects error"),
        EvalAltResult::ErrorMismatchOutputType(need, actual, _) if need == std::any::type_name::<INT>() && actual == "string"
    ));
}

#[test]
#[cfg(not(feature = "no_object"))]
fn test_mismatched_op_custom_type() -> Result<(), Box<EvalAltResult>> {
    #[derive(Debug, Clone)]
    struct TestStruct {
        x: INT,
    }

    impl TestStruct {
        fn new() -> Self {
            Self { x: 1 }
        }
    }

    let mut engine = Engine::new();

    engine
        .register_type_with_name::<TestStruct>("TestStruct")
        .register_fn("new_ts", TestStruct::new);

    assert!(matches!(*engine.eval::<bool>("
            let x = new_ts();
            let y = new_ts();
            x == y
        ").expect_err("should error"),
        EvalAltResult::ErrorFunctionNotFound(f, _) if f == "== (TestStruct, TestStruct)"));

    assert!(!engine.eval::<bool>("new_ts() == 42")?);

    assert!(matches!(
        *engine.eval::<INT>("60 + new_ts()").expect_err("should error"),
        EvalAltResult::ErrorFunctionNotFound(f, _) if f == format!("+ ({}, TestStruct)", std::any::type_name::<INT>())
    ));

    assert!(matches!(
        *engine.eval::<TestStruct>("42").expect_err("should error"),
        EvalAltResult::ErrorMismatchOutputType(need, actual, _)
            if need == "TestStruct" && actual == std::any::type_name::<INT>()
    ));

    Ok(())
}
