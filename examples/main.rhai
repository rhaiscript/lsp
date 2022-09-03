import "./foo.rhai" as foo;

let file = external::load_file();

foo::bar::hello();

let a = foo::bar::hello;

// We can access nested modules defined inline as well.
nested::inner::need::to::go_deeper::YEAH;

// We can use custom operators from definitions.
assert("rhai" is "awesome");

// Private function is not reachable.
// Uncomment the line below to see the error.
// foo::bar::cant_touch_this();
