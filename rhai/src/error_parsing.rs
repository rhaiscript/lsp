//! Module containing error definitions for the parsing process.

use crate::{EvalAltResult, Position};
#[cfg(feature = "no_std")]
use core_error::Error;
#[cfg(not(feature = "no_std"))]
use std::error::Error;
use std::fmt;
#[cfg(feature = "no_std")]
use std::prelude::v1::*;

/// _(internals)_ Error encountered when tokenizing the script text.
/// Exported under the `internals` feature only.
///
/// # Volatile Data Structure
///
/// This type is volatile and may change.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
#[non_exhaustive]
pub enum LexError {
    /// An unexpected symbol is encountered when tokenizing the script text.
    UnexpectedInput(String),
    /// A string literal is not terminated before a new-line or EOF.
    UnterminatedString,
    /// An identifier is in an invalid format.
    StringTooLong(usize),
    /// An string/character/numeric escape sequence is in an invalid format.
    MalformedEscapeSequence(String),
    /// An numeric literal is in an invalid format.
    MalformedNumber(String),
    /// An character literal is in an invalid format.
    MalformedChar(String),
    /// An identifier is in an invalid format.
    MalformedIdentifier(String),
    /// Bad symbol encountered when tokenizing the script text.
    ImproperSymbol(String, String),
}

impl Error for LexError {}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedInput(s) => write!(f, "Unexpected '{}'", s),
            Self::MalformedEscapeSequence(s) => write!(f, "Invalid escape sequence: '{}'", s),
            Self::MalformedNumber(s) => write!(f, "Invalid number: '{}'", s),
            Self::MalformedChar(s) => write!(f, "Invalid character: '{}'", s),
            Self::MalformedIdentifier(s) => write!(f, "Variable name is not proper: '{}'", s),
            Self::UnterminatedString => f.write_str("Open string is not terminated"),
            Self::StringTooLong(max) => write!(
                f,
                "Length of string literal exceeds the maximum limit ({})",
                max
            ),
            Self::ImproperSymbol(s, d) if d.is_empty() => {
                write!(f, "Invalid symbol encountered: '{}'", s)
            }
            Self::ImproperSymbol(_, d) => f.write_str(d),
        }
    }
}

impl LexError {
    /// Convert a [`LexError`] into a [`ParseError`].
    #[inline(always)]
    #[must_use]
    pub fn into_err(self, pos: Position) -> ParseError {
        ParseError(Box::new(self.into()), pos)
    }
}

/// Type of error encountered when parsing a script.
///
/// Some errors never appear when certain features are turned on.
/// They still exist so that the application can turn features on and off without going through
/// massive code changes to remove/add back enum variants in match statements.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
#[non_exhaustive]
pub enum ParseErrorType {
    /// The script ends prematurely.
    UnexpectedEOF,
    /// Error in the script text. Wrapped value is the lex error.
    BadInput(LexError),
    /// An unknown operator is encountered. Wrapped value is the operator.
    UnknownOperator(String),
    /// Expecting a particular token but not finding one. Wrapped values are the token and description.
    MissingToken(String, String),
    /// Expecting a particular symbol but not finding one. Wrapped value is the description.
    MissingSymbol(String),
    /// An expression in function call arguments `()` has syntax error. Wrapped value is the error
    /// description (if any).
    MalformedCallExpr(String),
    /// An expression in indexing brackets `[]` has syntax error. Wrapped value is the error
    /// description (if any).
    ///
    /// Never appears under the `no_index` feature.
    MalformedIndexExpr(String),
    /// An expression in an `in` expression has syntax error. Wrapped value is the error description
    /// (if any).
    ///
    /// Never appears under the `no_object` and `no_index` features combination.
    MalformedInExpr(String),
    /// A capturing  has syntax error. Wrapped value is the error description (if any).
    ///
    /// Never appears under the `no_closure` feature.
    MalformedCapture(String),
    /// A map definition has duplicated property names. Wrapped value is the property name.
    ///
    /// Never appears under the `no_object` feature.
    DuplicatedProperty(String),
    /// A `switch` case is duplicated.
    DuplicatedSwitchCase,
    /// A variable name is duplicated. Wrapped value is the variable name.
    DuplicatedVariable(String),
    /// The default case of a `switch` statement is not the last.
    WrongSwitchDefaultCase,
    /// The case condition of a `switch` statement is not appropriate.
    WrongSwitchCaseCondition,
    /// Missing a property name for custom types and maps.
    ///
    /// Never appears under the `no_object` feature.
    PropertyExpected,
    /// Missing a variable name after the `let`, `const`, `for` or `catch` keywords.
    VariableExpected,
    /// An identifier is a reserved keyword.
    Reserved(String),
    /// An expression is of the wrong type.
    /// Wrapped values are the type requested and type of the actual result.
    MismatchedType(String, String),
    /// Missing an expression. Wrapped value is the expression type.
    ExprExpected(String),
    /// Defining a doc-comment in an appropriate place (e.g. not at global level).
    ///
    /// Never appears under the `no_function` feature.
    WrongDocComment,
    /// Defining a function `fn` in an appropriate place (e.g. inside another function).
    ///
    /// Never appears under the `no_function` feature.
    WrongFnDefinition,
    /// Defining a function with a name that conflicts with an existing function.
    /// Wrapped values are the function name and number of parameters.
    ///
    /// Never appears under the `no_object` feature.
    FnDuplicatedDefinition(String, usize),
    /// Missing a function name after the `fn` keyword.
    ///
    /// Never appears under the `no_function` feature.
    FnMissingName,
    /// A function definition is missing the parameters list. Wrapped value is the function name.
    ///
    /// Never appears under the `no_function` feature.
    FnMissingParams(String),
    /// A function definition has duplicated parameters. Wrapped values are the function name and
    /// parameter name.
    ///
    /// Never appears under the `no_function` feature.
    FnDuplicatedParam(String, String),
    /// A function definition is missing the body. Wrapped value is the function name.
    ///
    /// Never appears under the `no_function` feature.
    FnMissingBody(String),
    /// Export statement not at global level.
    ///
    /// Never appears under the `no_module` feature.
    WrongExport,
    /// Assignment to an a constant variable. Wrapped value is the constant variable name.
    AssignmentToConstant(String),
    /// Assignment to an inappropriate LHS (left-hand-side) expression.
    /// Wrapped value is the error message (if any).
    AssignmentToInvalidLHS(String),
    /// Expression exceeding the maximum levels of complexity.
    ///
    /// Never appears under the `unchecked` feature.
    ExprTooDeep,
    /// Literal exceeding the maximum size. Wrapped values are the data type name and the maximum size.
    ///
    /// Never appears under the `unchecked` feature.
    LiteralTooLarge(String, usize),
    /// Break statement not inside a loop.
    LoopBreak,
}

impl ParseErrorType {
    /// Make a [`ParseError`] using the current type and position.
    #[inline(always)]
    #[must_use]
    pub(crate) fn into_err(self, pos: Position) -> ParseError {
        ParseError(self.into(), pos)
    }
}

impl fmt::Display for ParseErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadInput(err) => write!(f, "{}", err),

            Self::UnknownOperator(s) => write!(f, "Unknown operator: '{}'", s),

            Self::MalformedCallExpr(s) => match s.as_str() {
                "" => f.write_str("Invalid expression in function call arguments"),
                s => f.write_str(s)
            },
            Self::MalformedIndexExpr(s) => match s.as_str() {
                "" => f.write_str("Invalid index in indexing expression"),
                s => f.write_str(s)
            },
            Self::MalformedInExpr(s) => match s.as_str() {
                "" => f.write_str("Invalid 'in' expression"),
                s => f.write_str(s)
            },
            Self::MalformedCapture(s) => match s.as_str() {
                "" => f.write_str("Invalid capturing"),
                s => f.write_str(s)
            },

            Self::FnDuplicatedDefinition(s, n) => {
                write!(f, "Function '{}' with ", s)?;
                match n {
                    0 => f.write_str("no parameters already exists"),
                    1 => f.write_str("1 parameter already exists"),
                    _ => write!(f, "{} parameters already exists", n),
                }
            }
            Self::FnMissingBody(s) => match s.as_str() {
                "" => f.write_str("Expecting body statement block for anonymous function"),
                s => write!(f, "Expecting body statement block for function '{}'", s)
            },
            Self::FnMissingParams(s) => write!(f, "Expecting parameters for function '{}'", s),
            Self::FnDuplicatedParam(s, arg) => write!(f, "Duplicated parameter '{}' for function '{}'", arg, s),

            Self::DuplicatedProperty(s) => write!(f, "Duplicated property '{}' for object map literal", s),
            Self::DuplicatedSwitchCase => f.write_str("Duplicated switch case"),
            Self::DuplicatedVariable(s) => write!(f, "Duplicated variable name '{}'", s),

            Self::MismatchedType(r, a) => write!(f, "Expecting {}, not {}", r, a),
            Self::ExprExpected(s) => write!(f, "Expecting {} expression", s),
            Self::MissingToken(token, s) => write!(f, "Expecting '{}' {}", token, s),

            Self::MissingSymbol(s) if s.is_empty() => f.write_str("Expecting a symbol"),
            Self::MissingSymbol(s) => f.write_str(s),

            Self::AssignmentToConstant(s) => match s.as_str() {
                "" => f.write_str("Cannot assign to a constant value"),
                s => write!(f, "Cannot assign to constant '{}'", s)
            },
            Self::AssignmentToInvalidLHS(s) => match s.as_str() {
                "" => f.write_str("Expression cannot be assigned to"),
                s => f.write_str(s)
            },

            Self::LiteralTooLarge(typ, max) => write!(f, "{} exceeds the maximum limit ({})", typ, max),
            Self::Reserved(s) => write!(f, "'{}' is a reserved keyword", s),
            Self::UnexpectedEOF => f.write_str("Script is incomplete"),
            Self::WrongSwitchDefaultCase => f.write_str("Default switch case is not the last"),
            Self::WrongSwitchCaseCondition => f.write_str("Default switch case cannot have condition"),
            Self::PropertyExpected => f.write_str("Expecting name of a property"),
            Self::VariableExpected => f.write_str("Expecting name of a variable"),
            Self::WrongFnDefinition => f.write_str("Function definitions must be at global level and cannot be inside a block or another function"),
            Self::FnMissingName => f.write_str("Expecting function name in function declaration"),
            Self::WrongDocComment => f.write_str("Doc-comment must be followed immediately by a function definition"),
            Self::WrongExport => f.write_str("Export statement can only appear at global level"),
            Self::ExprTooDeep => f.write_str("Expression exceeds maximum complexity"),
            Self::LoopBreak => f.write_str("Break statement should only be used inside a loop"),
        }
    }
}

impl From<LexError> for ParseErrorType {
    #[inline(always)]
    fn from(err: LexError) -> Self {
        match err {
            LexError::StringTooLong(max) => {
                Self::LiteralTooLarge("Length of string literal".to_string(), max)
            }
            _ => Self::BadInput(err),
        }
    }
}

/// Error when parsing a script.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct ParseError(pub Box<ParseErrorType>, pub Position);

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)?;

        // Do not write any position if None
        if !self.1.is_none() {
            write!(f, " ({})", self.1)?;
        }

        Ok(())
    }
}

impl From<ParseErrorType> for Box<EvalAltResult> {
    #[inline(always)]
    fn from(err: ParseErrorType) -> Self {
        Box::new(err.into())
    }
}

impl From<ParseErrorType> for EvalAltResult {
    #[inline(always)]
    fn from(err: ParseErrorType) -> Self {
        EvalAltResult::ErrorParsing(err, Position::NONE)
    }
}

impl From<ParseError> for Box<EvalAltResult> {
    #[inline(always)]
    fn from(err: ParseError) -> Self {
        Box::new(err.into())
    }
}

impl From<ParseError> for EvalAltResult {
    #[inline(always)]
    fn from(err: ParseError) -> Self {
        EvalAltResult::ErrorParsing(*err.0, err.1)
    }
}
