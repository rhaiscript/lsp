//! This script runs for-loops

const MAX = 1_000_000;

print(`Iterating an array with ${MAX} items...`);

print("Ready... Go!");

let now = timestamp();

let list = [];

// Loop over range
for i in 0..MAX {
    list.push(i);
}

print(`Time = ${now.elapsed} seconds...`);

let sum = 0;

// Loop over array
for i in list {
    sum += i;
}

print(`Sum = ${sum}`);
print(`Finished. Total run time = ${now.elapsed} seconds.`);
