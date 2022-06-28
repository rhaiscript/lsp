import "./foo.rhai" as foo;

let file = external::load_file();

foo::bar::hello();

// Private function is not reachable.
foo::bar::cant_touch_this();
