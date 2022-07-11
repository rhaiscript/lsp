/// This definition file creates an `external` module.
///
/// Only the item definitions are known, values are provided
/// by the runtime.
module external;

/// Load a file into a byte array.
fn load_file(path: String) -> [u8];
