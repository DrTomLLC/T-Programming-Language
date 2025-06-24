// errors/src/lib.rs
//! Unified error handling for T-Lang compiler and runtime.
//! Provides structured, user-friendly diagnostics with source spans.

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

/// All possible errors in the T-Lang system.
#[derive(Error, Diagnostic, Debug, Clone)]
pub enum TlError {
    #[error("Lexical error: {message}")]
    #[diagnostic(
        code(tlang::lexer),
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
        code(tlang::parser),
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
        code(tlang::types),
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
        code(tlang::safety),
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
        code(tlang::runtime),
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
    #[diagnostic(code(tlang::io))]
    Io {
        message: String,
        #[diagnostic(skip)]
        source: Option<std::io::Error>,
    },

    #[error("Internal compiler error: {message}")]
    #[diagnostic(
        code(tlang::internal),
        help("This is a bug in the T-Lang compiler. Please report it.")
    )]
    Internal {
        message: String,
        location: String,
    },

    #[error("Name resolution error: {message}")]
    #[diagnostic(
        code(tlang::resolution),
        help("Check that all used names are properly declared and in scope")
    )]
    NameResolution {
        #[source_code]
        src: String,
        #[label("unresolved name")]
        span: SourceSpan,
        message: String,
    },

    #[error("Borrow check error: {message}")]
    #[diagnostic(
        code(tlang::borrow),
        help("Ensure all borrows follow ownership rules")
    )]
    BorrowCheck {
        #[source_code]
        src: String,
        #[label("borrow check failed")]
        span: SourceSpan,
        message: String,
    },

    #[error("Module system error: {message}")]
    #[diagnostic(
        code(tlang::module),
        help("Check module paths and visibility modifiers")
    )]
    Module {
        #[source_code]
        src: String,
        #[label("module error")]
        span: SourceSpan,
        message: String,
    },

    #[error("Code generation error: {message}")]
    #[diagnostic(
        code(tlang::codegen),
        help("This may be due to unsupported language features")
    )]
    Codegen {
        message: String,
        context: Option<String>,
    },

    #[error("Plugin error: {message}")]
    #[diagnostic(
        code(tlang::plugin),
        help("Check plugin compatibility and configuration")
    )]
    Plugin {
        message: String,
        plugin_name: String,
    },

    #[error("Configuration error: {message}")]
    #[diagnostic(code(tlang::config))]
    Config {
        message: String,
    },

    #[error("Multiple errors occurred")]
    #[diagnostic(code(tlang::multiple))]
    Multiple {
        errors: Vec<TlError>,
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

    /// Create a safety error with source context.
    pub fn safety(src: impl Into<String>, span: impl Into<SourceSpan>, message: impl Into<String>) -> Self {
        Self::Safety {
            src: src.into(),
            span: span.into(),
            message: message.into(),
        }
    }

    /// Create a runtime error with source context.
    pub fn runtime(src: impl Into<String>, span: impl Into<SourceSpan>, message: impl Into<String>) -> Self {
        Self::Runtime {
            src: src.into(),
            span: span.into(),
            message: message.into(),
        }
    }

    /// Create an I/O error.
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
            source: None,
        }
    }

    /// Create an I/O error with source.
    pub fn io_with_source(message: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            message: message.into(),
            source: Some(source),
        }
    }

    /// Create an internal compiler error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            location: std::panic::Location::caller().to_string(),
        }
    }

    /// Create a name resolution error.
    pub fn name_resolution(src: impl Into<String>, span: impl Into<SourceSpan>, message: impl Into<String>) -> Self {
        Self::NameResolution {
            src: src.into(),
            span: span.into(),
            message: message.into(),
        }
    }

    /// Create a borrow check error.
    pub fn borrow_check(src: impl Into<String>, span: impl Into<SourceSpan>, message: impl Into<String>) -> Self {
        Self::BorrowCheck {
            src: src.into(),
            span: span.into(),
            message: message.into(),
        }
    }

    /// Create a module system error.
    pub fn module(src: impl Into<String>, span: impl Into<SourceSpan>, message: impl Into<String>) -> Self {
        Self::Module {
            src: src.into(),
            span: span.into(),
            message: message.into(),
        }
    }

    /// Create a code generation error.
    pub fn codegen(message: impl Into<String>) -> Self {
        Self::Codegen {
            message: message.into(),
            context: None,
        }
    }

    /// Create a code generation error with context.
    pub fn codegen_with_context(message: impl Into<String>, context: impl Into<String>) -> Self {
        Self::Codegen {
            message: message.into(),
            context: Some(context.into()),
        }
    }

    /// Create a plugin error.
    pub fn plugin(plugin_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Plugin {
            message: message.into(),
            plugin_name: plugin_name.into(),
        }
    }

    /// Create a configuration error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Combine multiple errors into one.
    pub fn multiple(errors: Vec<TlError>) -> Self {
        Self::Multiple { errors }
    }

    /// Get the severity of this error.
    pub fn severity(&self) -> Severity {
        match self {
            TlError::Internal { .. } => Severity::Error,
            TlError::Lexer { .. } => Severity::Error,
            TlError::Parser { .. } => Severity::Error,
            TlError::Type { .. } => Severity::Error,
            TlError::Safety { .. } => Severity::Error,
            TlError::Runtime { .. } => Severity::Error,
            TlError::NameResolution { .. } => Severity::Error,
            TlError::BorrowCheck { .. } => Severity::Error,
            TlError::Module { .. } => Severity::Error,
            TlError::Codegen { .. } => Severity::Error,
            TlError::Plugin { .. } => Severity::Warning,
            TlError::Io { .. } => Severity::Error,
            TlError::Config { .. } => Severity::Error,
            TlError::Multiple { .. } => Severity::Error,
        }
    }

    /// Check if this is a fatal error that should stop compilation.
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            TlError::Internal { .. } | TlError::Io { .. } | TlError::Config { .. }
        )
    }

    /// Get the diagnostic code for this error.
    pub fn code(&self) -> &'static str {
        match self {
            TlError::Lexer { .. } => "E0001",
            TlError::Parser { .. } => "E0002",
            TlError::Type { .. } => "E0003",
            TlError::Safety { .. } => "E0004",
            TlError::Runtime { .. } => "E0005",
            TlError::NameResolution { .. } => "E0006",
            TlError::BorrowCheck { .. } => "E0007",
            TlError::Module { .. } => "E0008",
            TlError::Codegen { .. } => "E0009",
            TlError::Plugin { .. } => "E0010",
            TlError::Io { .. } => "E0011",
            TlError::Config { .. } => "E0012",
            TlError::Internal { .. } => "E9999",
            TlError::Multiple { .. } => "E0000",
        }
    }

    /// Get the source span for this error, if available.
    pub fn span(&self) -> Option<SourceSpan> {
        match self {
            TlError::Lexer { span, .. } => Some(*span),
            TlError::Parser { span, .. } => Some(*span),
            TlError::Type { span, .. } => Some(*span),
            TlError::Safety { span, .. } => Some(*span),
            TlError::Runtime { span, .. } => Some(*span),
            TlError::NameResolution { span, .. } => Some(*span),
            TlError::BorrowCheck { span, .. } => Some(*span),
            TlError::Module { span, .. } => Some(*span),
            _ => None,
        }
    }

    /// Add context to this error.
    pub fn with_context(self, context: impl Into<String>) -> Self {
        match self {
            TlError::Codegen { message, context: _ } => TlError::Codegen {
                message,
                context: Some(context.into()),
            },
            other => other,
        }
    }
}

impl From<std::io::Error> for TlError {
    fn from(error: std::io::Error) -> Self {
        TlError::io_with_source("I/O operation failed", error)
    }
}

impl From<serde_json::Error> for TlError {
    fn from(error: serde_json::Error) -> Self {
        TlError::config(format!("JSON parsing error: {}", error))
    }
}

/// Error collection for gathering multiple errors during compilation.
#[derive(Debug, Clone)]
pub struct ErrorCollector {
    errors: Vec<TlError>,
    max_errors: usize,
}

impl ErrorCollector {
    /// Create a new error collector.
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            max_errors: 100,
        }
    }

    /// Create an error collector with a maximum error count.
    pub fn with_limit(max_errors: usize) -> Self {
        Self {
            errors: Vec::new(),
            max_errors,
        }
    }

    /// Add an error to the collection.
    pub fn add(&mut self, error: TlError) {
        if self.errors.len() < self.max_errors {
            self.errors.push(error);
        }
    }

    /// Add multiple errors to the collection.
    pub fn extend(&mut self, errors: impl IntoIterator<Item = TlError>) {
        for error in errors {
            self.add(error);
            if self.errors.len() >= self.max_errors {
                break;
            }
        }
    }

    /// Check if any errors have been collected.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the number of errors collected.
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Check if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get a reference to all collected errors.
    pub fn errors(&self) -> &[TlError] {
        &self.errors
    }

    /// Take all collected errors, leaving the collector empty.
    pub fn take_errors(&mut self) -> Vec<TlError> {
        std::mem::take(&mut self.errors)
    }

    /// Convert to a result, returning the first error if any.
    pub fn into_result<T>(self, value: T) -> Result<T> {
        if self.errors.is_empty() {
            Ok(value)
        } else if self.errors.len() == 1 {
            Err(self.errors.into_iter().next().unwrap())
        } else {
            Err(TlError::multiple(self.errors))
        }
    }

    /// Convert to a result with a custom error for multiple errors.
    pub fn into_result_with<T, F>(self, value: T, f: F) -> Result<T>
    where
        F: FnOnce(Vec<TlError>) -> TlError,
    {
        if self.errors.is_empty() {
            Ok(value)
        } else if self.errors.len() == 1 {
            Err(self.errors.into_iter().next().unwrap())
        } else {
            Err(f(self.errors))
        }
    }
}

impl Default for ErrorCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper module for creating parse errors.
pub mod parse {
    use super::*;

    pub fn missing_token(
        file: &str,
        source: &str,
        span: SourceSpan,
        expected: &str,
    ) -> TlError {
        TlError::parser(
            source.to_string(),
            span,
            format!("Expected {}, found end of input", expected),
        )
    }

    pub fn unexpected_token(
        source: &str,
        span: SourceSpan,
        found: &str,
        expected: &str,
    ) -> TlError {
        TlError::parser(
            source.to_string(),
            span,
            format!("Expected {}, found {}", expected, found),
        )
    }

    pub fn invalid_syntax(source: &str, span: SourceSpan, message: &str) -> TlError {
        TlError::parser(source.to_string(), span, message.to_string())
    }
}

/// Helper module for creating type errors.
pub mod types {
    use super::*;

    pub fn type_mismatch(
        source: &str,
        span: SourceSpan,
        expected: &str,
        found: &str,
    ) -> TlError {
        TlError::type_error(
            source.to_string(),
            span,
            format!("Type mismatch: expected {}, found {}", expected, found),
        )
    }

    pub fn undefined_variable(source: &str, span: SourceSpan, name: &str) -> TlError {
        TlError::name_resolution(
            source.to_string(),
            span,
            format!("Undefined variable: {}", name),
        )
    }

    pub fn undefined_function(source: &str, span: SourceSpan, name: &str) -> TlError {
        TlError::name_resolution(
            source.to_string(),
            span,
            format!("Undefined function: {}", name),
        )
    }

    pub fn arity_mismatch(
        source: &str,
        span: SourceSpan,
        expected: usize,
        found: usize,
    ) -> TlError {
        TlError::type_error(
            source.to_string(),
            span,
            format!(
                "Argument count mismatch: expected {}, found {}",
                expected, found
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let span = SourceSpan::new(0.into(), 5);
        let error = TlError::lexer("source", span, "test error");

        assert_eq!(error.code(), "E0001");
        assert_eq!(error.severity(), Severity::Error);
        assert!(!error.is_fatal());
        assert_eq!(error.span(), Some(span));
    }

    #[test]
    fn test_error_collector() {
        let mut collector = ErrorCollector::new();
        assert!(collector.is_empty());

        collector.add(TlError::config("test error"));
        assert!(collector.has_errors());
        assert_eq!(collector.len(), 1);

        let result: Result<()> = collector.into_result(());
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_errors() {
        let errors = vec![
            TlError::config("error 1"),
            TlError::config("error 2"),
        ];

        let multiple = TlError::multiple(errors);
        assert_eq!(multiple.code(), "E0000");
    }
}