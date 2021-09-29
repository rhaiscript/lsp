use rhai::{Engine, EvalAltResult, INT};

fn add(x: INT, y: INT) -> INT {
    x + y
}

fn main() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    engine.register_fn("add", add);

    let result = engine.eval::<INT>("add(40, 2)")?;

    println!("Answer: {}", result); // prints 42

    Ok(())
}
