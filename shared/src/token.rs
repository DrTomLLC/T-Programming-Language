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
    Hash,        // #
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
    LeftShift,   // <<
    RightShift,  // >>

    // Comparison
    Equal,       // =
    EqualEqual,  // ==
    BangEqual,   // !=
    Less,        // <
    LessEqual,   // <=
    Greater,     // >
    GreaterEqual,// >=

    // Logical
    AndAnd,      // &&
    OrOr,        // ||

    // Assignment operators
    PlusEq,      // +=
    MinusEq,     // -=
    StarEq,      // *=
    SlashEq,     // /=
    PercentEq,   // %=
    CaretEq,     // ^=
    AndEq,       // &=
    OrEq,        // |=
    LeftShiftEq, // <<=
    RightShiftEq,// >>=

    // Increment/decrement
    PlusPlus,    // ++
    MinusMinus,  // --

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
            TokenType::True | TokenType::False | TokenType::Type | TokenType::Union |
            TokenType::Unsafe | TokenType::Use | TokenType::Where | TokenType::While
        )
    }

    /// Check if this token is an operator.
    pub fn is_operator(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::Plus | TokenType::Minus | TokenType::Star | TokenType::Slash |
            TokenType::Percent | TokenType::Caret | TokenType::Bang | TokenType::And |
            TokenType::Or | TokenType::LeftShift | TokenType::RightShift | TokenType::Equal |
            TokenType::EqualEqual | TokenType::BangEqual | TokenType::Less | TokenType::LessEqual |
            TokenType::Greater | TokenType::GreaterEqual | TokenType::AndAnd | TokenType::OrOr |
            TokenType::PlusEq | TokenType::MinusEq | TokenType::StarEq | TokenType::SlashEq |
            TokenType::PercentEq | TokenType::CaretEq | TokenType::AndEq | TokenType::OrEq |
            TokenType::LeftShiftEq | TokenType::RightShiftEq | TokenType::PlusPlus | TokenType::MinusMinus
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
            TokenType::FatArrow | TokenType::At | TokenType::Hash | TokenType::Dollar |
            TokenType::Tilde
        )
    }

    /// Get the precedence of this token if it's a binary operator.
    /// Returns None for non-binary operators.
    pub fn precedence(&self) -> Option<u8> {
        match self.token_type {
            // Assignment (right-associative, lowest precedence)
            TokenType::Equal | TokenType::PlusEq | TokenType::MinusEq | TokenType::StarEq |
            TokenType::SlashEq | TokenType::PercentEq | TokenType::CaretEq | TokenType::AndEq |
            TokenType::OrEq | TokenType::LeftShiftEq | TokenType::RightShiftEq => Some(1),

            // Logical OR
            TokenType::OrOr => Some(2),

            // Logical AND
            TokenType::AndAnd => Some(3),

            // Bitwise OR
            TokenType::Or => Some(4),

            // Bitwise XOR
            TokenType::Caret => Some(5),

            // Bitwise AND
            TokenType::And => Some(6),

            // Equality
            TokenType::EqualEqual | TokenType::BangEqual => Some(7),

            // Relational
            TokenType::Less | TokenType::LessEqual | TokenType::Greater | TokenType::GreaterEqual => Some(8),

            // Shift
            TokenType::LeftShift | TokenType::RightShift => Some(9),

            // Additive
            TokenType::Plus | TokenType::Minus => Some(10),

            // Multiplicative
            TokenType::Star | TokenType::Slash | TokenType::Percent => Some(11),

            // Not a binary operator
            _ => None,
        }
    }

    /// Check if this operator is right-associative.
    pub fn is_right_associative(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::Equal | TokenType::PlusEq | TokenType::MinusEq | TokenType::StarEq |
            TokenType::SlashEq | TokenType::PercentEq | TokenType::CaretEq | TokenType::AndEq |
            TokenType::OrEq | TokenType::LeftShiftEq | TokenType::RightShiftEq
        )
    }

    /// Check if this is a unary operator.
    pub fn is_unary_operator(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::Bang | TokenType::Minus | TokenType::Plus | TokenType::Star |
            TokenType::And | TokenType::PlusPlus | TokenType::MinusMinus
        )
    }

    /// Check if this token can start an expression.
    pub fn can_start_expression(&self) -> bool {
        self.is_literal() ||
            matches!(self.token_type,
            TokenType::Identifier(_) | TokenType::LParen |
            TokenType::Bang | TokenType::Minus | TokenType::Plus | TokenType::Star | TokenType::And
        )
    }

    /// Check if this token can start a statement.
    pub fn can_start_statement(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::Let | TokenType::Const | TokenType::If | TokenType::While |
            TokenType::For | TokenType::Loop | TokenType::Match | TokenType::Return |
            TokenType::Break | TokenType::Continue | TokenType::LBrace
        ) || self.can_start_expression()
    }

    /// Check if this token can start an item (top-level declaration).
    pub fn can_start_item(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::Fn | TokenType::Struct | TokenType::Enum | TokenType::Impl |
            TokenType::Trait | TokenType::Type | TokenType::Const | TokenType::Static |
            TokenType::Use | TokenType::Mod | TokenType::Pub | TokenType::Unsafe
        )
    }

    /// Get a human-readable description of this token type.
    pub fn type_description(&self) -> String {
        match &self.token_type {
            TokenType::Integer(_) => "integer".to_string(),
            TokenType::Float(_) => "float".to_string(),
            TokenType::String(_) => "string".to_string(),
            TokenType::Char(_) => "character".to_string(),
            TokenType::Identifier(_) => "identifier".to_string(),
            TokenType::Invalid(_) => "invalid token".to_string(),
            TokenType::Eof => "end of file".to_string(),

            // Keywords
            TokenType::As => "'as'".to_string(),
            TokenType::Async => "'async'".to_string(),
            TokenType::Await => "'await'".to_string(),
            TokenType::Break => "'break'".to_string(),
            TokenType::Const => "'const'".to_string(),
            TokenType::Continue => "'continue'".to_string(),
            TokenType::Else => "'else'".to_string(),
            TokenType::Enum => "'enum'".to_string(),
            TokenType::False => "'false'".to_string(),
            TokenType::Fn => "'fn'".to_string(),
            TokenType::For => "'for'".to_string(),
            TokenType::If => "'if'".to_string(),
            TokenType::Impl => "'impl'".to_string(),
            TokenType::In => "'in'".to_string(),
            TokenType::Let => "'let'".to_string(),
            TokenType::Loop => "'loop'".to_string(),
            TokenType::Match => "'match'".to_string(),
            TokenType::Mod => "'mod'".to_string(),
            TokenType::Move => "'move'".to_string(),
            TokenType::Mut => "'mut'".to_string(),
            TokenType::Pub => "'pub'".to_string(),
            TokenType::Ref => "'ref'".to_string(),
            TokenType::Return => "'return'".to_string(),
            TokenType::SelfValue => "'self'".to_string(),
            TokenType::SelfType => "'Self'".to_string(),
            TokenType::Static => "'static'".to_string(),
            TokenType::Struct => "'struct'".to_string(),
            TokenType::Super => "'super'".to_string(),
            TokenType::Trait => "'trait'".to_string(),
            TokenType::True => "'true'".to_string(),
            TokenType::Type => "'type'".to_string(),
            TokenType::Union => "'union'".to_string(),
            TokenType::Unsafe => "'unsafe'".to_string(),
            TokenType::Use => "'use'".to_string(),
            TokenType::Where => "'where'".to_string(),
            TokenType::While => "'while'".to_string(),

            // Punctuation
            TokenType::LParen => "'('".to_string(),
            TokenType::RParen => "')'".to_string(),
            TokenType::LBrace => "'{'".to_string(),
            TokenType::RBrace => "'}'".to_string(),
            TokenType::LBracket => "'['".to_string(),
            TokenType::RBracket => "']'".to_string(),
            TokenType::Comma => "','".to_string(),
            TokenType::Semicolon => "';'".to_string(),
            TokenType::Colon => "':'".to_string(),
            TokenType::ColonColon => "'::'".to_string(),
            TokenType::Dot => "'.'".to_string(),
            TokenType::DotDot => "'..'".to_string(),
            TokenType::DotDotDot => "'...'".to_string(),
            TokenType::DotDotEq => "'..='".to_string(),
            TokenType::Question => "'?'".to_string(),
            TokenType::Arrow => "'->'".to_string(),
            TokenType::FatArrow => "'=>'".to_string(),
            TokenType::At => "'@'".to_string(),
            TokenType::Hash => "'#'".to_string(),
            TokenType::Dollar => "'$'".to_string(),
            TokenType::Tilde => "'~'".to_string(),

            // Operators
            TokenType::Plus => "'+'".to_string(),
            TokenType::Minus => "'-'".to_string(),
            TokenType::Star => "'*'".to_string(),
            TokenType::Slash => "'/'".to_string(),
            TokenType::Percent => "'%'".to_string(),
            TokenType::Caret => "'^'".to_string(),
            TokenType::Bang => "'!'".to_string(),
            TokenType::And => "'&'".to_string(),
            TokenType::Or => "'|'".to_string(),
            TokenType::LeftShift => "'<<'".to_string(),
            TokenType::RightShift => "'>>'".to_string(),
            TokenType::Equal => "'='".to_string(),
            TokenType::EqualEqual => "'=='".to_string(),
            TokenType::BangEqual => "'!='".to_string(),
            TokenType::Less => "'<'".to_string(),
            TokenType::LessEqual => "'<='".to_string(),
            TokenType::Greater => "'>'".to_string(),
            TokenType::GreaterEqual => "'>='".to_string(),
            TokenType::AndAnd => "'&&'".to_string(),
            TokenType::OrOr => "'||'".to_string(),
            TokenType::PlusEq => "'+='".to_string(),
            TokenType::MinusEq => "'-='".to_string(),
            TokenType::StarEq => "'*='".to_string(),
            TokenType::SlashEq => "'/='".to_string(),
            TokenType::PercentEq => "'%='".to_string(),
            TokenType::CaretEq => "'^='".to_string(),
            TokenType::AndEq => "'&='".to_string(),
            TokenType::OrEq => "'|='".to_string(),
            TokenType::LeftShiftEq => "'<<='".to_string(),
            TokenType::RightShiftEq => "'>>='".to_string(),
            TokenType::PlusPlus => "'++'".to_string(),
            TokenType::MinusMinus => "'--'".to_string(),
        }
    }

    /// Extract identifier name if this is an identifier token.
    pub fn identifier_name(&self) -> Option<&str> {
        if let TokenType::Identifier(name) = &self.token_type {
            Some(name)
        } else {
            None
        }
    }

    /// Extract string value if this is a string token.
    pub fn string_value(&self) -> Option<&str> {
        if let TokenType::String(value) = &self.token_type {
            Some(value)
        } else {
            None
        }
    }

    /// Extract integer value if this is an integer token.
    pub fn integer_value(&self) -> Option<i64> {
        if let TokenType::Integer(value) = self.token_type {
            Some(value)
        } else {
            None
        }
    }

    /// Extract float value if this is a float token.
    pub fn float_value(&self) -> Option<f64> {
        if let TokenType::Float(value) = self.token_type {
            Some(value)
        } else {
            None
        }
    }

    /// Extract character value if this is a character token.
    pub fn char_value(&self) -> Option<char> {
        if let TokenType::Char(value) = self.token_type {
            Some(value)
        } else {
            None
        }
    }

    /// Check if this is a specific keyword.
    pub fn is_keyword_of(&self, keyword: &TokenType) -> bool {
        self.is_keyword() && &self.token_type == keyword
    }

    /// Get the byte offset within the source.
    pub fn offset(&self) -> usize {
        self.span.offset()
    }

    /// Get the length of this token in bytes.
    pub fn len(&self) -> usize {
        self.span.len()
    }

    /// Get the ending offset of this token.
    pub fn end_offset(&self) -> usize {
        self.offset() + self.len()
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

/// Helper trait for working with collections of tokens.
pub trait TokenStream {
    /// Find the first token of a specific type.
    fn find_token(&self, token_type: &TokenType) -> Option<usize>;

    /// Find all tokens of a specific type.
    fn find_all_tokens(&self, token_type: &TokenType) -> Vec<usize>;

    /// Check if the stream contains a specific token type.
    fn contains_token(&self, token_type: &TokenType) -> bool;

    /// Get all identifiers in the stream.
    fn identifiers(&self) -> Vec<&str>;

    /// Get all string literals in the stream.
    fn string_literals(&self) -> Vec<&str>;
}

impl TokenStream for Vec<Token> {
    fn find_token(&self, token_type: &TokenType) -> Option<usize> {
        self.iter().position(|token| &token.token_type == token_type)
    }

    fn find_all_tokens(&self, token_type: &TokenType) -> Vec<usize> {
        self.iter()
            .enumerate()
            .filter_map(|(i, token)| {
                if &token.token_type == token_type {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    fn contains_token(&self, token_type: &TokenType) -> bool {
        self.iter().any(|token| &token.token_type == token_type)
    }

    fn identifiers(&self) -> Vec<&str> {
        self.iter()
            .filter_map(|token| token.identifier_name())
            .collect()
    }

    fn string_literals(&self) -> Vec<&str> {
        self.iter()
            .filter_map(|token| token.string_value())
            .collect()
    }
}

impl TokenStream for &[Token] {
    fn find_token(&self, token_type: &TokenType) -> Option<usize> {
        self.iter().position(|token| &token.token_type == token_type)
    }

    fn find_all_tokens(&self, token_type: &TokenType) -> Vec<usize> {
        self.iter()
            .enumerate()
            .filter_map(|(i, token)| {
                if &token.token_type == token_type {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    fn contains_token(&self, token_type: &TokenType) -> bool {
        self.iter().any(|token| &token.token_type == token_type)
    }

    fn identifiers(&self) -> Vec<&str> {
        self.iter()
            .filter_map(|token| token.identifier_name())
            .collect()
    }

    fn string_literals(&self) -> Vec<&str> {
        self.iter()
            .filter_map(|token| token.string_value())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_token(token_type: TokenType) -> Token {
        Token::new(token_type, String::new(), SourceSpan::new(0.into(), 0))
    }

    #[test]
    fn test_token_creation() {
        let token = Token::new(
            TokenType::Identifier("test".to_string()),
            "test".to_string(),
            SourceSpan::new(0.into(), 4),
        );

        assert_eq!(token.identifier_name(), Some("test"));
        assert_eq!(token.len(), 4);
        assert_eq!(token.offset(), 0);
    }

    #[test]
    fn test_literal_detection() {
        assert!(make_token(TokenType::Integer(42)).is_literal());
        assert!(make_token(TokenType::Float(3.14)).is_literal());
        assert!(make_token(TokenType::String("hello".to_string())).is_literal());
        assert!(make_token(TokenType::Char('a')).is_literal());
        assert!(make_token(TokenType::True).is_literal());
        assert!(make_token(TokenType::False).is_literal());

        assert!(!make_token(TokenType::Identifier("test".to_string())).is_literal());
        assert!(!make_token(TokenType::Plus).is_literal());
    }

    #[test]
    fn test_keyword_detection() {
        assert!(make_token(TokenType::Fn).is_keyword());
        assert!(make_token(TokenType::Let).is_keyword());
        assert!(make_token(TokenType::If).is_keyword());
        assert!(make_token(TokenType::While).is_keyword());

        assert!(!make_token(TokenType::Identifier("test".to_string())).is_keyword());
        assert!(!make_token(TokenType::Plus).is_keyword());
    }

    #[test]
    fn test_operator_detection() {
        assert!(make_token(TokenType::Plus).is_operator());
        assert!(make_token(TokenType::EqualEqual).is_operator());
        assert!(make_token(TokenType::PlusEq).is_operator());

        assert!(!make_token(TokenType::Identifier("test".to_string())).is_operator());
        assert!(!make_token(TokenType::LParen).is_operator());
    }

    #[test]
    fn test_precedence() {
        assert_eq!(make_token(TokenType::Star).precedence(), Some(11));
        assert_eq!(make_token(TokenType::Plus).precedence(), Some(10));
        assert_eq!(make_token(TokenType::EqualEqual).precedence(), Some(7));
        assert_eq!(make_token(TokenType::Equal).precedence(), Some(1));

        assert_eq!(make_token(TokenType::LParen).precedence(), None);
        assert_eq!(make_token(TokenType::Identifier("test".to_string())).precedence(), None);
    }

    #[test]
    fn test_associativity() {
        assert!(make_token(TokenType::Equal).is_right_associative());
        assert!(make_token(TokenType::PlusEq).is_right_associative());

        assert!(!make_token(TokenType::Plus).is_right_associative());
        assert!(!make_token(TokenType::Star).is_right_associative());
    }

    #[test]
    fn test_expression_starters() {
        assert!(make_token(TokenType::Integer(42)).can_start_expression());
        assert!(make_token(TokenType::Identifier("test".to_string())).can_start_expression());
        assert!(make_token(TokenType::LParen).can_start_expression());
        assert!(make_token(TokenType::Bang).can_start_expression());

        assert!(!make_token(TokenType::RBrace).can_start_expression());
        assert!(!make_token(TokenType::Semicolon).can_start_expression());
    }

    #[test]
    fn test_statement_starters() {
        assert!(make_token(TokenType::Let).can_start_statement());
        assert!(make_token(TokenType::If).can_start_statement());
        assert!(make_token(TokenType::While).can_start_statement());
        assert!(make_token(TokenType::Return).can_start_statement());

        // Expressions can also start statements
        assert!(make_token(TokenType::Identifier("test".to_string())).can_start_statement());
    }

    #[test]
    fn test_item_starters() {
        assert!(make_token(TokenType::Fn).can_start_item());
        assert!(make_token(TokenType::Struct).can_start_item());
        assert!(make_token(TokenType::Enum).can_start_item());
        assert!(make_token(TokenType::Use).can_start_item());

        assert!(!make_token(TokenType::Let).can_start_item());
        assert!(!make_token(TokenType::Identifier("test".to_string())).can_start_item());
    }

    #[test]
    fn test_token_stream_helpers() {
        let tokens = vec![
            Token::new(TokenType::Fn, "fn".to_string(), SourceSpan::new(0.into(), 2)),
            Token::new(TokenType::Identifier("test".to_string()), "test".to_string(), SourceSpan::new(3.into(), 4)),
            Token::new(TokenType::LParen, "(".to_string(), SourceSpan::new(7.into(), 1)),
            Token::new(TokenType::RParen, ")".to_string(), SourceSpan::new(8.into(), 1)),
        ];

        assert_eq!(tokens.find_token(&TokenType::Fn), Some(0));
        assert_eq!(tokens.find_token(&TokenType::LParen), Some(2));
        assert_eq!(tokens.find_token(&TokenType::Semicolon), None);

        assert!(tokens.contains_token(&TokenType::Fn));
        assert!(!tokens.contains_token(&TokenType::Let));

        let identifiers = tokens.identifiers();
        assert_eq!(identifiers, vec!["test"]);
    }

    #[test]
    fn test_token_display() {
        let token = Token::new(
            TokenType::Identifier("hello".to_string()),
            "hello".to_string(),
            SourceSpan::new(0.into(), 5),
        );
        assert_eq!(format!("{}", token), "hello");

        let int_token = Token::new(
            TokenType::Integer(42),
            "42".to_string(),
            SourceSpan::new(0.into(), 2),
        );
        assert_eq!(format!("{}", int_token), "42");
    }
}