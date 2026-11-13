//! This script illustrates how to put doc-comments on functions.

/// The function `foo`, which prints `hello, world!` and a magic number,
/// accepts three parameters.
///
/// # Parameters
///
/// * `x` - `i64`
/// * `y` - `string`
/// * `z` - `bool`
///
/// # Notes
///
/// This is a doc-comment.  It can be obtained with the `metadata` feature.
///
/// An example is the `rhai-doc` app.
///
/// # Example
///
/// ```rhai
/// let x = foo(42, "hello", true);
///
/// print(x);     // prints 47
/// ```
fn foo(x, y, z) {
   print(`hello, world! ${if z { x + y.len() } else { x } }`);
}

foo(39, "bar", true);
