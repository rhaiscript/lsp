use rhai::{Engine, EvalAltResult, INT};

#[derive(Debug, Clone)]
struct TestStruct {
    x: INT,
}

impl TestStruct {
    pub fn update(&mut self) {
        self.x += 1000;
    }

    pub fn new() -> Self {
        Self { x: 1 }
    }
}

#[cfg(not(feature = "no_object"))]
fn main() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    engine
        .register_type::<TestStruct>()
        .register_fn("new_ts", TestStruct::new)
        .register_fn("update", TestStruct::update);

    let result = engine.eval::<TestStruct>(
        "
            let x = new_ts();
            x.update();
            x
        ",
    )?;

    println!("result: {}", result.x); // prints 1001

    Ok(())
}

#[cfg(feature = "no_object")]
fn main() {
    panic!("This example does not run under 'no_object'.");
}
