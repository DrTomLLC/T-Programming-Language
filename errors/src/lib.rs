// errors/src/lib.rs
//! Unified error handling for T-Lang compiler and runtime.
//! Provides structured, user-friendly diagnostics with source spans.

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

/// All possible errors in the T-Lang system.
#[derive(Error, Diagnostic, Debug)]
pub enum TlError {
    #[error("Lexical error: {message}")]
    #[diagnostic(
        code(t::lexer),
        help("Check for invalid characters or malformed tokens")
    )]
    Lexer {
        #[source_code]
        src: String,
        #[label("invalid token here")]
        span: SourceSpan,
        message: String,
    },

    #[error("Parse error: {message}")]
    #[diagnostic(
        code(t::parser),
        help("Check syntax against T-Lang grammar specification")
    )]
    Parser {
        #[source_code]
        src: String,
        #[label("unexpected token")]
        span: SourceSpan,
        message: String,
    },

    #[error("Type error: {message}")]
    #[diagnostic(
        code(t::types),
        help("Ensure all expressions have compatible types")
    )]
    Type {
        #[source_code]
        src: String,
        #[label("type mismatch")]
        span: SourceSpan,
        message: String,
    },

    #[error("Safety violation: {message}")]
    #[diagnostic(
        code(t::safety),
        help("Use explicit unsafe blocks for potentially dangerous operations")
    )]
    Safety {
        #[source_code]
        src: String,
        #[label("unsafe operation")]
        span: SourceSpan,
        message: String,
    },

    #[error("Runtime error: {message}")]
    #[diagnostic(
        code(t::runtime),
        help("Check for logic errors or invalid runtime state")
    )]
    Runtime {
        #[source_code]
        src: String,
        #[label("error occurred here")]
        span: SourceSpan,
        message: String,
    },

    #[error("I/O error: {message}")]
    #[diagnostic(code(t::io))]
    Io {
        message: String,
        #[diagnostic(skip)]
        source: Option<std::io::Error>,
    },

    #[error("Internal compiler error: {message}")]
    #[diagnostic(
        code(t::internal),
        help("This is a bug in the T-Lang compiler. Please report it.")
    )]
    Internal {
        message: String,
        location: String,
    },
}

/// Error severity levels for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Result type for T-Lang operations.
pub type Result<T> = std::result::Result<T, TlError>;

impl TlError {
    /// Create a lexer error with source context.
    pub fn lexer(src: impl Into<String>, span: impl Into<SourceSpan>, message: impl Into<String>) -> Self {
        Self::Lexer {
            src: src.into(),
            span: span.into(),
            message: message.into(),
        }
    }

    /// Create a parser error with source context.
    pub fn parser(src: impl Into<String>, span: impl Into<SourceSpan>, message: impl Into<String>) -> Self {
        Self::Parser {
            src: src.into(),
            span: span.into(),
            message: message.into(),
        }
    }

    /// Create a type error with source context.
    pub fn type_error(src: impl Into<String>, span: impl Into<SourceSpan>, message: impl Into<String>) -> Self {
        Self::Type {
            src: src.into(),
            span: span.into(),
            message: message.into(),
        }
    }

    /// Create a safety violation error.
    pub fn safety(src: impl Into<String>, span: impl Into<SourceSpan>, message: impl Into<String>) -> Self {
        Self::Safety {
            src: src.into(),
            span: span.into(),
            message: message.into(),
        }
    }

    /// Create a runtime error.
    pub fn runtime(src: impl Into<String>, span: impl Into<SourceSpan>, message: impl Into<String>) -> Self {
        Self::Runtime {
            src: src.into(),
            span: span.into(),
            message: message.into(),
        }
    }

    /// Create an I/O error.
    pub fn io(message: impl Into<String>, source: Option<std::io::Error>) -> Self {
        Self::Io {
            message: message.into(),
            source,
        }
    }

    /// Create an internal compiler error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            location: std::panic::Location::caller().to_string(),
        }
    }
}