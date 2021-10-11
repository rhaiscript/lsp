// This script defines a function that acts as a method

// Use 'this' to refer to the object of a method call
fn action(x, y) {
    this = this.abs() + x * y;      // 'this' can be modified
}

let obj = -40;

obj.action(1, 2);                   // call 'action' as method

print(`obj should now be 42: ${obj}`);
