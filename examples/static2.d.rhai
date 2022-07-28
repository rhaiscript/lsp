/// Same as `static.d.rhai`.
///
/// All modules, including static can be extended
/// in multiple definition files.
module static;

fn floor(number: f32) -> f32;

const RUNTIME_VERSION: string;

/// Magically compares two items.
// The precedence of the operator is represented by
// the binding the left and optionally the right binding powers
// after the `with` keyword.
//
// If the right binding power is higher,
// the operator is left-associative,
// otherwise the operator is right-associative.
op is(?, ?) -> bool with (1, 2);

/// Asserts that the given expression is true.
fn assert(expr: bool);
