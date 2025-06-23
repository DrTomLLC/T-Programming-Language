// compiler/src/runtime/error.rs

use std::fmt;

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    /// e.g. applying `-` to a non‑number, or `"a" + 1`
    TypeError(String),

    /// variable wasn’t found in the environment
    UndefinedVariable(String),

    /// calling a function with the wrong number of args
    WrongArity(String, usize /* expected */, usize /* got */),

    /// generic argument error (you can migrate uses of this into `WrongArity`)
    ArgumentError(String),

    /// hit a recursion cutoff
    RecursionLimitExceeded,

    /// catch‑all
    Custom(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::TypeError(msg) =>
                write!(f, "Type error: {}", msg),

            RuntimeError::UndefinedVariable(name) =>
                write!(f, "Undefined variable: {}", name),

            RuntimeError::WrongArity(fn_name, expected, got) =>
                write!(
                    f,
                    "Wrong number of arguments to `{}`: expected {}, got {}",
                    fn_name, expected, got
                ),

            RuntimeError::ArgumentError(msg) =>
                write!(f, "Argument error: {}", msg),

            RuntimeError::RecursionLimitExceeded =>
                write!(f, "Recursion limit exceeded"),

            RuntimeError::Custom(msg) =>
                write!(f, "Runtime error: {}", msg),
        }
    }
}

impl std::error::Error for RuntimeError {}
