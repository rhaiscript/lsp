// Rhai 1.9.0: syntax changes

// switch contains multi values
let x = 2;
switch x { 1 | 2 | 3 => 1, _ => 3 }
