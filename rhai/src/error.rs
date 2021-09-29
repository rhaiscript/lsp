//! Module containing error definitions for the evaluation process.

use crate::{Dynamic, ImmutableString, ParseErrorType, Position, INT};
#[cfg(feature = "no_std")]
use core_error::Error;
#[cfg(not(feature = "no_std"))]
use std::error::Error;
use std::fmt;
#[cfg(feature = "no_std")]
use std::prelude::v1::*;

/// Evaluation result.
///
/// All wrapped [`Position`] values represent the location in the script where the error occurs.
///
/// # Thread Safety
///
/// Currently, [`EvalAltResult`] is neither [`Send`] nor [`Sync`].
/// Turn on the `sync` feature to make it [`Send`] `+` [`Sync`].
#[derive(Debug)]
#[non_exhaustive]
pub enum EvalAltResult {
    /// System error. Wrapped values are the error message and the internal error.
    #[cfg(not(feature = "sync"))]
    ErrorSystem(String, Box<dyn Error>),
    /// System error. Wrapped values are the error message and the internal error.
    #[cfg(feature = "sync")]
    ErrorSystem(String, Box<dyn Error + Send + Sync>),

    /// Syntax error.
    ErrorParsing(ParseErrorType, Position),

    /// Usage of an unknown variable. Wrapped value is the variable name.
    ErrorVariableNotFound(String, Position),
    /// Call to an unknown function. Wrapped value is the function signature.
    ErrorFunctionNotFound(String, Position),
    /// An error has occurred inside a called function.
    /// Wrapped values are the function name, function source, and the interior error.
    ErrorInFunctionCall(String, String, Box<EvalAltResult>, Position),
    /// Usage of an unknown [module][crate::Module]. Wrapped value is the [module][crate::Module] name.
    ErrorModuleNotFound(String, Position),
    /// An error has occurred while loading a [module][crate::Module].
    /// Wrapped value are the [module][crate::Module] name and the interior error.
    ErrorInModule(String, Box<EvalAltResult>, Position),
    /// Access to `this` that is not bound.
    ErrorUnboundThis(Position),
    /// Data is not of the required type.
    /// Wrapped values are the type requested and type of the actual result.
    ErrorMismatchDataType(String, String, Position),
    /// Returned type is not the same as the required output type.
    /// Wrapped values are the type requested and type of the actual result.
    ErrorMismatchOutputType(String, String, Position),
    /// Array access out-of-bounds.
    /// Wrapped values are the current number of elements in the array and the index number.
    ErrorArrayBounds(usize, INT, Position),
    /// String indexing out-of-bounds.
    /// Wrapped values are the current number of characters in the string and the index number.
    ErrorStringBounds(usize, INT, Position),
    /// Bit-field indexing out-of-bounds.
    /// Wrapped values are the current number of bits in the bit-field and the index number.
    ErrorBitFieldBounds(usize, INT, Position),
    /// Trying to index into a type that has no indexer function defined. Wrapped value is the type name.
    ErrorIndexingType(String, Position),
    /// The `for` statement encounters a type that is not an iterator.
    ErrorFor(Position),
    /// Data race detected when accessing a variable. Wrapped value is the variable name.
    ErrorDataRace(String, Position),
    /// Assignment to a constant variable. Wrapped value is the variable name.
    ErrorAssignmentToConstant(String, Position),
    /// Inappropriate property access. Wrapped value is the property name.
    ErrorDotExpr(String, Position),
    /// Arithmetic error encountered. Wrapped value is the error message.
    ErrorArithmetic(String, Position),
    /// Number of operations over maximum limit.
    ErrorTooManyOperations(Position),
    /// [Modules][crate::Module] over maximum limit.
    ErrorTooManyModules(Position),
    /// Call stack over maximum limit.
    ErrorStackOverflow(Position),
    /// Data value over maximum size limit. Wrapped value is the type name.
    ErrorDataTooLarge(String, Position),
    /// The script is prematurely terminated. Wrapped value is the termination token.
    ErrorTerminated(Dynamic, Position),
    /// Run-time error encountered. Wrapped value is the error token.
    ErrorRuntime(Dynamic, Position),

    /// Breaking out of loops - not an error if within a loop.
    /// The wrapped value, if true, means breaking clean out of the loop (i.e. a `break` statement).
    /// The wrapped value, if false, means breaking the current context (i.e. a `continue` statement).
    LoopBreak(bool, Position),
    /// Not an error: Value returned from a script via the `return` keyword.
    /// Wrapped value is the result value.
    Return(Dynamic, Position),
}

impl Error for EvalAltResult {}

impl fmt::Display for EvalAltResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ErrorSystem(s, err) => match s.as_str() {
                "" => write!(f, "{}", err),
                s => write!(f, "{}: {}", s, err),
            }?,

            Self::ErrorParsing(p, _) => write!(f, "Syntax error: {}", p)?,

            #[cfg(not(feature = "no_function"))]
            Self::ErrorInFunctionCall(s, src, err, _) if crate::engine::is_anonymous_fn(s) => {
                write!(f, "{} in call to closure", err)?;
                if !src.is_empty() {
                    write!(f, " @ '{}'", src)?;
                }
            }
            Self::ErrorInFunctionCall(s, src, err, _) => {
                write!(f, "{} in call to function {}", err, s)?;
                if !src.is_empty() {
                    write!(f, " @ '{}'", src)?;
                }
            }

            Self::ErrorInModule(s, err, _) if s.is_empty() => {
                write!(f, "Error in module: {}", err)?
            }
            Self::ErrorInModule(s, err, _) => write!(f, "Error in module '{}': {}", s, err)?,

            Self::ErrorFunctionNotFound(s, _) => write!(f, "Function not found: {}", s)?,
            Self::ErrorVariableNotFound(s, _) => write!(f, "Variable not found: {}", s)?,
            Self::ErrorModuleNotFound(s, _) => write!(f, "Module not found: '{}'", s)?,
            Self::ErrorDataRace(s, _) => {
                write!(f, "Data race detected when accessing variable: {}", s)?
            }
            Self::ErrorDotExpr(s, _) => match s.as_str() {
                "" => f.write_str("Malformed dot expression"),
                s => f.write_str(s),
            }?,
            Self::ErrorIndexingType(s, _) => write!(f, "Indexer not registered for '{}'", s)?,
            Self::ErrorUnboundThis(_) => f.write_str("'this' is not bound")?,
            Self::ErrorFor(_) => f.write_str("For loop expects a type with an iterator defined")?,
            Self::ErrorTooManyOperations(_) => f.write_str("Too many operations")?,
            Self::ErrorTooManyModules(_) => f.write_str("Too many modules imported")?,
            Self::ErrorStackOverflow(_) => f.write_str("Stack overflow")?,
            Self::ErrorTerminated(_, _) => f.write_str("Script terminated")?,

            Self::ErrorRuntime(d, _) if d.is::<()>() => f.write_str("Runtime error")?,
            Self::ErrorRuntime(d, _)
                if d.read_lock::<ImmutableString>()
                    .map_or(false, |v| v.is_empty()) =>
            {
                write!(f, "Runtime error")?
            }
            Self::ErrorRuntime(d, _) => write!(f, "Runtime error: {}", d)?,

            Self::ErrorAssignmentToConstant(s, _) => write!(f, "Cannot modify constant {}", s)?,
            Self::ErrorMismatchOutputType(s, r, _) => match (r.as_str(), s.as_str()) {
                ("", s) => write!(f, "Output type is incorrect, expecting {}", s),
                (r, "") => write!(f, "Output type is incorrect: {}", r),
                (r, s) => write!(f, "Output type is incorrect: {} (expecting {})", r, s),
            }?,
            Self::ErrorMismatchDataType(s, r, _) => match (r.as_str(), s.as_str()) {
                ("", s) => write!(f, "Data type is incorrect, expecting {}", s),
                (r, "") => write!(f, "Data type is incorrect: {}", r),
                (r, s) => write!(f, "Data type is incorrect: {} (expecting {})", r, s),
            }?,
            Self::ErrorArithmetic(s, _) => match s.as_str() {
                "" => f.write_str("Arithmetic error"),
                s => f.write_str(s),
            }?,

            Self::LoopBreak(true, _) => f.write_str("'break' not inside a loop")?,
            Self::LoopBreak(false, _) => f.write_str("'continue' not inside a loop")?,

            Self::Return(_, _) => f.write_str("NOT AN ERROR - function returns value")?,

            Self::ErrorArrayBounds(max, index, _) => match max {
                0 => write!(f, "Array index {} out of bounds: array is empty", index),
                1 => write!(
                    f,
                    "Array index {} out of bounds: only 1 element in the array",
                    index
                ),
                _ => write!(
                    f,
                    "Array index {} out of bounds: only {} elements in the array",
                    index, max
                ),
            }?,
            Self::ErrorStringBounds(max, index, _) => match max {
                0 => write!(f, "String index {} out of bounds: string is empty", index),
                1 => write!(
                    f,
                    "String index {} out of bounds: only 1 character in the string",
                    index
                ),
                _ => write!(
                    f,
                    "String index {} out of bounds: only {} characters in the string",
                    index, max
                ),
            }?,
            Self::ErrorBitFieldBounds(max, index, _) => write!(
                f,
                "Bit-field index {} out of bounds: only {} bits in the bit-field",
                index, max
            )?,
            Self::ErrorDataTooLarge(typ, _) => write!(f, "{} exceeds maximum limit", typ)?,
        }

        // Do not write any position if None
        if !self.position().is_none() {
            write!(f, " ({})", self.position())?;
        }

        Ok(())
    }
}

impl<T: AsRef<str>> From<T> for EvalAltResult {
    #[inline(always)]
    fn from(err: T) -> Self {
        Self::ErrorRuntime(err.as_ref().to_string().into(), Position::NONE)
    }
}

impl<T: AsRef<str>> From<T> for Box<EvalAltResult> {
    #[inline(always)]
    fn from(err: T) -> Self {
        Box::new(EvalAltResult::ErrorRuntime(
            err.as_ref().to_string().into(),
            Position::NONE,
        ))
    }
}

impl EvalAltResult {
    /// Is this a pseudo error?  A pseudo error is one that does not occur naturally.
    ///
    /// [`LoopBreak`][EvalAltResult::LoopBreak] and [`Return`][EvalAltResult::Return] are pseudo errors.
    #[must_use]
    pub const fn is_pseudo_error(&self) -> bool {
        match self {
            Self::LoopBreak(_, _) | Self::Return(_, _) => true,
            _ => false,
        }
    }
    /// Can this error be caught?
    #[must_use]
    pub const fn is_catchable(&self) -> bool {
        match self {
            Self::ErrorSystem(_, _) => false,
            Self::ErrorParsing(_, _) => false,

            Self::ErrorFunctionNotFound(_, _)
            | Self::ErrorInFunctionCall(_, _, _, _)
            | Self::ErrorInModule(_, _, _)
            | Self::ErrorUnboundThis(_)
            | Self::ErrorMismatchDataType(_, _, _)
            | Self::ErrorArrayBounds(_, _, _)
            | Self::ErrorStringBounds(_, _, _)
            | Self::ErrorBitFieldBounds(_, _, _)
            | Self::ErrorIndexingType(_, _)
            | Self::ErrorFor(_)
            | Self::ErrorVariableNotFound(_, _)
            | Self::ErrorModuleNotFound(_, _)
            | Self::ErrorDataRace(_, _)
            | Self::ErrorAssignmentToConstant(_, _)
            | Self::ErrorMismatchOutputType(_, _, _)
            | Self::ErrorDotExpr(_, _)
            | Self::ErrorArithmetic(_, _)
            | Self::ErrorRuntime(_, _) => true,

            Self::ErrorTooManyOperations(_)
            | Self::ErrorTooManyModules(_)
            | Self::ErrorStackOverflow(_)
            | Self::ErrorDataTooLarge(_, _)
            | Self::ErrorTerminated(_, _) => false,

            Self::LoopBreak(_, _) | Self::Return(_, _) => false,
        }
    }
    /// Is this error a system exception?
    #[must_use]
    pub const fn is_system_exception(&self) -> bool {
        match self {
            Self::ErrorSystem(_, _) => true,
            Self::ErrorParsing(_, _) => true,

            Self::ErrorTooManyOperations(_)
            | Self::ErrorTooManyModules(_)
            | Self::ErrorStackOverflow(_)
            | Self::ErrorDataTooLarge(_, _) => true,

            Self::ErrorTerminated(_, _) => true,

            _ => false,
        }
    }
    /// Get the [position][Position] of this error.
    #[cfg(not(feature = "no_object"))]
    pub(crate) fn dump_fields(&self, map: &mut crate::Map) {
        map.insert(
            "error".into(),
            format!("{:?}", self)
                .split('(')
                .next()
                .expect("debug format of error is `ErrorXXX(...)`")
                .into(),
        );

        match self {
            Self::LoopBreak(_, _) | Self::Return(_, _) => (),

            Self::ErrorSystem(_, _)
            | Self::ErrorParsing(_, _)
            | Self::ErrorUnboundThis(_)
            | Self::ErrorFor(_)
            | Self::ErrorArithmetic(_, _)
            | Self::ErrorTooManyOperations(_)
            | Self::ErrorTooManyModules(_)
            | Self::ErrorStackOverflow(_)
            | Self::ErrorRuntime(_, _) => (),

            Self::ErrorFunctionNotFound(f, _) => {
                map.insert("function".into(), f.into());
            }
            Self::ErrorInFunctionCall(f, s, _, _) => {
                map.insert("function".into(), f.into());
                map.insert("source".into(), s.into());
            }
            Self::ErrorInModule(m, _, _) => {
                map.insert("module".into(), m.into());
            }
            Self::ErrorMismatchDataType(r, a, _) | Self::ErrorMismatchOutputType(r, a, _) => {
                map.insert("requested".into(), r.into());
                map.insert("actual".into(), a.into());
            }
            Self::ErrorArrayBounds(n, i, _)
            | Self::ErrorStringBounds(n, i, _)
            | Self::ErrorBitFieldBounds(n, i, _) => {
                map.insert("length".into(), (*n as INT).into());
                map.insert("index".into(), (*i as INT).into());
            }
            Self::ErrorIndexingType(t, _) => {
                map.insert("type".into(), t.into());
            }
            Self::ErrorVariableNotFound(v, _)
            | Self::ErrorDataRace(v, _)
            | Self::ErrorAssignmentToConstant(v, _) => {
                map.insert("variable".into(), v.into());
            }
            Self::ErrorModuleNotFound(m, _) => {
                map.insert("module".into(), m.into());
            }
            Self::ErrorDotExpr(p, _) => {
                map.insert("property".into(), p.into());
            }

            Self::ErrorDataTooLarge(t, _) => {
                map.insert("type".into(), t.into());
            }
            Self::ErrorTerminated(t, _) => {
                map.insert("token".into(), t.clone());
            }
        };
    }
    /// Get the [position][Position] of this error.
    #[must_use]
    pub const fn position(&self) -> Position {
        match self {
            Self::ErrorSystem(_, _) => Position::NONE,

            Self::ErrorParsing(_, pos)
            | Self::ErrorFunctionNotFound(_, pos)
            | Self::ErrorInFunctionCall(_, _, _, pos)
            | Self::ErrorInModule(_, _, pos)
            | Self::ErrorUnboundThis(pos)
            | Self::ErrorMismatchDataType(_, _, pos)
            | Self::ErrorArrayBounds(_, _, pos)
            | Self::ErrorStringBounds(_, _, pos)
            | Self::ErrorBitFieldBounds(_, _, pos)
            | Self::ErrorIndexingType(_, pos)
            | Self::ErrorFor(pos)
            | Self::ErrorVariableNotFound(_, pos)
            | Self::ErrorModuleNotFound(_, pos)
            | Self::ErrorDataRace(_, pos)
            | Self::ErrorAssignmentToConstant(_, pos)
            | Self::ErrorMismatchOutputType(_, _, pos)
            | Self::ErrorDotExpr(_, pos)
            | Self::ErrorArithmetic(_, pos)
            | Self::ErrorTooManyOperations(pos)
            | Self::ErrorTooManyModules(pos)
            | Self::ErrorStackOverflow(pos)
            | Self::ErrorDataTooLarge(_, pos)
            | Self::ErrorTerminated(_, pos)
            | Self::ErrorRuntime(_, pos)
            | Self::LoopBreak(_, pos)
            | Self::Return(_, pos) => *pos,
        }
    }
    /// Remove the [position][Position] information from this error.
    ///
    /// The [position][Position] of this error is set to [`NONE`][Position::NONE] afterwards.
    pub fn clear_position(&mut self) -> &mut Self {
        self.set_position(Position::NONE)
    }
    /// Remove the [position][Position] information from this error and return it.
    ///
    /// The [position][Position] of this error is set to [`NONE`][Position::NONE] afterwards.
    pub fn take_position(&mut self) -> Position {
        let pos = self.position();
        self.set_position(Position::NONE);
        pos
    }
    /// Override the [position][Position] of this error.
    pub fn set_position(&mut self, new_position: Position) -> &mut Self {
        match self {
            Self::ErrorSystem(_, _) => (),

            Self::ErrorParsing(_, pos)
            | Self::ErrorFunctionNotFound(_, pos)
            | Self::ErrorInFunctionCall(_, _, _, pos)
            | Self::ErrorInModule(_, _, pos)
            | Self::ErrorUnboundThis(pos)
            | Self::ErrorMismatchDataType(_, _, pos)
            | Self::ErrorArrayBounds(_, _, pos)
            | Self::ErrorStringBounds(_, _, pos)
            | Self::ErrorBitFieldBounds(_, _, pos)
            | Self::ErrorIndexingType(_, pos)
            | Self::ErrorFor(pos)
            | Self::ErrorVariableNotFound(_, pos)
            | Self::ErrorModuleNotFound(_, pos)
            | Self::ErrorDataRace(_, pos)
            | Self::ErrorAssignmentToConstant(_, pos)
            | Self::ErrorMismatchOutputType(_, _, pos)
            | Self::ErrorDotExpr(_, pos)
            | Self::ErrorArithmetic(_, pos)
            | Self::ErrorTooManyOperations(pos)
            | Self::ErrorTooManyModules(pos)
            | Self::ErrorStackOverflow(pos)
            | Self::ErrorDataTooLarge(_, pos)
            | Self::ErrorTerminated(_, pos)
            | Self::ErrorRuntime(_, pos)
            | Self::LoopBreak(_, pos)
            | Self::Return(_, pos) => *pos = new_position,
        }
        self
    }
    /// Consume the current [`EvalAltResult`] and return a new one with the specified [`Position`]
    /// if the current position is [`Position::None`].
    #[inline]
    #[must_use]
    pub(crate) fn fill_position(mut self: Box<Self>, new_position: Position) -> Box<Self> {
        if self.position().is_none() {
            self.set_position(new_position);
        }
        self
    }
}

impl<T> From<EvalAltResult> for Result<T, Box<EvalAltResult>> {
    #[inline(always)]
    fn from(err: EvalAltResult) -> Self {
        Err(err.into())
    }
}
