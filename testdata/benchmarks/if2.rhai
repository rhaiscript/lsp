let a = 42;
let b = 123;

let x = if a <= b {         // if-expression
    b - a
} else {
    a - b
} * 10;

print(`x should be 810: ${x}`);
