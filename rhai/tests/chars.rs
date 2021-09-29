use rhai::{Engine, EvalAltResult};

#[test]
fn test_chars() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(engine.eval::<char>("'y'")?, 'y');
    assert_eq!(engine.eval::<char>(r"'\''")?, '\'');
    assert_eq!(engine.eval::<char>(r#"'"'"#)?, '"');
    assert_eq!(engine.eval::<char>(r"'\u2764'")?, '❤');

    #[cfg(not(feature = "no_index"))]
    {
        assert_eq!(engine.eval::<char>(r#"let x="hello"; x[2]"#)?, 'l');
        assert_eq!(
            engine.eval::<String>(r#"let y="hello"; y[2]='$'; y"#)?,
            "he$lo"
        );
    }

    assert!(engine.eval::<char>(r"'\uhello'").is_err());
    assert!(engine.eval::<char>("''").is_err());

    Ok(())
}
