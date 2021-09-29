use rhai::{Engine, EvalAltResult, INT};

#[test]
fn test_comments() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(
        engine.eval::<INT>("let x = 42; x // I am a single line comment, yay!")?,
        42
    );

    assert_eq!(
        engine.eval::<INT>(
            "
                let /* I am a
                    multi-line
                        comment, yay!
                    */ x = 42; x
            "
        )?,
        42
    );

    assert_eq!(engine.eval::<()>("/* Hello world */")?, ());

    Ok(())
}

#[cfg(not(feature = "no_function"))]
#[cfg(feature = "metadata")]
#[test]
fn test_comments_doc() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    let ast = engine.compile(
        "
            /// Hello world


            fn foo() {}
        ",
    )?;

    assert_eq!(
        ast.iter_functions().next().unwrap().comments[0],
        "/// Hello world"
    );

    assert!(engine
        .compile(
            "
                /// Hello world
                let x = 42;
            "
        )
        .is_err());

    engine.compile(
        "
            ///////////////
            let x = 42;

            /***************/
            let x = 42;
        ",
    )?;

    let ast = engine.compile(
        "
            /** Hello world
            ** how are you?
            **/

            fn foo() {}
        ",
    )?;

    assert_eq!(
        ast.iter_functions().next().unwrap().comments[0],
        "/** Hello world\n            ** how are you?\n            **/"
    );

    assert!(engine
        .compile(
            "
                /** Hello world */
                let x = 42;
            "
        )
        .is_err());

    Ok(())
}
