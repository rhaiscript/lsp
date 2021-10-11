// This script runs 1 million iterations
// to test the speed of the scripting engine.

let now = timestamp();
let x = 1_000_000;

print("Ready... Go!");

while x > 0 {
    x -= 1;
}

print(`Finished. Run time = ${now.elapsed} seconds.`);
