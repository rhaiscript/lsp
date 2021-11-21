// This file declares a module with a given name.
//
// The contained definitions become available according to Rhai module import semantics.
//
// The name of the module cannot be a valid path, otherwise a Rhai file is expected.
//
// Cannot contain operator definitions.
module "hello";

/// Says hello.
///
/// Example usage:
/// ```rhai
/// import "hello" as greetings;
/// greetings::hello(greetings::DEFAULT_NAME);
/// ```
///
fn hello(name: string): void;

/// A default name to greet.
const DEFAULT_NAME: string;
