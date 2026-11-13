//! This script runs for-loops.

let arr = [1, true, 123.456, "hello", 3, 42];

// Loop over array with counter
for (a, i) in arr {
    for (b, j) in ['x', 42, (), 123, 99, 0.5] {
        if b > 100 { continue; }

        print(`(${i}, ${j}) = (${a}, ${b})`);
    }

    if a == 3 { break; }
}
//print(a);                 // <- if you uncomment this line, the script will fail to compile
                            //    because 'a' is not defined here

for i in range(5, 0, -1) {  // runs from 5 down to 1
    print(i);
}
