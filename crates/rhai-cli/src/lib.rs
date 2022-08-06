pub mod args;
mod execute;

use rhai_common::environment::Environment;

pub struct Rhai<E: Environment> {
    #[allow(dead_code)]
    env: E,
}

impl<E: Environment> Rhai<E> {
    pub fn new(env: E) -> Self {
        Self { env }
    }
}
