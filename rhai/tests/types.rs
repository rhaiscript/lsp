use rhai::{Engine, EvalAltResult, INT};

#[test]
fn test_type_of() -> Result<(), Box<EvalAltResult>> {
    #[derive(Clone)]
    struct TestStruct {
        x: INT,
    }

    let mut engine = Engine::new();

    #[cfg(not(feature = "only_i32"))]
    assert_eq!(engine.eval::<String>("type_of(60 + 5)")?, "i64");

    #[cfg(feature = "only_i32")]
    assert_eq!(engine.eval::<String>("type_of(60 + 5)")?, "i32");

    #[cfg(not(feature = "no_float"))]
    {
        #[cfg(not(feature = "f32_float"))]
        assert_eq!(engine.eval::<String>("type_of(1.0 + 2.0)")?, "f64");

        #[cfg(feature = "f32_float")]
        assert_eq!(engine.eval::<String>("type_of(1.0 + 2.0)")?, "f32");
    }

    #[cfg(not(feature = "no_index"))]
    assert_eq!(
        engine.eval::<String>(r#"type_of([true, 2, "hello"])"#)?,
        "array"
    );

    #[cfg(not(feature = "no_object"))]
    assert_eq!(
        engine.eval::<String>(r#"type_of(#{a:true, "":2, "z":"hello"})"#)?,
        "map"
    );

    #[cfg(not(feature = "no_object"))]
    {
        engine.register_type_with_name::<TestStruct>("Hello");
        engine.register_fn("new_ts", || TestStruct { x: 1 });

        assert_eq!(engine.eval::<String>("type_of(new_ts())")?, "Hello");
    }

    assert_eq!(engine.eval::<String>(r#"type_of("hello")"#)?, "string");

    #[cfg(not(feature = "no_object"))]
    assert_eq!(engine.eval::<String>(r#""hello".type_of()"#)?, "string");

    #[cfg(not(feature = "only_i32"))]
    assert_eq!(engine.eval::<String>("let x = 123; type_of(x)")?, "i64");

    #[cfg(feature = "only_i32")]
    assert_eq!(engine.eval::<String>("let x = 123; type_of(x)")?, "i32");

    Ok(())
}
