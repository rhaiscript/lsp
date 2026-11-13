//! This script defines a function with many parameters.

const KEY = 38;

fn f(a, b, c, d, e, f) {
    let x = global::KEY;        // <- access global module
    a - b * c - d * e - f + x
}

let result = f(100, 5, 2, 9, 6, 32);

print(`result should be 42: ${result}`);
