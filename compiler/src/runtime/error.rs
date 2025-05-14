// runtime/error.rs

use std::fmt;

#[derive(Debug)]
pub enum RuntimeError {
    TypeError(String),
    UndefinedVariable(String),
    ArgumentError(String),
    RecursionLimitExceeded,
    Custom(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::TypeError(msg) => write!(f, "Type error: {}", msg),
            RuntimeError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            RuntimeError::ArgumentError(msg) => write!(f, "Argument error: {}", msg),
            RuntimeError::RecursionLimitExceeded => write!(f, "Recursion limit exceeded"),
            RuntimeError::Custom(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

impl std::error::Error for RuntimeError {}
