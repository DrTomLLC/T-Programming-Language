// File: errors/src/lib.rs

use thiserror::Error;
use miette::{Diagnostic, NamedSource, SourceSpan};

/// A lexing error.
///
/// Highlights the offending span in the original source.
#[derive(Debug, Error, Diagnostic)]
#[error("lex error: {message}")]
#[diagnostic(code(tlang::lex))]
pub struct LexError {
    /// The full source, so we can show a code frame.
    #[source_code]
    pub src: NamedSource<String>,

    /// Where in `src` the error happened.
    #[label("{message}")]
    pub span: SourceSpan,

    /// A human‐readable error message.
    pub message: String,
}

/// A parsing error.
#[derive(Debug, Error, Diagnostic)]
#[error("parse error: {message}")]
#[diagnostic(code(tlang::parse))]
pub struct ParseError {
    #[source_code]
    pub src: NamedSource<String>,

    #[label("{message}")]
    pub span: SourceSpan,

    pub message: String,
}

/// A runtime evaluation error.
#[derive(Debug, Error, Diagnostic)]
#[error("runtime error: {message}")]
#[diagnostic(code(tlang::runtime))]
pub struct RuntimeError {
    #[source_code]
    pub src: NamedSource<String>,

    #[label("{message}")]
    pub span: SourceSpan,

    pub message: String,
}

impl LexError {
    pub fn new(file: &str, src: &str, offset: usize, length: usize, message: impl Into<String>) -> Self {
        LexError {
            src: NamedSource::new(file.to_string(), src.to_string()),
            span: SourceSpan::new(offset.into(), length),
            message: message.into(),
        }
    }
}

impl ParseError {
    pub fn new(file: &str, src: &str, offset: usize, length: usize, message: impl Into<String>) -> Self {
        ParseError {
            src: NamedSource::new(file.to_string(), src.to_string()),
            span: SourceSpan::new(offset.into(), length),
            message: message.into(),
        }
    }
}

impl RuntimeError {
    pub fn new(file: &str, src: &str, offset: usize, length: usize, message: impl Into<String>) -> Self {
        RuntimeError {
            src: NamedSource::new(file.to_string(), src.to_string()),
            span: SourceSpan::new(offset.into(), length),
            message: message.into(),
        }
    }
}

/// Central, exhaustive error definitions for T‑Lang, covering all phases.
/// This crate defines a unified `TlError` with rich diagnostics.
/// A unique error code per phase and kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // Lexing (1000–1099)
    UnexpectedChar,
    UnterminatedString,
    InvalidNumberFormat,

    // Parsing (1100–1199)
    UnexpectedToken,
    MissingToken,
    InvalidExpression,
    UnclosedGrouping,

    // Semantic (1200–1299)
    UndefinedVariable,
    TypeMismatch,
    DuplicateDefinition,
    /// Generic semantic‐analysis error
    SemanticError,

    // Codegen (1300–1399)
    CodegenFailure,

    // Runtime (1400–1499)
    RuntimeError,
    DivisionByZero,
    NullReference,

    Unsupported,
}

impl ErrorCode {
    /// Numeric code for tools.
    pub fn as_u16(self) -> u16 {
        match self {
            ErrorCode::UnexpectedChar      => 1000,
            ErrorCode::UnterminatedString  => 1001,
            ErrorCode::InvalidNumberFormat => 1002,
            ErrorCode::UnexpectedToken     => 1100,
            ErrorCode::MissingToken        => 1101,
            ErrorCode::InvalidExpression   => 1102,
            ErrorCode::UnclosedGrouping    => 1103,
            ErrorCode::UndefinedVariable   => 1200,
            ErrorCode::TypeMismatch        => 1201,
            ErrorCode::DuplicateDefinition => 1202,
            ErrorCode::SemanticError       => 1299,
            ErrorCode::CodegenFailure      => 1300,
            ErrorCode::RuntimeError        => 1400,
            ErrorCode::DivisionByZero      => 1401,
            ErrorCode::NullReference       => 1402,
            ErrorCode::Unsupported         => 1500,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::UnexpectedChar      => "E_UNEXPECTED_CHAR",
            ErrorCode::UnterminatedString  => "E_UNTERMINATED_STRING",
            ErrorCode::InvalidNumberFormat => "E_INVALID_NUMBER_FORMAT",
            ErrorCode::UnexpectedToken     => "E_UNEXPECTED_TOKEN",
            ErrorCode::MissingToken        => "E_MISSING_TOKEN",
            ErrorCode::InvalidExpression   => "E_INVALID_EXPRESSION",
            ErrorCode::UnclosedGrouping    => "E_UNCLOSED_GROUPING",
            ErrorCode::UndefinedVariable   => "E_UNDEFINED_VARIABLE",
            ErrorCode::TypeMismatch        => "E_TYPE_MISMATCH",
            ErrorCode::DuplicateDefinition => "E_DUPLICATE_DEFINITION",
            ErrorCode::SemanticError       => "E_SEMANTIC_ERROR",
            ErrorCode::CodegenFailure      => "E_CODEGEN_FAILURE",
            ErrorCode::RuntimeError        => "E_RUNTIME_ERROR",
            ErrorCode::DivisionByZero      => "E_DIVISION_BY_ZERO",
            ErrorCode::NullReference       => "E_NULL_REFERENCE",
            ErrorCode::Unsupported         => "E_UNSUPPORTED",
        }
    }
}

/// The unified error type for all compiler and runtime phases.
#[derive(Debug, Error)]
#[error("{message}")]
pub struct TlError {
    /// Where in the source the error happened.
    pub span: SourceSpan,

    /// The full source text (for IDEs / code‐lenses).
    pub src: NamedSource<String>,

    /// The high-level phase + identifier.
    pub code: ErrorCode,

    /// Human-friendly diagnostic message.
    pub message: String,

    /// Optional underlying cause.
    #[source]
    pub cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl TlError {
    /// Create a new error.
    pub fn new(
        src_name: &str,
        src_text: &str,
        span: impl Into<SourceSpan>,
        code: ErrorCode,
        message: impl Into<String>,
    ) -> Self {
        TlError {
            span: span.into(),
            src: NamedSource::new(src_name.to_string(), src_text.to_string()),
            code,
            message: message.into(),
            cause: None,
        }
    }

    /// Construct a generic parse error for an unexpected token.
    pub fn unexpected_token(
        src_name: &str,
        src_text: &str,
        span: impl Into<SourceSpan>,
        expected: impl Into<String>,
        found: impl Into<String>,
    ) -> Self {
        let err = TlError::new(
            src_name,
            src_text,
            span,
            ErrorCode::UnexpectedToken,
            format!("Expected {}, found {}", expected.into(), found.into()),
        );
        err
    }

    /// Construct a generic semantic‐analysis error.
    pub fn semantic(
        src_name: &str,
        src_text: &str,
        span: impl Into<SourceSpan>,
        message: impl Into<String>,
    ) -> Self {
        TlError::new(
            src_name,
            src_text,
            span,
            ErrorCode::SemanticError,
            message,
        )
    }

    /// Attach an underlying cause.
    pub fn with_cause(
        mut self,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }
}

pub mod lex {
    use super::*;

    pub fn unexpected_char(
        src_name: &str,
        src_text: &str,
        offset: usize,
        found: char,
    ) -> TlError {
        TlError::new(
            src_name,
            src_text,
            (offset, 1),
            ErrorCode::UnexpectedChar,
            format!("Unexpected character `{}`", found),
        )
    }

    pub fn unterminated_string(
        src_name: &str,
        src_text: &str,
        start_offset: usize,
    ) -> TlError {
        TlError::new(
            src_name,
            src_text,
            (start_offset, 1),
            ErrorCode::UnterminatedString,
            "Unterminated string literal",
        )
    }

    pub fn invalid_number(
        src_name: &str,
        src_text: &str,
        span: SourceSpan,
    ) -> TlError {
        TlError::new(
            src_name,
            src_text,
            span,
            ErrorCode::InvalidNumberFormat,
            "Invalid number format",
        )
    }
}

pub mod parse {
    use super::*;

    pub fn unexpected_token(
        src_name: &str,
        src_text: &str,
        span: SourceSpan,
        expected: &str,
        found: &str,
    ) -> TlError {
        TlError::new(
            src_name,
            src_text,
            span,
            ErrorCode::UnexpectedToken,
            format!("Expected {}, found {}", expected, found),
        )
    }

    pub fn missing_token(
        src_name: &str,
        src_text: &str,
        span: SourceSpan,
        needed: &str,
    ) -> TlError {
        TlError::new(
            src_name,
            src_text,
            span,
            ErrorCode::MissingToken,
            format!("Missing {}", needed),
        )
    }

    pub fn invalid_expression(
        src_name: &str,
        src_text: &str,
        span: SourceSpan,
    ) -> TlError {
        TlError::new(
            src_name,
            src_text,
            span,
            ErrorCode::InvalidExpression,
            "Invalid expression",
        )
    }

    pub fn unclosed_grouping(
        src_name: &str,
        src_text: &str,
        span: SourceSpan,
    ) -> TlError {
        TlError::new(
            src_name,
            src_text,
            span,
            ErrorCode::UnclosedGrouping,
            "Missing closing parenthesis or brace",
        )
    }
}

pub mod sema {
    use super::*;

    pub fn undefined_variable(
        src_name: &str,
        src_text: &str,
        span: SourceSpan,
        name: &str,
    ) -> TlError {
        TlError::new(
            src_name,
            src_text,
            span,
            ErrorCode::UndefinedVariable,
            format!("Undefined variable `{}`", name),
        )
    }

    pub fn type_mismatch(
        src_name: &str,
        src_text: &str,
        span: SourceSpan,
        expected: &str,
        found: &str,
    ) -> TlError {
        TlError::new(
            src_name,
            src_text,
            span,
            ErrorCode::TypeMismatch,
            format!("Type mismatch: expected {}, found {}", expected, found),
        )
    }

    pub fn duplicate_definition(
        src_name: &str,
        src_text: &str,
        span: SourceSpan,
        name: &str,
    ) -> TlError {
        TlError::new(
            src_name,
            src_text,
            span,
            ErrorCode::DuplicateDefinition,
            format!("`{}` is already defined", name),
        )
    }
}

pub mod codegen {
    use super::*;

    pub fn failure(msg: &str) -> TlError {
        TlError::new("<codegen>", "", (0, 0), ErrorCode::CodegenFailure, msg)
    }
}

pub mod runtime {
    use super::*;

    pub fn generic(span: SourceSpan, msg: &str) -> TlError {
        TlError::new("<runtime>", "", span, ErrorCode::RuntimeError, msg)
    }

    pub fn division_by_zero(span: SourceSpan) -> TlError {
        TlError::new("<runtime>", "", span, ErrorCode::DivisionByZero, "Division by zero")
    }

    pub fn null_reference(span: SourceSpan) -> TlError {
        TlError::new("<runtime>", "", span, ErrorCode::NullReference, "Null reference")
    }
}
