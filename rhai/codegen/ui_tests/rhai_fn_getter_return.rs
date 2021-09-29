use rhai::plugin::*;

#[derive(Clone)]
pub struct Point {
    x: f32,
    y: f32,
}

#[export_module]
pub mod test_module {
    pub use super::Point;
    #[rhai_fn(get = "foo")]
    pub fn test_fn(input: &mut Point) {
        input.x *= 2.0;
    }
}

fn main() {
    let mut n = Point {
        x: 0.0,
        y: 10.0,
    };
    test_module::test_fn(&mut n);
    if n.x > 10.0 {
        println!("yes");
    } else {
        println!("no");
    }
}
