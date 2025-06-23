/// T-Lang /compiler/parser.error.rs
use crate::error::LexError;
use thiserror::Error;

/// Parse errors for T-Lang.
#[derive(Debug, Error)]
pub enum ParseError {
    /// A tokenizer error occurred.
    #[error("Lex error: {0}")]
    Lex(#[from] LexError),

    /// Ran out of tokens when expecting more input.
    #[error("Unexpected end of input")]
    UnexpectedEOF,

    /// Token didn’t match what we expected.
    #[error("Expected `{0}`")]
    ExpectedToken(String),

    /// Saw a token we can’t handle in this context.
    #[error("Unexpected token `{0}`")]
    UnexpectedToken(String),

    /// Saw something that should have been an identifier.
    #[error("Expected identifier")]
    ExpectedIdentifier,
}
