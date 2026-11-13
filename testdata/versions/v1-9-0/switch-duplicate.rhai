// Rhai 1.9.0: syntax changes

// allow duplicate switch
let x = 2;
switch x { 1 => 1, 1 => 2, _ => 3 }