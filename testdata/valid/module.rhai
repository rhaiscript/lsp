//! This script imports an external script as a module.

import "loop" as x;

print(`Module test! foo = ${x::foo}`);
