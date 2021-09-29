#![cfg(not(feature = "unchecked"))]
use rhai::{Engine, EvalAltResult, ParseErrorType};

#[cfg(not(feature = "no_index"))]
use rhai::Array;

#[cfg(not(feature = "no_object"))]
use rhai::Map;

#[test]
fn test_max_string_size() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    engine.set_max_string_size(10);

    assert_eq!(
        *engine
            .compile(r#"let x = "hello, world!";"#)
            .expect_err("should error")
            .0,
        ParseErrorType::LiteralTooLarge("Length of string literal".to_string(), 10)
    );

    assert_eq!(
        *engine
            .compile(r#"let x = "朝に紅顔、暮に白骨";"#)
            .expect_err("should error")
            .0,
        ParseErrorType::LiteralTooLarge("Length of string literal".to_string(), 10)
    );

    assert!(matches!(
        *engine
            .eval::<String>(
                r#"
                    let x = "hello, ";
                    let y = "world!";
                    x + y
                "#
            )
            .expect_err("should error"),
        EvalAltResult::ErrorDataTooLarge(_, _)
    ));

    #[cfg(not(feature = "no_object"))]
    assert!(matches!(
        *engine
            .eval::<String>(
                r#"
                    let x = "hello";
                    x.pad(100, '!');
                    x
                "#
            )
            .expect_err("should error"),
        EvalAltResult::ErrorDataTooLarge(_, _)
    ));

    engine.set_max_string_size(0);

    assert_eq!(
        engine.eval::<String>(
            r#"
                let x = "hello, ";
                let y = "world!";
                x + y
            "#
        )?,
        "hello, world!"
    );

    Ok(())
}

#[test]
#[cfg(not(feature = "no_index"))]
fn test_max_array_size() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    engine.set_max_array_size(10);

    #[cfg(not(feature = "no_object"))]
    engine.set_max_map_size(10);

    assert_eq!(
        *engine
            .compile("let x = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15];")
            .expect_err("should error")
            .0,
        ParseErrorType::LiteralTooLarge("Size of array literal".to_string(), 10)
    );

    assert!(matches!(
        *engine
            .eval::<Array>(
                "
                    let x = [1,2,3,4,5,6];
                    let y = [7,8,9,10,11,12];
                    x + y
                "
            )
            .expect_err("should error"),
        EvalAltResult::ErrorDataTooLarge(_, _)
    ));

    #[cfg(not(feature = "no_object"))]
    assert!(matches!(
        *engine
            .eval::<Array>(
                "
                    let x = [1,2,3,4,5,6];
                    x.pad(100, 42);
                    x
                "
            )
            .expect_err("should error"),
        EvalAltResult::ErrorDataTooLarge(_, _)
    ));

    assert!(matches!(
        *engine
            .eval::<Array>(
                "
                    let x = [1,2,3];
                    [x, x, x, x]
                "
            )
            .expect_err("should error"),
        EvalAltResult::ErrorDataTooLarge(_, _)
    ));

    #[cfg(not(feature = "no_object"))]
    assert!(matches!(
        *engine
            .eval::<Array>(
                "
                    let x = #{a:1, b:2, c:3};
                    [x, x, x, x]
                "
            )
            .expect_err("should error"),
        EvalAltResult::ErrorDataTooLarge(_, _)
    ));

    assert!(matches!(
        *engine
            .eval::<Array>(
                "
                    let x = [1];
                    let y = [x, x];
                    let z = [y, y];
                    [z, z, z]
                "
            )
            .expect_err("should error"),
        EvalAltResult::ErrorDataTooLarge(_, _)
    ));

    engine.set_max_array_size(0);

    assert_eq!(
        engine
            .eval::<Array>(
                "
                    let x = [1,2,3,4,5,6];
                    let y = [7,8,9,10,11,12];
                    x + y
                "
            )?
            .len(),
        12
    );

    assert_eq!(
        engine
            .eval::<Array>(
                "
                    let x = [1,2,3];
                    [x, x, x, x]
                "
            )?
            .len(),
        4
    );

    Ok(())
}

#[test]
#[cfg(not(feature = "no_object"))]
fn test_max_map_size() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    engine.set_max_map_size(10);

    #[cfg(not(feature = "no_index"))]
    engine.set_max_array_size(10);

    assert_eq!(
        *engine
            .compile(
                "let x = #{a:1,b:2,c:3,d:4,e:5,f:6,g:7,h:8,i:9,j:10,k:11,l:12,m:13,n:14,o:15};"
            )
            .expect_err("should error")
            .0,
        ParseErrorType::LiteralTooLarge(
            "Number of properties in object map literal".to_string(),
            10
        )
    );

    assert!(matches!(
        *engine
            .run(
                "
                    let x = #{};
                    loop { x.a = x; }
                "
            )
            .expect_err("should error"),
        EvalAltResult::ErrorDataTooLarge(_, _)
    ));

    assert!(matches!(
        *engine
            .eval::<Map>(
                "
                    let x = #{a:1,b:2,c:3,d:4,e:5,f:6};
                    let y = #{g:7,h:8,i:9,j:10,k:11,l:12};
                    x + y
                "
            )
            .expect_err("should error"),
        EvalAltResult::ErrorDataTooLarge(_, _)
    ));

    assert!(matches!(
        *engine
            .eval::<Map>(
                "
                    let x = #{a:1,b:2,c:3};
                    #{u:x, v:x, w:x, z:x}
                "
            )
            .expect_err("should error"),
        EvalAltResult::ErrorDataTooLarge(_, _)
    ));

    #[cfg(not(feature = "no_index"))]
    assert!(matches!(
        *engine
            .eval::<Map>(
                "
                    let x = [1, 2, 3];
                    #{u:x, v:x, w:x, z:x}
                "
            )
            .expect_err("should error"),
        EvalAltResult::ErrorDataTooLarge(_, _)
    ));

    engine.set_max_map_size(0);

    assert_eq!(
        engine
            .eval::<Map>(
                "
                    let x = #{a:1,b:2,c:3,d:4,e:5,f:6};
                    let y = #{g:7,h:8,i:9,j:10,k:11,l:12};
                    x + y
                "
            )?
            .len(),
        12
    );

    assert_eq!(
        engine
            .eval::<Map>(
                "
                    let x = #{a:1,b:2,c:3};
                    #{u:x, v:x, w:x, z:x}
                "
            )?
            .len(),
        4
    );

    Ok(())
}
