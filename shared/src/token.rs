// shared/src/token.rs

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    // Literals
    Identifier,
    Number,
    String,
    Bool(bool),

    // Keywords
    Let,
    Const,
    Fn,
    If,
    Else,
    While,
    Return,
    True,
    False,

    // Grouping
    LParen,
    RParen,
    LBrace,
    RBrace,

    // Punctuation
    Comma,
    Semicolon,

    // Operators
    Equals,
    Plus,
    Minus,
    Star,
    Slash,

    // End of input
    Eof,

    // Comparison
    Unknown,
    EqualEqual,
    Bang,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenType,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn new(kind: TokenType, lexeme: &str, line: usize) -> Self {
        Self {
            kind,
            lexeme: lexeme.to_string(),
            line,
        }
    }
}
