use crate::parser::ParseError as ScaffoldParseError;
use errors::TlError;

pub fn convert_parse_error(err: ScaffoldParseError, source: &str) -> TlError {
    match err {
        ScaffoldParseError::UnexpectedToken(token) => {
            TlError::parse_error(
                source.to_string(),
                errors::SourceSpan { start: errors::SourcePos(0), len: 1 },
                format!("Unexpected token: {}", token)
            )
        }
        ScaffoldParseError::UnexpectedEof => {
            TlError::parse_error(
                source.to_string(),
                errors::SourceSpan { start: errors::SourcePos(0), len: 1 },
                "Unexpected end of file".to_string()
            )
        }
        ScaffoldParseError::InvalidSyntax(msg) => {
            TlError::parse_error(
                source.to_string(),
                errors::SourceSpan { start: errors::SourcePos(0), len: 1 },
                format!("Invalid syntax: {}", msg)
            )
        }
    }
}

pub fn convert_type_error(message: &str, source: &str) -> TlError {
    TlError::type_error(
        source.to_string(),
        errors::SourceSpan { start: errors::SourcePos(0), len: 1 },
        message.to_string()
    )
}

pub type ScaffoldResult<T> = std::result::Result<T, TlError>;