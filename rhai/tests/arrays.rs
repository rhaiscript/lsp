#![cfg(not(feature = "no_index"))]
use rhai::{Array, Engine, EvalAltResult, INT};

fn convert_to_vec<T: Clone + 'static>(array: Array) -> Vec<T> {
    array.into_iter().map(|v| v.clone_cast::<T>()).collect()
}

#[test]
fn test_arrays() -> Result<(), Box<EvalAltResult>> {
    let mut a = Array::new();
    a.push((42 as INT).into());

    assert_eq!(a[0].clone_cast::<INT>(), 42);

    let engine = Engine::new();

    assert_eq!(engine.eval::<INT>("let x = [1, 2, 3]; x[1]")?, 2);
    assert_eq!(engine.eval::<INT>("let x = [1, 2, 3,]; x[1]")?, 2);
    assert_eq!(engine.eval::<INT>("let y = [1, 2, 3]; y[1] = 5; y[1]")?, 5);
    assert_eq!(
        engine.eval::<char>(r#"let y = [1, [ 42, 88, "93" ], 3]; y[1][2][1]"#)?,
        '3'
    );
    assert_eq!(engine.eval::<INT>("let y = [1, 2, 3]; y[0]")?, 1);
    assert_eq!(engine.eval::<INT>("let y = [1, 2, 3]; y[-1]")?, 3);
    assert_eq!(engine.eval::<INT>("let y = [1, 2, 3]; y[-3]")?, 1);
    assert!(engine.eval::<bool>("let y = [1, 2, 3]; 2 in y")?);
    assert_eq!(engine.eval::<INT>("let y = [1, 2, 3]; y += 4; y[3]")?, 4);
    assert_eq!(
        convert_to_vec::<INT>(engine.eval("let y = [1, 2, 3]; y[1] += 4; y")?),
        [1, 6, 3]
    );

    #[cfg(not(feature = "no_object"))]
    {
        assert_eq!(
            convert_to_vec::<INT>(engine.eval("let y = [1, 2, 3]; y.push(4); y")?),
            [1, 2, 3, 4]
        );
        assert_eq!(
            convert_to_vec::<INT>(engine.eval("let y = [1, 2, 3]; y.insert(0, 4); y")?),
            [4, 1, 2, 3]
        );
        assert_eq!(
            convert_to_vec::<INT>(engine.eval("let y = [1, 2, 3]; y.insert(999, 4); y")?),
            [1, 2, 3, 4]
        );
        assert_eq!(
            convert_to_vec::<INT>(engine.eval("let y = [1, 2, 3]; y.insert(-2, 4); y")?),
            [1, 4, 2, 3]
        );
        assert_eq!(
            convert_to_vec::<INT>(engine.eval("let y = [1, 2, 3]; y.insert(-999, 4); y")?),
            [4, 1, 2, 3]
        );
        assert_eq!(
            engine.eval::<INT>("let y = [1, 2, 3]; let z = [42]; y[z.len]")?,
            2
        );
        assert_eq!(
            engine.eval::<INT>("let y = [1, 2, [3, 4, 5, 6]]; let z = [42]; y[2][z.len]")?,
            4
        );
        assert_eq!(
            engine.eval::<INT>("let y = [1, 2, 3]; let z = [2]; y[z[0]]")?,
            3
        );

        assert_eq!(
            convert_to_vec::<INT>(engine.eval(
                "
                    let x = [2, 9];
                    x.insert(-1, 1);
                    x.insert(999, 3);
                    x.insert(-9, 99);

                    let r = x.remove(2);

                    let y = [4, 5];
                    x.append(y);

                    x 
                "
            )?),
            [99, 2, 9, 3, 4, 5]
        );
    }

    assert_eq!(
        convert_to_vec::<INT>(engine.eval(
            "
                let x = [1, 2, 3];
                x += [4, 5];
                x
            "
        )?),
        [1, 2, 3, 4, 5]
    );
    assert_eq!(
        convert_to_vec::<INT>(engine.eval(
            "
                let x = [1, 2, 3];
                let y = [4, 5];
                x + y
            "
        )?),
        [1, 2, 3, 4, 5]
    );

    Ok(())
}

#[test]
#[cfg(not(feature = "no_object"))]
fn test_array_with_structs() -> Result<(), Box<EvalAltResult>> {
    #[derive(Clone)]
    struct TestStruct {
        x: INT,
    }

    impl TestStruct {
        fn update(&mut self) {
            self.x += 1000;
        }

        fn get_x(&mut self) -> INT {
            self.x
        }

        fn set_x(&mut self, new_x: INT) {
            self.x = new_x;
        }

        fn new() -> Self {
            Self { x: 1 }
        }
    }

    let mut engine = Engine::new();

    engine.register_type::<TestStruct>();

    engine.register_get_set("x", TestStruct::get_x, TestStruct::set_x);
    engine.register_fn("update", TestStruct::update);
    engine.register_fn("new_ts", TestStruct::new);

    assert_eq!(engine.eval::<INT>("let a = [new_ts()]; a[0].x")?, 1);

    assert_eq!(
        engine.eval::<INT>(
            "
                let a = [new_ts()];
                a[0].x = 100;
                a[0].update();
                a[0].x
            "
        )?,
        1100
    );

    Ok(())
}

#[cfg(not(feature = "no_object"))]
#[cfg(not(feature = "no_function"))]
#[cfg(not(feature = "no_closure"))]
#[test]
fn test_arrays_map_reduce() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    assert_eq!(engine.eval::<INT>("[1].map(|x| x + 41)[0]")?, 42);
    assert_eq!(engine.eval::<INT>("([1].map(|x| x + 41))[0]")?, 42);

    assert_eq!(
        convert_to_vec::<INT>(engine.eval(
            "
                let x = [1, 2, 3];
                x.filter(|v| v > 2)
            "
        )?),
        [3]
    );

    assert_eq!(
        convert_to_vec::<INT>(engine.eval(
            "
                let x = [1, 2, 3];
                x.filter(|v, i| v > i)
            "
        )?),
        [1, 2, 3]
    );

    assert_eq!(
        convert_to_vec::<INT>(engine.eval(
            "
                let x = [1, 2, 3];
                x.map(|v| v * 2)
            "
        )?),
        [2, 4, 6]
    );

    assert_eq!(
        convert_to_vec::<INT>(engine.eval(
            "
                let x = [1, 2, 3];
                x.map(|v, i| v * i)
            "
        )?),
        [0, 2, 6]
    );

    assert_eq!(
        engine.eval::<INT>(
            r#"
                let x = [1, 2, 3];
                x.reduce(|sum, v| if sum.type_of() == "()" { v * v } else { sum + v * v })
            "#
        )?,
        14
    );

    assert_eq!(
        engine.eval::<INT>(
            "
                let x = [1, 2, 3];
                x.reduce(|sum, v, i| {
                    if i == 0 { sum = 10 }
                    sum + v * v
                })
            "
        )?,
        24
    );

    assert_eq!(
        engine.eval::<INT>(
            r#"
                let x = [1, 2, 3];
                x.reduce_rev(|sum, v| if sum.type_of() == "()" { v * v } else { sum + v * v })
            "#
        )?,
        14
    );

    assert_eq!(
        engine.eval::<INT>(
            "
                let x = [1, 2, 3];
                x.reduce_rev(|sum, v, i| { if i == 2 { sum = 10 } sum + v * v })
            "
        )?,
        24
    );

    assert!(engine.eval::<bool>(
        "
            let x = [1, 2, 3];
            x.some(|v| v > 1)
        "
    )?);

    assert!(engine.eval::<bool>(
        "
            let x = [1, 2, 3];
            x.some(|v, i| v * i == 0)
        "
    )?);

    assert!(!engine.eval::<bool>(
        "
            let x = [1, 2, 3];
            x.all(|v| v > 1)
        "
    )?);

    assert!(engine.eval::<bool>(
        "
            let x = [1, 2, 3];
            x.all(|v, i| v > i)
        "
    )?);

    Ok(())
}
