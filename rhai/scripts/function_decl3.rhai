// This script defines a function with many parameters and calls it

const KEY = 38;

fn f(a, b, c, d, e, f) {
    a - b * c - d * e - f + global::KEY
}

let result = f(100, 5, 2, 9, 6, 32);

print(`result should be 42: ${result}`);
