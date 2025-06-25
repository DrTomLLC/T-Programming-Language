// File: shared/src/token.rs - ENHANCED
// -----------------------------------------------------------------------------

use miette::SourceSpan;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub kind: TokenType,
    pub lexeme: String,
    #[serde(skip)]
    pub span: SourceSpan,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    // Literals with values
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Identifier(String),

    // Keywords
    Let, Const, Fn, If, Else, While, For, Loop, Match,
    Return, Break, Continue, True, False, Struct, Enum,
    Trait, Impl, Use, Pub, Mod, Static, Extern, Async,
    Await, Move, Mut, Ref, SelfValue, SelfType, Super,
    Type, Union, Unsafe, Where, As, In,

    // Operators
    Plus, Minus, Star, Slash, Percent, // + - * / %
    Equal, EqualEqual, Bang, BangEqual, // = == ! !=
    Less, LessEqual, Greater, GreaterEqual, // < <= > >=
    AmpAmp, PipePipe, // && ||
    Amp, Pipe, Caret, Tilde, // & | ^ ~
    LessLess, GreaterGreater, // << >>

    // Assignment operators
    PlusEq, MinusEq, StarEq, SlashEq, // += -= *= /=

    // Punctuation
    LParen, RParen, // ( )
    LBrace, RBrace, // { }
    LBracket, RBracket, // [ ]
    Comma, Semicolon, Colon, ColonColon, // , ; : ::
    Dot, DotDot, DotDotDot, DotDotEq, // . .. ... ..=
    Arrow, FatArrow, // -> =>
    Question, At, Pound, Dollar, // ? @ # $

    // Special
    Eof,
    Newline,
    Comment(String),

    // Error recovery
    Unknown,
}

impl Token {
    pub fn new(kind: TokenType, lexeme: String, span: SourceSpan) -> Self {
        // Calculate line/col from span
        let line = 1; // TODO: Calculate from source if needed
        let col = span.offset();

        Self { kind, lexeme, span, line, col }
    }

    pub fn eof(span: SourceSpan) -> Self {
        Self::new(TokenType::Eof, String::new(), span)
    }

    pub fn is_keyword(&self) -> bool {
        matches!(self.kind,
            TokenType::Let | TokenType::Const | TokenType::Fn |
            TokenType::If | TokenType::Else | TokenType::While |
            TokenType::For | TokenType::Return | TokenType::True |
            TokenType::False | TokenType::Struct | TokenType::Enum
        )
    }

    pub fn is_literal(&self) -> bool {
        matches!(self.kind,
            TokenType::Integer(_) | TokenType::Float(_) |
            TokenType::String(_) | TokenType::Char(_) |
            TokenType::True | TokenType::False
        )
    }

    pub fn is_operator(&self) -> bool {
        matches!(self.kind,
            TokenType::Plus | TokenType::Minus | TokenType::Star |
            TokenType::Slash | TokenType::Percent | TokenType::Equal |
            TokenType::EqualEqual | TokenType::Bang | TokenType::BangEqual |
            TokenType::Less | TokenType::LessEqual | TokenType::Greater |
            TokenType::GreaterEqual | TokenType::AmpAmp | TokenType::PipePipe
        )
    }
}
/// A stream of tokens for macro processing
#[derive(Debug, Clone, PartialEq)]
pub struct TokenStream {
    pub tokens: Vec<Token>,
}

impl TokenStream {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }
}