//! Module containing all built-in _packages_ available to Rhai, plus facilities to define custom packages.

use crate::{Module, Shared};

pub(crate) mod arithmetic;
mod array_basic;
mod fn_basic;
mod iter_basic;
mod lang_core;
mod logic;
mod map_basic;
mod math_basic;
mod pkg_core;
mod pkg_std;
mod string_basic;
mod string_more;
mod time_basic;

pub use arithmetic::ArithmeticPackage;
#[cfg(not(feature = "no_index"))]
pub use array_basic::BasicArrayPackage;
pub use fn_basic::BasicFnPackage;
pub use iter_basic::BasicIteratorPackage;
pub use logic::LogicPackage;
#[cfg(not(feature = "no_object"))]
pub use map_basic::BasicMapPackage;
pub use math_basic::BasicMathPackage;
pub use pkg_core::CorePackage;
pub use pkg_std::StandardPackage;
pub use string_basic::BasicStringPackage;
pub use string_more::MoreStringPackage;
#[cfg(not(feature = "no_std"))]
pub use time_basic::BasicTimePackage;

/// Trait that all packages must implement.
pub trait Package {
    /// Register all the functions in a package into a store.
    fn init(lib: &mut Module);

    /// Retrieve the generic package library from this package.
    #[must_use]
    fn as_shared_module(&self) -> Shared<Module>;
}

/// Macro that makes it easy to define a _package_ (which is basically a shared [module][Module])
/// and register functions into it.
///
/// Functions can be added to the package using [`Module::set_native_fn`].
///
/// # Example
///
/// Define a package named `MyPackage` with a single function named `my_add`:
///
/// ```
/// use rhai::{Dynamic, EvalAltResult};
/// use rhai::def_package;
///
/// fn add(x: i64, y: i64) -> Result<i64, Box<EvalAltResult>> { Ok(x + y) }
///
/// def_package!(rhai:MyPackage:"My super-duper package", lib,
/// {
///     // Load a binary function with all value parameters.
///     lib.set_native_fn("my_add", add);
/// });
/// ```
#[macro_export]
macro_rules! def_package {
    ($root:ident : $package:ident : $comment:expr , $lib:ident , $block:stmt) => {
        #[doc=$comment]
        pub struct $package($root::Shared<$root::Module>);

        impl $root::packages::Package for $package {
            fn as_shared_module(&self) -> $root::Shared<$root::Module> {
                self.0.clone()
            }

            fn init($lib: &mut $root::Module) {
                $block
            }
        }

        impl Default for $package {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $package {
            pub fn new() -> Self {
                let mut module = $root::Module::new();
                <Self as $root::packages::Package>::init(&mut module);
                module.build_index();
                Self(module.into())
            }
        }
    };
}
