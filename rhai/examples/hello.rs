use rhai::{Engine, EvalAltResult, INT};

fn main() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();

    engine.run(r#"print("hello, world!")"#)?;

    let result = engine.eval::<INT>("40 + 2")?;

    println!("Answer: {}", result); // prints 42

    Ok(())
}
