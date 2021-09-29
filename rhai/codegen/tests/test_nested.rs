use rhai::{Array, Engine, EvalAltResult, FLOAT};

pub mod one_fn_module_nested_attr {
    use rhai::plugin::*;

    #[export_module]
    pub mod advanced_math {
        use rhai::plugin::*;
        use rhai::FLOAT;

        #[rhai_fn(return_raw)]
        pub fn get_mystic_number() -> Result<FLOAT, Box<EvalAltResult>> {
            Ok(42.0)
        }
    }
}

#[test]
fn one_fn_module_nested_attr_test() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    let m = rhai::exported_module!(crate::one_fn_module_nested_attr::advanced_math);
    engine.register_static_module("Math::Advanced", m.into());

    assert_eq!(
        engine.eval::<FLOAT>(r#"let m = Math::Advanced::get_mystic_number(); m"#)?,
        42.0
    );
    Ok(())
}

pub mod one_fn_sub_module_nested_attr {
    use rhai::plugin::*;

    #[export_module]
    pub mod advanced_math {
        #[rhai_mod(name = "constants")]
        pub mod my_module {
            use rhai::plugin::*;
            use rhai::FLOAT;
            #[rhai_fn(return_raw)]
            pub fn get_mystic_number() -> Result<FLOAT, Box<EvalAltResult>> {
                Ok(42.0)
            }
        }
    }
}

#[test]
fn one_fn_sub_module_nested_attr_test() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    let m = rhai::exported_module!(crate::one_fn_sub_module_nested_attr::advanced_math);
    engine.register_static_module("Math::Advanced", m.into());

    assert_eq!(
        engine.eval::<FLOAT>(r#"let m = Math::Advanced::constants::get_mystic_number(); m"#)?,
        42.0
    );
    Ok(())
}

mod export_nested_by_prefix {
    use rhai::plugin::*;

    #[export_module(export_prefix = "foo_")]
    pub mod my_adds {
        pub mod foo_first_adders {
            use rhai::{FLOAT, INT};

            pub fn add_float(f1: FLOAT, f2: FLOAT) -> FLOAT {
                f1 + f2
            }

            pub fn add_int(i1: INT, i2: INT) -> INT {
                i1 + i2
            }
        }

        pub mod foo_second_adders {
            use rhai::{FLOAT, INT};

            pub fn add_float(f1: FLOAT, f2: FLOAT) -> FLOAT {
                f1 + f2
            }

            pub fn add_int(i1: INT, i2: INT) -> INT {
                i1 + i2
            }
        }

        #[rhai_mod(name = "foo_third_adders")]
        pub mod baz_third_adders {
            use rhai::{FLOAT, INT};

            pub fn add_float(f1: FLOAT, f2: FLOAT) -> FLOAT {
                f1 + f2
            }

            pub fn add_int(i1: INT, i2: INT) -> INT {
                i1 + i2
            }
        }

        pub mod bar_fourth_adders {
            use rhai::{FLOAT, INT};

            pub fn add_float(f1: FLOAT, f2: FLOAT) -> FLOAT {
                f1 + f2
            }

            pub fn add_int(i1: INT, i2: INT) -> INT {
                i1 + i2
            }
        }
    }
}

#[test]
fn export_nested_by_prefix_test() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    let m = rhai::exported_module!(crate::export_nested_by_prefix::my_adds);
    engine.register_static_module("Math::Advanced", m.into());

    let output_array = engine.eval::<Array>(
        r#"
        let ex = 41.0;
        let fx = Math::Advanced::foo_first_adders::add_float(ex, 1.0);

        let ei = 41;
        let fi = Math::Advanced::foo_first_adders::add_int(ei, 1);

        let gx = 41.0;
        let hx = Math::Advanced::foo_second_adders::add_float(gx, 1.0);

        let gi = 41;
        let hi = Math::Advanced::foo_second_adders::add_int(gi, 1);

        [fx, hx, fi, hi]
        "#,
    )?;
    assert_eq!(&output_array[0].as_float().unwrap(), &42.0);
    assert_eq!(&output_array[1].as_float().unwrap(), &42.0);
    assert_eq!(&output_array[2].as_int().unwrap(), &42);
    assert_eq!(&output_array[3].as_int().unwrap(), &42);

    assert!(matches!(*engine.eval::<FLOAT>(
        r#"
        let ex = 41.0;
        let fx = Math::Advanced::foo_third_adders::add_float(ex, 1.0);
        fx
        "#).unwrap_err(),
        EvalAltResult::ErrorFunctionNotFound(s, p)
            if s == "Math::Advanced::foo_third_adders::add_float (f64, f64)"
            && p == rhai::Position::new(3, 52)));

    assert!(matches!(*engine.eval::<FLOAT>(
        r#"
        let ex = 41;
        let fx = Math::Advanced::foo_third_adders::add_int(ex, 1);
        fx
        "#).unwrap_err(),
        EvalAltResult::ErrorFunctionNotFound(s, p)
            if s == "Math::Advanced::foo_third_adders::add_int (i64, i64)"
            && p == rhai::Position::new(3, 52)));

    assert!(matches!(*engine.eval::<FLOAT>(
        r#"
        let ex = 41;
        let fx = Math::Advanced::bar_fourth_adders::add_int(ex, 1);
        fx
        "#).unwrap_err(),
        EvalAltResult::ErrorFunctionNotFound(s, p)
            if s == "Math::Advanced::bar_fourth_adders::add_int (i64, i64)"
            && p == rhai::Position::new(3, 53)));

    assert!(matches!(*engine.eval::<FLOAT>(
        r#"
        let ex = 41.0;
        let fx = Math::Advanced::bar_fourth_adders::add_float(ex, 1.0);
        fx
        "#).unwrap_err(),
        EvalAltResult::ErrorFunctionNotFound(s, p)
            if s == "Math::Advanced::bar_fourth_adders::add_float (f64, f64)"
            && p == rhai::Position::new(3, 53)));

    Ok(())
}
