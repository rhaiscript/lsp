//! This script defines multiple versions of the same function
//! for use as method with different data types.

// For strings
fn string.calc(x) {
    this.len + x
}
// For integers
fn int.calc(x) {
    this * x
}
// For booleans
fn bool.calc(x) {
    if this { x } else { 0}
}
// For arrays
fn array.calc(x) {
    this.len + x
}
// For object maps
fn map.calc(x) {
    this[x]
}
// Catch-all
fn calc(x) {
    `${this}: ${x}`
}

print("hello".calc(42));                // 47
print(42.calc(42));                     // 1764
print(true.calc(42));                   // 42
print(false.calc(42));                  // 0
print([1,2,3].calc(42));                // 45
print(#{"a": 1, "b": 2}.calc("b"));     // 2
print('x'.calc(42));                    // x: 42
