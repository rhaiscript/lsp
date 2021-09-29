use rhai::plugin::*;

#[derive(Clone)]
pub struct Point {
    x: f32,
    y: f32,
}

#[export_module]
pub mod test_module {
    pub use super::Point;
    #[rhai_fn(index_get)]
    pub fn test_fn(input: &mut Point, i: f32) {
        input.x *= 2.0;
    }
}

fn main() {
    let mut n = Point {
        x: 0.0,
        y: 10.0,
    };
    if test_module::test_fn(&mut n, 5.0) {
        println!("yes");
    } else {
        println!("no");
    }
}
