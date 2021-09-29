use rhai::{Engine, EvalAltResult, Module, INT};

#[cfg(not(feature = "no_float"))]
use rhai::FLOAT;

#[cfg(feature = "decimal")]
#[cfg(not(feature = "no_float"))]
use rust_decimal::Decimal;

#[test]
fn test_for_loop() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    #[cfg(not(feature = "no_index"))]
    assert_eq!(
        engine.eval::<INT>(
            "
                let sum1 = 0;
                let sum2 = 0;
                let inputs = [1, 2, 3, 4, 5];

                for x in inputs {
                    sum1 += x;
                }

                for x in range(1, 6) {
                    sum2 += x;
                }

                for x in range(1, 6, 3) {
                    sum2 += x;
                }

                sum1 + sum2
            "
        )?,
        35
    );

    #[cfg(not(feature = "no_index"))]
    assert_eq!(
        engine.eval::<INT>(
            "
                let sum = 0;
                let inputs = [1, 2, 3, 4, 5];

                for (x, i) in inputs {
                    sum += x * (i + 1);
                }
                sum
            "
        )?,
        55
    );

    assert_eq!(
        engine.eval::<INT>(
            "
                let sum = 0;
                for x in range(1, 10) { sum += x; }
                sum
            "
        )?,
        45
    );

    assert_eq!(
        engine.eval::<INT>(
            "
                let sum = 0;
                for x in range(1, 10, 2) { sum += x; }
                sum
            "
        )?,
        25
    );

    #[cfg(not(feature = "unchecked"))]
    assert_eq!(
        engine.eval::<INT>(
            "
                let sum = 0;
                for x in range(10, 1, 2) { sum += x; }
                sum
            "
        )?,
        0
    );

    #[cfg(not(feature = "unchecked"))]
    assert_eq!(
        engine.eval::<INT>(
            "
                let sum = 0;
                for x in range(1, 10, -2) { sum += x; }
                sum
            "
        )?,
        0
    );

    assert_eq!(
        engine.eval::<INT>(
            "
                let sum = 0;
                for x in range(10, 1, -2) { sum += x; }
                sum
            "
        )?,
        30
    );

    #[cfg(not(feature = "no_float"))]
    {
        assert_eq!(
            engine.eval::<FLOAT>(
                "
                    let sum = 0.0;
                    for x in range(1.0, 10.0, 2.0) { sum += x; }
                    sum
                "
            )?,
            25.0
        );

        #[cfg(not(feature = "unchecked"))]
        assert_eq!(
            engine.eval::<FLOAT>(
                "
                    let sum = 0.0;
                    for x in range(10.0, 1.0, 2.0) { sum += x; }
                    sum
                "
            )?,
            0.0
        );

        #[cfg(not(feature = "unchecked"))]
        assert_eq!(
            engine.eval::<FLOAT>(
                "
                    let sum = 0.0;
                    for x in range(1.0, 10.0, -2.0) { sum += x; }
                    sum
                "
            )?,
            0.0
        );

        assert_eq!(
            engine.eval::<FLOAT>(
                "
                    let sum = 0.0;
                    for x in range(10.0, 1.0, -2.0) { sum += x; }
                    sum
                "
            )?,
            30.0
        );
    }

    #[cfg(not(feature = "no_float"))]
    #[cfg(feature = "decimal")]
    {
        assert_eq!(
            engine.eval::<Decimal>(
                "
                    let sum = to_decimal(0);
                    for x in range(to_decimal(1), to_decimal(10), to_decimal(2)) { sum += x; }
                    sum
                "
            )?,
            Decimal::from(25)
        );

        #[cfg(not(feature = "unchecked"))]
        assert_eq!(
            engine.eval::<Decimal>(
                "
                    let sum = to_decimal(0);
                    for x in range(to_decimal(10), to_decimal(1), to_decimal(2)) { sum += x; }
                    sum
                "
            )?,
            Decimal::from(0)
        );

        #[cfg(not(feature = "unchecked"))]
        assert_eq!(
            engine.eval::<Decimal>(
                "
                    let sum = to_decimal(0);
                    for x in range(to_decimal(1), to_decimal(10), to_decimal(-2)) { sum += x; }
                    sum
                "
            )?,
            Decimal::from(0)
        );

        assert_eq!(
            engine.eval::<Decimal>(
                "
                    let sum = to_decimal(0);
                    for x in range(to_decimal(10), to_decimal(1), to_decimal(-2)) { sum += x; }
                    sum
                "
            )?,
            Decimal::from(30)
        );
    }

    Ok(())
}

#[cfg(not(feature = "unchecked"))]
#[test]
fn test_for_overflow() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    #[cfg(not(feature = "only_i32"))]
    let script = "
        let sum = 0;

        for x in range(9223372036854775807, 0, 9223372036854775807) {
            sum += 1;
        }

        sum
    ";
    #[cfg(feature = "only_i32")]
    let script = "
        let sum = 0;

        for x in range(2147483647 , 0, 2147483647 ) {
            sum += 1;
        }

        sum
    ";

    assert_eq!(engine.eval::<INT>(script)?, 0);

    Ok(())
}

#[test]
fn test_for_string() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    let script = r#"
        let s = "hello";
        let sum = 0;

        for ch in chars(s) {
            sum += to_int(ch);
        }

        sum
    "#;

    assert_eq!(engine.eval::<INT>(script)?, 532);

    Ok(())
}

#[cfg(not(feature = "no_object"))]
#[cfg(not(feature = "no_index"))]
#[test]
fn test_for_object() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    let script = r#"
        let sum = 0;
        let keys = "";
        let map = #{a: 1, b: 2, c: 3};

        for key in map.keys() {
            keys += key;
        }
        for value in map.values() {
            sum += value;
        }

        keys.len + sum
    "#;

    assert_eq!(engine.eval::<INT>(script)?, 9);

    Ok(())
}

#[derive(Debug, Clone)]
struct MyIterableType(String);

impl IntoIterator for MyIterableType {
    type Item = char;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.chars().collect::<Vec<_>>().into_iter()
    }
}

#[cfg(not(feature = "no_module"))]
#[test]
fn test_for_module_iterator() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    // Set a type iterator deep inside a nested module chain
    let mut sub_module = Module::new();
    sub_module.set_iterable::<MyIterableType>();
    sub_module.set_native_fn("new_ts", || Ok(MyIterableType("hello".to_string())));

    let mut module = Module::new();
    module.set_sub_module("inner", sub_module);

    engine.register_static_module("testing", module.into());

    let script = r#"
        let item = testing::inner::new_ts();
        let result = "";

        for x in item {
            result += x;
        }
        result
    "#;

    assert_eq!(engine.eval::<String>(script)?, "hello");

    Ok(())
}
