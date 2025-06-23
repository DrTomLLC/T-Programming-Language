// shared/src/token.rs

//! Token and TokenType definitions for T‑Lang, including span info
//! and utility constructors for both raw and typed tokens.

use crate::ast::Span;
use serde::{Serialize, Deserialize};

/// The kind of token parsed by the lexer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    // Keywords
    Let, Const, Fn, If, Else, While, Return,
    True, False,

    // Delimiters
    LParen, RParen, LBrace, RBrace, Semicolon, Comma,

    // Operators
    Plus, Minus, Star, Slash,
    Equals, Greater, Less,
    EqualEqual, BangEqual, GreaterEqual, LessEqual,

    // Literals
    Identifier,
    Number,
    String,

    // End-of-file
    Eof,

    // Fallback
    Unknown,
}

impl TokenType {
    /// Human‑friendly name for diagnostics.
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenType::Let           => "`let`",
            TokenType::Const         => "`const`",
            TokenType::Fn            => "`fn`",
            TokenType::If            => "`if`",
            TokenType::Else          => "`else`",
            TokenType::While         => "`while`",
            TokenType::Return        => "`return`",
            TokenType::True          => "`true`",
            TokenType::False         => "`false`",
            TokenType::LParen        => "`(`",
            TokenType::RParen        => "`)`",
            TokenType::LBrace        => "`{`",
            TokenType::RBrace        => "`}`",
            TokenType::Semicolon     => "`;`",
            TokenType::Comma         => "`,`",
            TokenType::Plus          => "`+`",
            TokenType::Minus         => "`-`",
            TokenType::Star          => "`*`",
            TokenType::Slash         => "`/`",
            TokenType::Equals        => "`=`",
            TokenType::Greater       => "`>`",
            TokenType::Less          => "`<`",
            TokenType::EqualEqual    => "`==`",
            TokenType::BangEqual     => "`!=`",
            TokenType::GreaterEqual  => "`>=`",
            TokenType::LessEqual     => "`<=`",
            TokenType::Identifier    => "identifier",
            TokenType::Number        => "number literal",
            TokenType::String        => "string literal",
            TokenType::Eof           => "end of file",
            TokenType::Unknown       => "unknown token",
        }
    }
}

/// A raw token with a byte‑offset span, as produced by the tokenizer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawToken {
    pub kind:     TokenType,
    pub lexeme:   String,
    pub span:     Span,
}

impl RawToken {
    /// Create a new raw token.
    pub fn new(kind: TokenType, lexeme: impl Into<String>, span: Span) -> Self {
        RawToken {
            kind,
            lexeme: lexeme.into(),
            span,
        }
    }
}
/// A typed token including line/column for REPL and error‑reporting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub kind:   TokenType,
    pub lexeme: String,
    pub line:   usize,
    pub col:    usize,
}

impl Token {
    /// Construct a new typed token.
    pub fn new(kind: TokenType, lexeme: &str, line: usize, col: usize) -> Self {
        Token {
            kind,
            lexeme: lexeme.to_string(),
            line,
            col,
        }
    }

    /// Pretty‑print for REPL echoes.
    pub fn display(&self) -> String {
        format!(
            "{} ({} at {}:{})",
            self.lexeme,
            self.kind.as_str(),
            self.line,
            self.col
        )
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}:{}", self.lexeme, self.line, self.col)
    }
}
