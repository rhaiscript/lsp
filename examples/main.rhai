import "./foo.rhai" as foo;

let file = external::load_file();

foo::bar::hello();

// Private function is not reachable.
// Uncomment the line below to see the error.
// foo::bar::cant_touch_this();
