use rhai::{Engine, EvalAltResult, INT};

#[test]
fn test_binary_ops() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(engine.eval::<INT>("10 + 4")?, 14);
    assert_eq!(engine.eval::<INT>("10 - 4")?, 6);
    assert_eq!(engine.eval::<INT>("10 * 4")?, 40);
    assert_eq!(engine.eval::<INT>("10 / 4")?, 2);
    assert_eq!(engine.eval::<INT>("10 % 4")?, 2);
    assert_eq!(engine.eval::<INT>("10 ** 4")?, 10000);
    assert_eq!(engine.eval::<INT>("10 << 4")?, 160);
    assert_eq!(engine.eval::<INT>("10 >> 4")?, 0);
    assert_eq!(engine.eval::<INT>("10 & 4")?, 0);
    assert_eq!(engine.eval::<INT>("10 | 4")?, 14);
    assert_eq!(engine.eval::<INT>("10 ^ 4")?, 14);

    assert!(engine.eval::<bool>("42 == 42")?);
    assert!(!engine.eval::<bool>("42 != 42")?);
    assert!(!engine.eval::<bool>("42 > 42")?);
    assert!(engine.eval::<bool>("42 >= 42")?);
    assert!(!engine.eval::<bool>("42 < 42")?);
    assert!(engine.eval::<bool>("42 <= 42")?);

    assert_eq!(engine.eval::<INT>("let x = 10; x += 4; x")?, 14);
    assert_eq!(engine.eval::<INT>("let x = 10; x -= 4; x")?, 6);
    assert_eq!(engine.eval::<INT>("let x = 10; x *= 4; x")?, 40);
    assert_eq!(engine.eval::<INT>("let x = 10; x /= 4; x")?, 2);
    assert_eq!(engine.eval::<INT>("let x = 10; x %= 4; x")?, 2);
    assert_eq!(engine.eval::<INT>("let x = 10; x **= 4; x")?, 10000);
    assert_eq!(engine.eval::<INT>("let x = 10; x <<= 4; x")?, 160);
    assert_eq!(engine.eval::<INT>("let x = 10; x >>= 4; x")?, 0);
    assert_eq!(engine.eval::<INT>("let x = 10; x &= 4; x")?, 0);
    assert_eq!(engine.eval::<INT>("let x = 10; x |= 4; x")?, 14);
    assert_eq!(engine.eval::<INT>("let x = 10; x ^= 4; x")?, 14);

    #[cfg(not(feature = "no_float"))]
    {
        use rhai::FLOAT;

        assert_eq!(engine.eval::<FLOAT>("10.0 + 4.0")?, 14.0);
        assert_eq!(engine.eval::<FLOAT>("10.0 - 4.0")?, 6.0);
        assert_eq!(engine.eval::<FLOAT>("10.0 * 4.0")?, 40.0);
        assert_eq!(engine.eval::<FLOAT>("10.0 / 4.0")?, 2.5);
        assert_eq!(engine.eval::<FLOAT>("10.0 % 4.0")?, 2.0);
        assert_eq!(engine.eval::<FLOAT>("10.0 ** 4.0")?, 10000.0);

        assert_eq!(engine.eval::<FLOAT>("10.0 + 4")?, 14.0);
        assert_eq!(engine.eval::<FLOAT>("10.0 - 4")?, 6.0);
        assert_eq!(engine.eval::<FLOAT>("10.0 * 4")?, 40.0);
        assert_eq!(engine.eval::<FLOAT>("10.0 / 4")?, 2.5);
        assert_eq!(engine.eval::<FLOAT>("10.0 % 4")?, 2.0);
        assert_eq!(engine.eval::<FLOAT>("10.0 ** 4")?, 10000.0);

        assert_eq!(engine.eval::<FLOAT>("10 + 4.0")?, 14.0);
        assert_eq!(engine.eval::<FLOAT>("10 - 4.0")?, 6.0);
        assert_eq!(engine.eval::<FLOAT>("10 * 4.0")?, 40.0);
        assert_eq!(engine.eval::<FLOAT>("10 / 4.0")?, 2.5);
        assert_eq!(engine.eval::<FLOAT>("10 % 4.0")?, 2.0);
        assert_eq!(engine.eval::<FLOAT>("10 ** 4.0")?, 10000.0);

        assert!(engine.eval::<bool>("42 == 42.0")?);
        assert!(!engine.eval::<bool>("42 != 42.0")?);
        assert!(!engine.eval::<bool>("42 > 42.0")?);
        assert!(engine.eval::<bool>("42 >= 42.0")?);
        assert!(!engine.eval::<bool>("42 < 42.0")?);
        assert!(engine.eval::<bool>("42 <= 42.0")?);

        assert!(engine.eval::<bool>("42.0 == 42")?);
        assert!(!engine.eval::<bool>("42.0 != 42")?);
        assert!(!engine.eval::<bool>("42.0 > 42")?);
        assert!(engine.eval::<bool>("42.0 >= 42")?);
        assert!(!engine.eval::<bool>("42.0 < 42")?);
        assert!(engine.eval::<bool>("42.0 <= 42")?);

        assert_eq!(engine.eval::<FLOAT>("let x = 10.0; x += 4.0; x")?, 14.0);
        assert_eq!(engine.eval::<FLOAT>("let x = 10.0; x -= 4.0; x")?, 6.0);
        assert_eq!(engine.eval::<FLOAT>("let x = 10.0; x *= 4.0; x")?, 40.0);
        assert_eq!(engine.eval::<FLOAT>("let x = 10.0; x /= 4.0; x")?, 2.5);
        assert_eq!(engine.eval::<FLOAT>("let x = 10.0; x %= 4.0; x")?, 2.0);
        assert_eq!(engine.eval::<FLOAT>("let x = 10.0; x **= 4.0; x")?, 10000.0);

        assert_eq!(engine.eval::<FLOAT>("let x = 10.0; x += 4; x")?, 14.0);
        assert_eq!(engine.eval::<FLOAT>("let x = 10.0; x -= 4; x")?, 6.0);
        assert_eq!(engine.eval::<FLOAT>("let x = 10.0; x *= 4; x")?, 40.0);
        assert_eq!(engine.eval::<FLOAT>("let x = 10.0; x /= 4; x")?, 2.5);
        assert_eq!(engine.eval::<FLOAT>("let x = 10.0; x %= 4; x")?, 2.0);
        assert_eq!(engine.eval::<FLOAT>("let x = 10.0; x **= 4; x")?, 10000.0);
    }

    assert_eq!(
        engine.eval::<String>(r#""hello" + ", world""#)?,
        "hello, world"
    );
    assert_eq!(engine.eval::<String>(r#""hello" + '!'"#)?, "hello!");
    assert_eq!(engine.eval::<String>(r#""hello" - "el""#)?, "hlo");
    assert_eq!(engine.eval::<String>(r#""hello" - 'l'"#)?, "heo");

    assert!(!engine.eval::<bool>(r#""a" == "x""#)?);
    assert!(engine.eval::<bool>(r#""a" != "x""#)?);
    assert!(!engine.eval::<bool>(r#""a" > "x""#)?);
    assert!(!engine.eval::<bool>(r#""a" >= "x""#)?);
    assert!(engine.eval::<bool>(r#""a" < "x""#)?);
    assert!(engine.eval::<bool>(r#""a" <= "x""#)?);

    assert!(engine.eval::<bool>(r#""x" == 'x'"#)?);
    assert!(!engine.eval::<bool>(r#""x" != 'x'"#)?);
    assert!(!engine.eval::<bool>(r#""x" > 'x'"#)?);
    assert!(engine.eval::<bool>(r#""x" >= 'x'"#)?);
    assert!(!engine.eval::<bool>(r#""x" < 'x'"#)?);
    assert!(engine.eval::<bool>(r#""x" <= 'x'"#)?);

    assert!(engine.eval::<bool>(r#"'x' == "x""#)?);
    assert!(!engine.eval::<bool>(r#"'x' != "x""#)?);
    assert!(!engine.eval::<bool>(r#"'x' > "x""#)?);
    assert!(engine.eval::<bool>(r#"'x' >= "x""#)?);
    assert!(!engine.eval::<bool>(r#"'x' < "x""#)?);
    assert!(engine.eval::<bool>(r#"'x' <= "x""#)?);

    // Incompatible types compare to false
    assert!(!engine.eval::<bool>("true == 42")?);
    assert!(engine.eval::<bool>("true != 42")?);
    assert!(!engine.eval::<bool>("true > 42")?);
    assert!(!engine.eval::<bool>("true >= 42")?);
    assert!(!engine.eval::<bool>("true < 42")?);
    assert!(!engine.eval::<bool>("true <= 42")?);

    assert!(!engine.eval::<bool>(r#""42" == 42"#)?);
    assert!(engine.eval::<bool>(r#""42" != 42"#)?);
    assert!(!engine.eval::<bool>(r#""42" > 42"#)?);
    assert!(!engine.eval::<bool>(r#""42" >= 42"#)?);
    assert!(!engine.eval::<bool>(r#""42" < 42"#)?);
    assert!(!engine.eval::<bool>(r#""42" <= 42"#)?);

    Ok(())
}
