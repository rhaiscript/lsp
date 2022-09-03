// Simply re-export bar.
//
// Notice how this module has no definition file, thus no description.
import "./bar.rhai" as bar;

bar::hello();

/// A function that expects to have scoped-values
/// can have definitions contiguous in doc comment blocks:
/**

```rhai-scope

/// Hello is always available when this function is called.
let hello;

```

*/
/// Only doc comment blocks are supported,
/// the following is ignored:
///
/// ```rhai-scope
/// let hi;
/// ```
fn scoped() {
  return hello;

  // error:
  // hi
}
