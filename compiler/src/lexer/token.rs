// compiler/src/lexer/token.rs
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // single‑char
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,
    // one or two char
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,
    // literals
    Identifier(String),
    String(String),
    Number(f64),
    // keywords
    If, Else, While, For, Fun, Return, True, False, Let,
    // EOF
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

// In your existing lexer you’d import Token and emit them instead of raw strings.
