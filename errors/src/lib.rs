// errors/src/lib.rs
use miette::{Diagnostic, SourceSpan};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A wrapper around SourceSpan that supports serialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializableSourceSpan {
    pub offset: usize,
    pub length: usize,
}

impl From<SourceSpan> for SerializableSourceSpan {
    fn from(span: SourceSpan) -> Self {
        Self {
            offset: span.offset(),
            length: span.len(),
        }
    }
}

impl From<SerializableSourceSpan> for SourceSpan {
    fn from(span: SerializableSourceSpan) -> Self {
        SourceSpan::from((span.offset, span.length))
    }
}

/// A serializable error wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableError {
    pub message: String,
    pub kind: String,
}

impl From<Box<dyn std::error::Error + Send + Sync>> for SerializableError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self {
            message: err.to_string(),
            kind: "Error".to_string(),
        }
    }
}

/// The main error type for T-Lang compiler.
#[derive(Debug, Error, Diagnostic, Serialize, Deserialize)]
pub enum TlError {
    #[error("Lexer error: {message}")]
    #[diagnostic(code(tl::lexer))]
    Lexer {
        message: String,
        #[label("here")]
        #[serde(with = "span_serde")]
        span: SourceSpan,
        #[source_code]
        source_code: String,
    },

    #[error("Parser error: {message}")]
    #[diagnostic(code(tl::parser))]
    Parser {
        message: String,
        #[label("here")]
        #[serde(with = "span_serde")]
        span: SourceSpan,
        #[source_code]
        source_code: String,
    },

    #[error("Semantic error: {message}")]
    #[diagnostic(code(tl::semantic))]
    Semantic {
        message: String,
        #[label("here")]
        #[serde(with = "span_serde")]
        span: SourceSpan,
        #[source_code]
        source_code: String,
    },

    #[error("Type error: {message}")]
    #[diagnostic(code(tl::type_error))] // Changed from tl::type to avoid keyword
    TypeError {
        message: String,
        #[label("here")]
        #[serde(with = "span_serde")]
        span: SourceSpan,
        #[source_code]
        source_code: String,
    },

    #[error("I/O error: {message}")]
    #[diagnostic(code(tl::io))]
    Io {
        message: String,
        #[source]
        #[serde(skip)] // Skip serialization for the source error
        source_error: Option<Box<dyn std::error::Error + Send + Sync>>,
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
        #[serde(with = "option_span_serde")]
        span: Option<SourceSpan>,
        #[source_code]
        source_code: Option<String>,
    },
}

// Custom serialization for SourceSpan
mod span_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(span: &SourceSpan, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let serializable_span = SerializableSourceSpan::from(*span);
        serializable_span.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<SourceSpan, D::Error>
    where
        D: Deserializer<'de>,
    {
        let serializable_span = SerializableSourceSpan::deserialize(deserializer)?;
        Ok(SourceSpan::from(serializable_span))
    }
}

// Custom serialization for Option<SourceSpan>
mod option_span_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(span: &Option<SourceSpan>, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match span {
            Some(span) => {
                let serializable_span = SerializableSourceSpan::from(*span);
                Some(serializable_span).serialize(serializer)
            }
            None => None::<SerializableSourceSpan>.serialize(serializer),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Option<SourceSpan>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let serializable_span: Option<SerializableSourceSpan> = Option::deserialize(deserializer)?;
        Ok(serializable_span.map(SourceSpan::from))
    }
}

impl Clone for TlError {
    fn clone(&self) -> Self {
        match self {
            TlError::Lexer { message, span, source_code } => TlError::Lexer {
                message: message.clone(),
                span: *span,
                source_code: source_code.clone(),
            },
            TlError::Parser { message, span, source_code } => TlError::Parser {
                message: message.clone(),
                span: *span,
                source_code: source_code.clone(),
            },
            TlError::Semantic { message, span, source_code } => TlError::Semantic {
                message: message.clone(),
                span: *span,
                source_code: source_code.clone(),
            },
            TlError::TypeError { message, span, source_code } => TlError::TypeError {
                message: message.clone(),
                span: *span,
                source_code: source_code.clone(),
            },
            TlError::Io { message, source_error: _ } => TlError::Io {
                message: message.clone(),
                source_error: None, // Can't clone trait objects, so we skip the source
            },
            TlError::Internal { message, location } => TlError::Internal {
                message: message.clone(),
                location: location.clone(),
            },
            TlError::Runtime { message, span, source_code } => TlError::Runtime {
                message: message.clone(),
                span: *span,
                source_code: source_code.clone(),
            },
        }
    }
}

impl TlError {
    /// Create a lexer error.
    pub fn lexer(source_code: String, span: SourceSpan, message: impl Into<String>) -> Self {
        Self::Lexer {
            message: message.into(),
            span,
            source_code,
        }
    }

    /// Create a parser error.
    pub fn parser(source_code: String, span: SourceSpan, message: impl Into<String>) -> Self {
        Self::Parser {
            message: message.into(),
            span,
            source_code,
        }
    }

    /// Create a semantic error.
    pub fn semantic(source_code: String, span: SourceSpan, message: impl Into<String>) -> Self {
        Self::Semantic {
            message: message.into(),
            span,
            source_code,
        }
    }

    /// Create a type error.
    pub fn type_error(source_code: String, span: SourceSpan, message: impl Into<String>) -> Self {
        Self::TypeError {
            message: message.into(),
            span,
            source_code,
        }
    }

    /// Create an I/O error.
    pub fn io(message: impl Into<String>, source: Option<std::io::Error>) -> Self {
        Self::Io {
            message: message.into(),
            source_error: source.map(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// Create an internal compiler error.
    #[track_caller]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            location: std::panic::Location::caller().to_string(),
        }
    }

    /// Create a runtime error.
    pub fn runtime(message: impl Into<String>, span: Option<SourceSpan>, source_code: Option<String>) -> Self {
        Self::Runtime {
            message: message.into(),
            span,
            source_code,
        }
    }
}

/// Error codes for different types of errors.
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

    // Generic categories
    Lexer,
    Parser,
    Semantic,
    TypeError,
    Io,
    Internal,
    Runtime,
}

/// Result type alias for T-Lang operations.
pub type Result<T> = std::result::Result<T, TlError>;

/// Error collector for gathering multiple errors during compilation.
#[derive(Debug, Clone, Default)]
pub struct ErrorCollector {
    errors: Vec<TlError>,
    warnings: Vec<TlError>,
}

impl ErrorCollector {
    /// Create a new error collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an error to the collection.
    pub fn add_error(&mut self, error: TlError) {
        self.errors.push(error);
    }

    /// Add a warning to the collection.
    pub fn add_warning(&mut self, warning: TlError) {
        self.warnings.push(warning);
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get all errors.
    pub fn errors(&self) -> &[TlError] {
        &self.errors
    }

    /// Get all warnings.
    pub fn warnings(&self) -> &[TlError] {
        &self.warnings
    }

    /// Get all diagnostics (errors and warnings).
    pub fn all_diagnostics(&self) -> Vec<&TlError> {
        self.errors.iter().chain(self.warnings.iter()).collect()
    }

    /// Clear all collected errors and warnings.
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