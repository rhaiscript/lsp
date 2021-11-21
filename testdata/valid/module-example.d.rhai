// This file declares a module for a Rhai file with the same name.
//
// The path to the rhai file can be omitted, in that case it is assumed to be "<module-file-name>.rhai"
// where `module-file-name` is this the name of this file without the `.d.rhai` extension.
//
// The items defined are available according to Rhai module semantics.
//
// Cannot contain operator definitions.
module "./module-example";

fn hello(name: string);
