// This module defines or extends the local scope.
//
// Items defined are always available as-is.
//
// Operators are only allowed to be defined this way.
static module;

/// The built-in `type_of` function that accepts anything and returns a string.
fn type_of(value: ?): string;

/// Describes a 2D coordinate.
type Coord = #{
    x: i64,
    y: i64,
};

/// Add two coordinates together by adding both `x` and `y` fields together individually
/// in the returned new coordinate.
op +(Coord, Coord): Coord;

/// A named coordinate.
type Point = #{
    /// A name can be either a string or a generated number.
    name: string | i64,
    coord: Coord,
};

/// An operator that returns whether the coordinates of two points are the same.
///
/// Example:
/// ```rhai
/// if start_point at finish_point {
///     // probably do nothing?
/// }
/// ```
op at(Coord, Coord): bool;

/// Returns information about the given point in a string.
///
/// Can be called like `point.to_string()`.
fn to_string(this: Point): string;

/// Get the current point available to the script, whatever this means.
///
/// Example:
/// ```rhai
/// let point = current_point();
///
/// switch point.name {
///     "start" => {
///         print(`starting from: ${point}`)
///     },
///     "finish" => {
///         print(`finishing at: ${point}`)
///     },
///     _ => {
///         throw error("unknown point");
///     }
/// }
/// ```
fn current_point(): Point;

/// Get all the points in an array.
fn all_points(): [Point];

/// Some point that can be modified.
///
/// It might not always be available.
let some_point: Point | void;
