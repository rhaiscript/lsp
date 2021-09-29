#![cfg(not(feature = "no_object"))]

use rhai::{Engine, EvalAltResult, INT};

#[test]
fn test_get_set() -> Result<(), Box<EvalAltResult>> {
    #[derive(Clone)]
    struct TestStruct {
        x: INT,
        y: INT,
        array: Vec<INT>,
    }

    impl TestStruct {
        fn get_x(&mut self) -> INT {
            self.x
        }

        fn set_x(&mut self, new_x: INT) {
            self.x = new_x;
        }

        fn get_y(&mut self) -> INT {
            self.y
        }

        fn new() -> Self {
            Self {
                x: 1,
                y: 0,
                array: vec![1, 2, 3, 4, 5],
            }
        }
    }

    let mut engine = Engine::new();

    engine.register_type::<TestStruct>();

    engine.register_get_set("x", TestStruct::get_x, TestStruct::set_x);
    engine.register_get("y", TestStruct::get_y);
    engine.register_fn("add", |value: &mut INT| *value += 41);
    engine.register_fn("new_ts", TestStruct::new);

    assert_eq!(engine.eval::<INT>("let a = new_ts(); a.x = 500; a.x")?, 500);
    assert_eq!(engine.eval::<INT>("let a = new_ts(); a.x.add(); a.x")?, 42);
    assert_eq!(engine.eval::<INT>("let a = new_ts(); a.y.add(); a.y")?, 0);

    engine.register_indexer_get_set(
        |value: &mut TestStruct, index: &str| value.array[index.len()],
        |value: &mut TestStruct, index: &str, new_val: INT| value.array[index.len()] = new_val,
    );

    #[cfg(not(feature = "no_index"))]
    assert_eq!(engine.eval::<INT>(r#"let a = new_ts(); a["abc"]"#)?, 4);

    #[cfg(not(feature = "no_index"))]
    assert_eq!(
        engine.eval::<INT>(r#"let a = new_ts(); a["abc"] = 42; a["abc"]"#)?,
        42
    );

    assert_eq!(engine.eval::<INT>(r"let a = new_ts(); a.abc")?, 4);
    assert_eq!(
        engine.eval::<INT>(r"let a = new_ts(); a.abc = 42; a.abc")?,
        42
    );

    Ok(())
}

#[test]
fn test_get_set_chain() -> Result<(), Box<EvalAltResult>> {
    #[derive(Clone)]
    struct TestChild {
        x: INT,
    }

    impl TestChild {
        fn get_x(&mut self) -> INT {
            self.x
        }

        fn set_x(&mut self, new_x: INT) {
            self.x = new_x;
        }

        fn new() -> TestChild {
            TestChild { x: 1 }
        }
    }

    #[derive(Clone)]
    struct TestParent {
        child: TestChild,
    }

    impl TestParent {
        fn get_child(&mut self) -> TestChild {
            self.child.clone()
        }

        fn set_child(&mut self, new_child: TestChild) {
            self.child = new_child;
        }

        fn new() -> TestParent {
            TestParent {
                child: TestChild::new(),
            }
        }
    }

    let mut engine = Engine::new();

    engine.register_type::<TestChild>();
    engine.register_type_with_name::<TestParent>("TestParent");

    engine.register_get_set("x", TestChild::get_x, TestChild::set_x);
    engine.register_get_set("child", TestParent::get_child, TestParent::set_child);

    engine.register_fn("new_tp", TestParent::new);

    assert_eq!(
        engine.eval::<INT>("let a = new_tp(); a.child.x = 500; a.child.x")?,
        500
    );

    assert_eq!(
        engine.eval::<String>("let a = new_tp(); type_of(a)")?,
        "TestParent"
    );

    Ok(())
}

#[test]
fn test_get_set_op_assignment() -> Result<(), Box<EvalAltResult>> {
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct Num(INT);

    impl Num {
        fn get(&mut self) -> INT {
            self.0
        }
        fn set(&mut self, x: INT) {
            self.0 = x;
        }
    }

    let mut engine = Engine::new();

    engine
        .register_type::<Num>()
        .register_fn("new_ts", || Num(40))
        .register_get_set("v", Num::get, Num::set);

    assert_eq!(
        engine.eval::<Num>("let a = new_ts(); a.v = a.v + 2; a")?,
        Num(42)
    );
    assert_eq!(
        engine.eval::<Num>("let a = new_ts(); a.v += 2; a")?,
        Num(42)
    );

    Ok(())
}
