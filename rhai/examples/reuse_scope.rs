use rhai::{Engine, EvalAltResult, Scope, INT};

fn main() -> Result<(), Box<EvalAltResult>> {
    let engine = Engine::new();
    let mut scope = Scope::new();

    engine.eval_with_scope::<()>(&mut scope, "let x = 4 + 5")?;

    println!("x = {}", scope.get_value::<INT>("x").unwrap());

    for _ in 0..10 {
        let result = engine.eval_with_scope::<INT>(&mut scope, "x += 1; x")?;

        println!("result: {}", result);
    }

    println!("x = {}", scope.get_value::<INT>("x").unwrap());

    Ok(())
}
