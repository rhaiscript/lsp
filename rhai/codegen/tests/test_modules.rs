use rhai::{Array, Engine, EvalAltResult, FLOAT, INT};

pub mod empty_module {
    use rhai::plugin::*;

    #[export_module]
    pub mod EmptyModule {}
}

#[test]
fn empty_module_test() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    let m = rhai::exported_module!(crate::empty_module::EmptyModule);
    engine.register_static_module("Module::Empty", m.into());

    Ok(())
}

pub mod one_fn_module {
    use rhai::plugin::*;

    #[export_module]
    pub mod advanced_math {
        use rhai::FLOAT;
        pub fn get_mystic_number() -> FLOAT {
            42.0 as FLOAT
        }
    }
}

#[test]
fn one_fn_module_test() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    let m = rhai::exported_module!(crate::one_fn_module::advanced_math);
    engine.register_static_module("Math::Advanced", m.into());

    assert_eq!(
        engine.eval::<FLOAT>(r#"let m = Math::Advanced::get_mystic_number();m"#)?,
        42.0
    );
    Ok(())
}

pub mod one_fn_and_const_module {
    use rhai::plugin::*;

    #[export_module]
    pub mod advanced_math {
        use rhai::FLOAT;

        pub const MYSTIC_NUMBER: FLOAT = 42.0 as FLOAT;

        pub fn euclidean_distance(x1: FLOAT, y1: FLOAT, x2: FLOAT, y2: FLOAT) -> FLOAT {
            ((y2 - y1).abs().powf(2.0) + (x2 - x1).abs().powf(2.0)).sqrt()
        }
    }
}

#[test]
fn one_fn_and_const_module_test() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    let m = rhai::exported_module!(crate::one_fn_and_const_module::advanced_math);
    engine.register_static_module("Math::Advanced", m.into());

    assert_eq!(
        engine.eval::<FLOAT>(
            r#"
            let m = Math::Advanced::MYSTIC_NUMBER;
            let x = Math::Advanced::euclidean_distance(0.0, 1.0, 0.0, m);
            x"#
        )?,
        41.0
    );
    Ok(())
}

pub mod raw_fn_str_module {
    use rhai::plugin::*;

    #[export_module]
    pub mod host_io {
        pub fn write_out_str(message: &str) -> bool {
            eprintln!("{}", message);
            true
        }
    }
}

#[test]
fn raw_fn_str_module_test() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    let m = rhai::exported_module!(crate::raw_fn_str_module::host_io);
    engine.register_static_module("Host::IO", m.into());

    assert_eq!(
        engine.eval::<bool>(r#"let x = Host::IO::write_out_str("hello world!"); x"#)?,
        true
    );
    Ok(())
}

pub mod mut_opaque_ref_module {
    use rhai::plugin::*;
    use rhai::INT;

    #[derive(Clone)]
    pub struct StatusMessage {
        os_code: Option<INT>,
        message: String,
        is_ok: bool,
    }

    #[export_module]
    pub mod host_msg {
        use super::{StatusMessage, INT};

        pub fn new_message(is_ok: bool, message: &str) -> StatusMessage {
            StatusMessage {
                is_ok,
                os_code: None,
                message: message.to_string(),
            }
        }

        pub fn new_os_message(is_ok: bool, os_code: INT) -> StatusMessage {
            StatusMessage {
                is_ok,
                os_code: Some(os_code),
                message: format!("OS Code {}", os_code),
            }
        }

        pub fn write_out_message(message: &mut StatusMessage) -> bool {
            eprintln!("{}", message.message);
            true
        }
    }
}

#[test]
fn mut_opaque_ref_test() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    let m = rhai::exported_module!(crate::mut_opaque_ref_module::host_msg);
    engine.register_static_module("Host::Msg", m.into());

    assert_eq!(
        engine.eval::<bool>(
            r#"
            let success = "it worked";
            let message1 = Host::Msg::new_message(true, success);
            let ok1 = Host::Msg::write_out_message(message1);
            let message2 = Host::Msg::new_os_message(true, 0);
            let ok2 = Host::Msg::write_out_message(message2);
            ok1 && ok2"#
        )?,
        true
    );
    Ok(())
}

mod duplicate_fn_rename {
    use rhai::plugin::*;
    #[export_module]
    pub mod my_adds {
        use rhai::{FLOAT, INT};

        #[rhai_fn(name = "add")]
        pub fn add_float(f1: FLOAT, f2: FLOAT) -> FLOAT {
            f1 + f2
        }

        #[rhai_fn(name = "add")]
        pub fn add_int(i1: INT, i2: INT) -> INT {
            i1 + i2
        }
    }
}

#[test]
fn duplicate_fn_rename_test() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    engine.register_fn("get_mystic_number", || 42 as FLOAT);
    let m = rhai::exported_module!(crate::duplicate_fn_rename::my_adds);
    engine.register_static_module("Math::Advanced", m.into());

    let output_array = engine.eval::<Array>(
        r#"
        let fx = get_mystic_number();
        let fy = Math::Advanced::add(fx, 1.0);
        let ix = 42;
        let iy = Math::Advanced::add(ix, 1);
        [fy, iy]
        "#,
    )?;
    assert_eq!(&output_array[0].as_float().unwrap(), &43.0);
    assert_eq!(&output_array[1].as_int().unwrap(), &43);
    Ok(())
}

mod multiple_fn_rename {
    use rhai::plugin::*;
    #[export_module]
    pub mod my_adds {
        use rhai::{FLOAT, INT};

        pub fn get_mystic_number() -> FLOAT {
            42.0
        }
        #[rhai_fn(name = "add", name = "+", name = "add_together")]
        pub fn add_float(f1: FLOAT, f2: FLOAT) -> FLOAT {
            f1 + f2 * 2.0
        }

        #[rhai_fn(name = "add", name = "+", name = "add_together")]
        pub fn add_int(i1: INT, i2: INT) -> INT {
            i1 + i2 * 2
        }

        #[rhai_fn(name = "prop", get = "prop")]
        pub fn get_prop(x: FLOAT) -> FLOAT {
            x * 2.0
        }

        #[rhai_fn(name = "idx", index_get)]
        pub fn index(x: FLOAT, i: INT) -> FLOAT {
            x + (i as FLOAT)
        }
    }
}

#[test]
fn multiple_fn_rename_test() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    let m = rhai::exported_module!(crate::multiple_fn_rename::my_adds);
    engine.register_global_module(m.into());

    let output_array = engine.eval::<Array>(
        r#"
       let fx = get_mystic_number();
       let fy1 = add(fx, 1.0);
       let fy2 = add_together(fx, 1.0);
       let fy3 = fx + 1.0;
       let p1 = fx.prop;
       let p2 = prop(fx);
       let idx1 = fx[1];
       let idx2 = idx(fx, 1);
       let ix = 42;
       let iy1 = add(ix, 1);
       let iy2 = add_together(ix, 1);
       let iy3 = ix + 1;
       [fy1, fy2, fy3, iy1, iy2, iy3, p1, p2, idx1, idx2]
       "#,
    )?;
    assert_eq!(&output_array[0].as_float().unwrap(), &44.0);
    assert_eq!(&output_array[1].as_float().unwrap(), &44.0);
    assert_eq!(&output_array[2].as_float().unwrap(), &44.0);
    assert_eq!(&output_array[3].as_int().unwrap(), &44);
    assert_eq!(&output_array[4].as_int().unwrap(), &44);
    assert_eq!(&output_array[5].as_int().unwrap(), &44);
    assert_eq!(&output_array[6].as_float().unwrap(), &84.0);
    assert_eq!(&output_array[7].as_float().unwrap(), &84.0);
    assert_eq!(&output_array[8].as_float().unwrap(), &43.0);
    assert_eq!(&output_array[9].as_float().unwrap(), &43.0);
    Ok(())
}

mod export_by_prefix {
    use rhai::plugin::*;

    #[export_module(export_prefix = "foo_")]
    pub mod my_adds {
        use rhai::{FLOAT, INT};

        #[rhai_fn(name = "foo_add_f")]
        pub fn foo_add1(f1: FLOAT, f2: FLOAT) -> FLOAT {
            f1 + f2
        }

        #[rhai_fn(name = "bar_add_i")]
        fn foo_add_int(i1: INT, i2: INT) -> INT {
            i1 + i2
        }

        #[rhai_fn(name = "foo_add_float2")]
        pub fn add_float2(f1: FLOAT, f2: FLOAT) -> FLOAT {
            f1 + f2
        }

        pub fn foo_m(f1: FLOAT, f2: FLOAT) -> FLOAT {
            f1 + f2
        }

        fn foo_n(i1: INT, i2: INT) -> INT {
            i1 + i2
        }

        pub fn bar_m(f1: FLOAT, f2: FLOAT) -> FLOAT {
            f1 + f2
        }
    }
}

#[test]
fn export_by_prefix_test() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    let m = rhai::exported_module!(crate::export_by_prefix::my_adds);
    engine.register_static_module("Math::Advanced", m.into());

    let output_array = engine.eval::<Array>(
        r#"
        let ex = 41.0;
        let fx = Math::Advanced::foo_add_f(ex, 1.0);
        let gx = Math::Advanced::foo_m(41.0, 1.0);
        let ei = 41;
        let fi = Math::Advanced::bar_add_i(ei, 1);
        let gi = Math::Advanced::foo_n(41, 1);
        [fx, gx, fi, gi]
        "#,
    )?;
    assert_eq!(&output_array[0].as_float().unwrap(), &42.0);
    assert_eq!(&output_array[1].as_float().unwrap(), &42.0);
    assert_eq!(&output_array[2].as_int().unwrap(), &42);
    assert_eq!(&output_array[3].as_int().unwrap(), &42);

    assert!(matches!(*engine.eval::<FLOAT>(
        r#"
        let ex = 41.0;
        let fx = Math::Advanced::foo_add_float2(ex, 1.0);
        fx
        "#).unwrap_err(),
        EvalAltResult::ErrorFunctionNotFound(s, p)
            if s == "Math::Advanced::foo_add_float2 (f64, f64)"
            && p == rhai::Position::new(3, 34)));

    assert!(matches!(*engine.eval::<FLOAT>(
        r#"
        let ex = 41.0;
        let fx = Math::Advanced::bar_m(ex, 1.0);
        fx
        "#).unwrap_err(),
        EvalAltResult::ErrorFunctionNotFound(s, p)
            if s == "Math::Advanced::bar_m (f64, f64)"
            && p == rhai::Position::new(3, 34)));

    Ok(())
}

mod export_all {
    use rhai::plugin::*;

    #[export_module(export_all)]
    pub mod my_adds {
        use rhai::{FLOAT, INT};

        #[rhai_fn(name = "foo_add_f")]
        pub fn add_float(f1: FLOAT, f2: FLOAT) -> FLOAT {
            f1 + f2
        }

        #[rhai_fn(name = "foo_add_i")]
        fn add_int(i1: INT, i2: INT) -> INT {
            i1 + i2
        }

        #[rhai_fn(skip)]
        pub fn add_float2(f1: FLOAT, f2: FLOAT) -> FLOAT {
            f1 + f2
        }

        pub fn foo_m(f1: FLOAT, f2: FLOAT) -> FLOAT {
            f1 + f2
        }

        fn foo_n(i1: INT, i2: INT) -> INT {
            i1 + i2
        }

        #[rhai_fn(skip)]
        fn foo_p(i1: INT, i2: INT) -> INT {
            i1 * i2
        }
    }
}

#[test]
fn export_all_test() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    let m = rhai::exported_module!(crate::export_all::my_adds);
    engine.register_static_module("Math::Advanced", m.into());

    let output_array = engine.eval::<Array>(
        r#"
        let ex = 41.0;
        let fx = Math::Advanced::foo_add_f(ex, 1.0);
        let gx = Math::Advanced::foo_m(41.0, 1.0);
        let ei = 41;
        let fi = Math::Advanced::foo_add_i(ei, 1);
        let gi = Math::Advanced::foo_n(41, 1);
        [fx, gx, fi, gi]
        "#,
    )?;
    assert_eq!(&output_array[0].as_float().unwrap(), &42.0);
    assert_eq!(&output_array[1].as_float().unwrap(), &42.0);
    assert_eq!(&output_array[2].as_int().unwrap(), &42);
    assert_eq!(&output_array[3].as_int().unwrap(), &42);

    assert!(matches!(*engine.eval::<INT>(
        r#"
        let ex = 41;
        let fx = Math::Advanced::foo_p(ex, 1);
        fx
        "#).unwrap_err(),
        EvalAltResult::ErrorFunctionNotFound(s, p)
            if s == "Math::Advanced::foo_p (i64, i64)"
            && p == rhai::Position::new(3, 34)));

    Ok(())
}
