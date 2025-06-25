// errors/src/lib.rs
use miette::{Diagnostic, SourceSpan};
use serde_json::value::{Deserialize, Serialize};
use thiserror::Error;

/// The main error type for T-Lang compiler.
#[derive(Debug, Error, Diagnostic, Clone, Serialize, Deserialize)]
pub enum TlError {
    #[error("Lexer error: {message}")]
    #[diagnostic(code(tl::lexer))]
    Lexer {
        message: String,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        source: String,
    },

    #[error("Parser error: {message}")]
    #[diagnostic(code(tl::parser))]
    Parser {
        message: String,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        source: String,
    },

    #[error("Semantic error: {message}")]
    #[diagnostic(code(tl::semantic))]
    Semantic {
        message: String,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        source: String,
    },

    #[error("Type error: {message}")]
    #[diagnostic(code(tl::type))]
    Type {
        message: String,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        source: String,
    },

    #[error("I/O error: {message}")]
    #[diagnostic(code(tl::io))]
    Io {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Internal compiler error: {message}")]
    #[diagnostic(
        code(tl::internal),
        help("This is a bug in the T-Lang compiler. Please report it.")
    )]
    Internal {
        message: String,
        location: String,
    },

    #[error("Runtime error: {message}")]
    #[diagnostic(code(tl::runtime))]
    Runtime {
        message: String,
        #[label("here")]
        span: Option<SourceSpan>,
        #[source_code]
        source: Option<String>,
    },
}

impl TlError {
    /// Create a lexer error.
    pub fn lexer(source: String, span: SourceSpan, message: impl Into<String>) -> Self {
        Self::Lexer {
            message: message.into(),
            span,
            source,
        }
    }

    /// Create a parser error.
    pub fn parser(source: String, span: SourceSpan, message: impl Into<String>) -> Self {
        Self::Parser {
            message: message.into(),
            span,
            source,
        }
    }

    /// Create a semantic error.
    pub fn semantic(source: String, span: SourceSpan, message: impl Into<String>) -> Self {
        Self::Semantic {
            message: message.into(),
            span,
            source,
        }
    }

    /// Create a type error.
    pub fn type_error(source: String, span: SourceSpan, message: impl Into<String>) -> Self {
        Self::Type {
            message: message.into(),
            span,
            source,
        }
    }

    /// Create an I/O error.
    pub fn io(message: impl Into<String>, source: Option<std::io::Error>) -> Self {
        Self::Io {
            message: message.into(),
            source: source.map(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// Create an internal error.
    #[track_caller]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            location: std::panic::Location::caller().to_string(),
        }
    }

    /// Create a runtime error.
    pub fn runtime(message: impl Into<String>, span: Option<SourceSpan>, source: Option<String>) -> Self {
        Self::Runtime {
            message: message.into(),
            span,
            source,
        }
    }
}

/// Error codes for categorizing different types of errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    // Lexer errors
    UnexpectedCharacter,
    UnterminatedString,
    UnterminatedComment,
    InvalidNumber,
    InvalidEscape,

    // Parser errors
    UnexpectedToken,
    ExpectedToken,
    UnexpectedEOF,
    MissingToken,

    // Semantic errors
    UndefinedVariable,
    UndefinedFunction,
    UndefinedType,
    DuplicateDefinition,
    InvalidAssignment,

    // Type errors
    TypeMismatch,
    IncompatibleTypes,
    InvalidOperation,
    UnknownType,

    // Runtime errors
    RuntimeError,
    DivisionByZero,
    IndexOutOfBounds,
    NullPointerDereference,

    // General errors
    Unsupported,
    InvalidInput,
    InternalError,
}

/// Result type alias for T-Lang operations.
pub type Result<T> = std::result::Result<T, TlError>;

/// Error collector for accumulating multiple errors during compilation.
#[derive(Debug, Default)]
pub struct ErrorCollector {
    errors: Vec<TlError>,
    warnings: Vec<TlError>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_error(&mut self, error: TlError) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: TlError) {
        self.warnings.push(warning);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn errors(&self) -> &[TlError] {
        &self.errors
    }

    pub fn warnings(&self) -> &[TlError] {
        &self.warnings
    }

    pub fn clear(&mut self) {
        self.errors.clear();
        self.warnings.clear();
    }

    /// Convert into a single error if there are any errors, or Ok if none.
    pub fn into_result(self) -> Result<()> {
        if let Some(first_error) = self.errors.into_iter().next() {
            Err(first_error)
        } else {
            Ok(())
        }
    }
}