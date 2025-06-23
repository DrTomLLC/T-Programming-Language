// shared/src/token.rs
//! Token definitions for T-Lang.
//! Represents all possible tokens that can appear in T-Lang source code.

use miette::SourceSpan;
use serde::{Deserialize, Serialize};

/// A token in T-Lang source code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub span: SourceSpan,
}

/// All possible token types in T-Lang.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    True,
    False,

    // Identifiers and keywords
    Identifier(String),

    // Keywords
    As,
    Async,
    Await,
    Break,
    Const,
    Continue,
    Else,
    Enum,
    Fn,
    For,
    If,
    Impl,
    In,
    Let,
    Loop,
    Match,
    Mod,
    Move,
    Mut,
    Pub,
    Ref,
    Return,
    SelfValue, // self
    SelfType,  // Self
    Static,
    Struct,
    Super,
    Trait,
    Type,
    Union,
    Unsafe,
    Use,
    Where,
    While,

    // Punctuation
    LParen,      // (
    RParen,      // )
    LBrace,      // {
    RBrace,      // }
    LBracket,    // [
    RBracket,    // ]
    Comma,       // ,
    Semicolon,   // ;
    Colon,       // :
    ColonColon,  // ::
    Dot,         // .
    DotDot,      // ..
    DotDotDot,   // ...
    DotDotEq,    // ..=
    Question,    // ?
    Arrow,       // ->
    FatArrow,    // =>
    At,          // @
    Pound,       // #
    Dollar,      // $
    Tilde,       // ~

    // Operators
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    Caret,       // ^
    Bang,        // !
    And,         // &
    Or,          // |
    Shl,         // <<
    Shr,         // >>

    // Comparison
    Eq,          // =
    EqEq,        // ==
    Ne,          // !=
    Lt,          // <
    Le,          // <=
    Gt,          // >
    Ge,          // >=

    // Logical
    AndAnd,      // &&
    OrOr,        // ||

    // Assignment
    PlusEq,      // +=
    MinusEq,     // -=
    StarEq,      // *=
    SlashEq,     // /=
    PercentEq,   // %=
    CaretEq,     // ^=
    AndEq,       // &=
    OrEq,        // |=
    ShlEq,       // <<=
    ShrEq,       // >>=

    

    // Special
    Eof,
    Invalid(String), // For error recovery
}

impl Token {
    /// Create a new token.
    pub fn new(token_type: TokenType, lexeme: String, span: SourceSpan) -> Self {
        Self {
            token_type,
            lexeme,
            span,
        }
    }

    /// Check if this token is a literal.
    pub fn is_literal(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::Integer(_) | TokenType::Float(_) | TokenType::String(_) |
            TokenType::Char(_) | TokenType::True | TokenType::False
        )
    }

    /// Check if this token is a keyword.
    pub fn is_keyword(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::As | TokenType::Async | TokenType::Await | TokenType::Break |
            TokenType::Const | TokenType::Continue | TokenType::Else | TokenType::Enum |
            TokenType::Fn | TokenType::For | TokenType::If | TokenType::Impl |
            TokenType::In | TokenType::Let | TokenType::Loop | TokenType::Match |
            TokenType::Mod | TokenType::Move | TokenType::Mut | TokenType::Pub |
            TokenType::Ref | TokenType::Return | TokenType::SelfValue | TokenType::SelfType |
            TokenType::Static | TokenType::Struct | TokenType::Super | TokenType::Trait |
            TokenType::Type | TokenType::Union | TokenType::Unsafe | TokenType::Use |
            TokenType::Where | TokenType::While
        )
    }

    /// Check if this token is an operator.
    pub fn is_operator(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::Plus | TokenType::Minus | TokenType::Star | TokenType::Slash |
            TokenType::Percent | TokenType::Caret | TokenType::Bang | TokenType::And |
            TokenType::Or | TokenType::Shl | TokenType::Shr | TokenType::Eq |
            TokenType::EqEq | TokenType::Ne | TokenType::Lt | TokenType::Le |
            TokenType::Gt | TokenType::Ge | TokenType::AndAnd | TokenType::OrOr |
            TokenType::PlusEq | TokenType::MinusEq | TokenType::StarEq | TokenType::SlashEq |
            TokenType::PercentEq | TokenType::CaretEq | TokenType::AndEq | TokenType::OrEq |
            TokenType::ShlEq | TokenType::ShrEq
        )
    }

    /// Check if this token is punctuation.
    pub fn is_punctuation(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::LParen | TokenType::RParen | TokenType::LBrace | TokenType::RBrace |
            TokenType::LBracket | TokenType::RBracket | TokenType::Comma | TokenType::Semicolon |
            TokenType::Colon | TokenType::ColonColon | TokenType::Dot | TokenType::DotDot |
            TokenType::DotDotDot | TokenType::DotDotEq | TokenType::Question | TokenType::Arrow |
            TokenType::FatArrow | TokenType::At | TokenType::Pound | TokenType::Dollar |
            TokenType::Tilde
        )
    }

    /// Get the precedence of this token if it's a binary operator.
    pub fn precedence(&self) -> Option<u8> {
        match self.token_type {
            TokenType::OrOr => Some(1),
            TokenType::AndAnd => Some(2),
            TokenType::EqEq | TokenType::Ne => Some(3),
            TokenType::Lt | TokenType::Le | TokenType::Gt | TokenType::Ge => Some(4),
            TokenType::Or => Some(5),
            TokenType::Caret => Some(6),
            TokenType::And => Some(7),
            TokenType::Shl | TokenType::Shr => Some(8),
            TokenType::Plus | TokenType::Minus => Some(9),
            TokenType::Star | TokenType::Slash | TokenType::Percent => Some(10),
            _ => None,
        }
    }

    /// Check if this operator is right-associative.
    pub fn is_right_associative(&self) -> bool {
        matches!(self.token_type, TokenType::Eq)
    }

    /// Get a human-readable description of this token type.
    pub fn type_description(&self) -> &'static str {
        match self.token_type {
            TokenType::Integer(_) => "integer literal",
            TokenType::Float(_) => "float literal",
            TokenType::String(_) => "string literal",
            TokenType::Char(_) => "character literal",
            TokenType::True | TokenType::False => "boolean literal",
            TokenType::Identifier(_) => "identifier",
            TokenType::LParen => "'('",
            TokenType::RParen => "')'",
            TokenType::LBrace => "'{'",
            TokenType::RBrace => "'}'",
            TokenType::LBracket => "'['",
            TokenType::RBracket => "']'",
            TokenType::Semicolon => "';'",
            TokenType::Comma => "','",
            TokenType::Dot => "'.'",
            TokenType::Arrow => "'->'",
            TokenType::FatArrow => "'=>'",
            TokenType::Eq => "'='",
            TokenType::EqEq => "'=='",
            TokenType::Ne => "'!='",
            TokenType::Lt => "'<'",
            TokenType::Le => "'<='",
            TokenType::Gt => "'>'",
            TokenType::Ge => "'>='",
            TokenType::Plus => "'+'",
            TokenType::Minus => "'-'",
            TokenType::Star => "'*'",
            TokenType::Slash => "'/'",
            TokenType::Percent => "'%'",
            TokenType::Bang => "'!'",
            TokenType::AndAnd => "'&&'",
            TokenType::OrOr => "'||'",
            TokenType::Eof => "end of file",
            TokenType::Invalid(_) => "invalid token",
            _ => {
                if self.is_keyword() {
                    "keyword"
                } else {
                    "token"
                }
            }
        }
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Integer(n) => write!(f, "{}", n),
            TokenType::Float(n) => write!(f, "{}", n),
            TokenType::String(s) => write!(f, "\"{}\"", s),
            TokenType::Char(c) => write!(f, "'{}'", c),
            TokenType::Identifier(name) => write!(f, "{}", name),
            TokenType::Invalid(s) => write!(f, "Invalid({})", s),
            _ => {
                let token = Token::new(self.clone(), String::new(), SourceSpan::new(0.into(), 0));
                write!(f, "{}", token.type_description())
            }
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.lexeme.is_empty() {
            write!(f, "{}", self.token_type)
        } else {
            write!(f, "{}", self.lexeme)
        }
    }
}