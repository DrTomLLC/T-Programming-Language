// shared/src/tokenizer.rs
// shared/src/tokenizer.rs
//! Tokenizer for T-Lang source code.
//! Converts source text into a stream of tokens for parsing.
//!
//! Design principles:
//! - No panics or unwraps
//! - Comprehensive error reporting with source spans
//! - Support for all T-Lang token types
//! - Unicode-aware string handling

use crate::token::{Token, TokenType};
use errors::{Result, TlError};
use miette::SourceSpan;

/// Main tokenizer struct that processes source code.
pub struct Tokenizer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    source: String,
}

impl Tokenizer {
    /// Create a new tokenizer for the given source code.
    pub fn new(source: String) -> Self {
        let input: Vec<char> = source.chars().collect();
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
            source,
        }
    }

    /// Tokenize the entire input into a vector of tokens.
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            match self.next_token() {
                Ok(Some(token)) => tokens.push(token),
                Ok(None) => continue, // Skip whitespace/comments
                Err(e) => return Err(e),
            }
        }

        // Add EOF token
        tokens.push(Token::new(
            TokenType::Eof,
            String::new(),
            self.current_span(0),
        ));

        Ok(tokens)
    }

    /// Get the next token from the input.
    fn next_token(&mut self) -> Result<Option<Token>> {
        self.skip_whitespace();

        if self.is_at_end() {
            return Ok(None);
        }

        let start_pos = self.position;
        let ch = self.advance();

        let token_type = match ch {
            // Single-character tokens
            '(' => TokenType::LParen,
            ')' => TokenType::RParen,
            '{' => TokenType::LBrace,
            '}' => TokenType::RBrace,
            '[' => TokenType::LBracket,
            ']' => TokenType::RBracket,
            ',' => TokenType::Comma,
            ';' => TokenType::Semicolon,
            '+' => self.match_char('=').then_some(TokenType::PlusEq).unwrap_or(TokenType::Plus),
            '-' => {
                if self.match_char('=') {
                    TokenType::MinusEq
                } else if self.match_char('>') {
                    TokenType::Arrow
                } else {
                    TokenType::Minus
                }
            },
            '*' => self.match_char('=').then_some(TokenType::StarEq).unwrap_or(TokenType::Star),
            '/' => {
                if self.match_char('/') {
                    self.skip_line_comment();
                    return Ok(None);
                } else if self.match_char('*') {
                    self.skip_block_comment()?;
                    return Ok(None);
                } else if self.match_char('=') {
                    TokenType::SlashEq
                } else {
                    TokenType::Slash
                }
            },
            '%' => self.match_char('=').then_some(TokenType::PercentEq).unwrap_or(TokenType::Percent),
            '^' => self.match_char('=').then_some(TokenType::CaretEq).unwrap_or(TokenType::Caret),
            '!' => self.match_char('=').then_some(TokenType::Ne).unwrap_or(TokenType::Bang),
            '=' => {
                if self.match_char('=') {
                    TokenType::EqEq
                } else if self.match_char('>') {
                    TokenType::FatArrow
                } else {
                    TokenType::Eq
                }
            },
            '<' => {
                if self.match_char('=') {
                    TokenType::Le
                } else if self.match_char('<') {
                    self.match_char('=').then_some(TokenType::ShlEq).unwrap_or(TokenType::Shl)
                } else {
                    TokenType::Lt
                }
            },
            '>' => {
                if self.match_char('=') {
                    TokenType::Ge
                } else if self.match_char('>') {
                    self.match_char('=').then_some(TokenType::ShrEq).unwrap_or(TokenType::Shr)
                } else {
                    TokenType::Gt
                }
            },
            '&' => {
                if self.match_char('&') {
                    TokenType::AndAnd
                } else if self.match_char('=') {
                    TokenType::AndEq
                } else {
                    TokenType::And
                }
            },
            '|' => {
                if self.match_char('|') {
                    TokenType::OrOr
                } else if self.match_char('=') {
                    TokenType::OrEq
                } else {
                    TokenType::Or
                }
            },
            '.' => {
                if self.match_char('.') {
                    if self.match_char('.') {
                        TokenType::DotDotDot
                    } else if self.match_char('=') {
                        TokenType::DotDotEq
                    } else {
                        TokenType::DotDot
                    }
                } else {
                    TokenType::Dot
                }
            },
            ':' => {
                if self.match_char(':') {
                    TokenType::ColonColon
                } else {
                    TokenType::Colon
                }
            },
            '?' => TokenType::Question,
            '@' => TokenType::At,
            '#' => TokenType::Pound,
            '$' => TokenType::Dollar,
            '~' => TokenType::Tilde,

            // String literals
            '"' => return self.string_literal(),
            '\'' => return self.char_literal(),

            // Numbers
            '0'..='9' => return self.number_literal(start_pos),

            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => return self.identifier_or_keyword(start_pos),

            // Invalid character
            _ => {
                return Err(TlError::lexer(
                    self.source.clone(),
                    self.current_span(1),
                    format!("Unexpected character: '{}'", ch),
                ));
            }
        };

        let lexeme = self.get_lexeme(start_pos);
        let span = self.span_from(start_pos);

        Ok(Some(Token::new(token_type, lexeme, span)))
    }

    /// Parse a string literal.
    fn string_literal(&mut self) -> Result<Option<Token>> {
        let start_pos = self.position - 1; // Include opening quote
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\\' {
                self.advance(); // consume backslash
                if self.is_at_end() {
                    return Err(TlError::lexer(
                        self.source.clone(),
                        self.current_span(1),
                        "Unterminated string literal",
                    ));
                }

                let escaped = match self.advance() {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    '\'' => '\'',
                    '0' => '\0',
                    c => {
                        return Err(TlError::lexer(
                            self.source.clone(),
                            self.current_span(1),
                            format!("Invalid escape sequence: \\{}", c),
                        ));
                    }
                };
                value.push(escaped);
            } else {
                value.push(self.advance());
            }
        }

        if self.is_at_end() {
            return Err(TlError::lexer(
                self.source.clone(),
                self.current_span(1),
                "Unterminated string literal",
            ));
        }

        self.advance(); // closing quote

        let span = self.span_from(start_pos);
        Ok(Some(Token::new(TokenType::String(value), self.get_lexeme(start_pos), span)))
    }

    /// Parse a character literal.
    fn char_literal(&mut self) -> Result<Option<Token>> {
        let start_pos = self.position - 1;

        if self.is_at_end() {
            return Err(TlError::lexer(
                self.source.clone(),
                self.current_span(1),
                "Unterminated character literal",
            ));
        }

        let ch = if self.peek() == '\\' {
            self.advance(); // consume backslash
            if self.is_at_end() {
                return Err(TlError::lexer(
                    self.source.clone(),
                    self.current_span(1),
                    "Unterminated character literal",
                ));
            }

            match self.advance() {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '\\' => '\\',
                '"' => '"',
                '\'' => '\'',
                '0' => '\0',
                c => {
                    return Err(TlError::lexer(
                        self.source.clone(),
                        self.current_span(1),
                        format!("Invalid escape sequence: \\{}", c),
                    ));
                }
            }
        } else {
            self.advance()
        };

        if self.is_at_end() || self.peek() != '\'' {
            return Err(TlError::lexer(
                self.source.clone(),
                self.current_span(1),
                "Unterminated character literal",
            ));
        }

        self.advance(); // closing quote

        let span = self.span_from(start_pos);
        Ok(Some(Token::new(TokenType::Char(ch), self.get_lexeme(start_pos), span)))
    }

    /// Parse a number literal.
    fn number_literal(&mut self, start_pos: usize) -> Result<Option<Token>> {
        // Consume digits
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Look for decimal point
        let mut is_float = false;
        if self.peek() == '.' && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
            is_float = true;
            self.advance(); // consume '.'
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        // Look for exponent
        if matches!(self.peek(), 'e' | 'E') {
            is_float = true;
            self.advance();
            if matches!(self.peek(), '+' | '-') {
                self.advance();
            }
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let lexeme = self.get_lexeme(start_pos);
        let span = self.span_from(start_pos);

        let token_type = if is_float {
            match lexeme.parse::<f64>() {
                Ok(value) => TokenType::Float(value),
                Err(_) => {
                    return Err(TlError::lexer(
                        self.source.clone(),
                        span,
                        format!("Invalid float literal: {}", lexeme),
                    ));
                }
            }
        } else {
            match lexeme.parse::<i64>() {
                Ok(value) => TokenType::Integer(value),
                Err(_) => {
                    return Err(TlError::lexer(
                        self.source.clone(),
                        span,
                        format!("Invalid integer literal: {}", lexeme),
                    ));
                }
            }
        };

        Ok(Some(Token::new(token_type, lexeme, span)))
    }

    /// Parse an identifier or keyword.
    fn identifier_or_keyword(&mut self, start_pos: usize) -> Result<Option<Token>> {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let lexeme = self.get_lexeme(start_pos);
        let span = self.span_from(start_pos);

        let token_type = match lexeme.as_str() {
            // Keywords
            "as" => TokenType::As,
            "async" => TokenType::Async,
            "await" => TokenType::Await,
            "break" => TokenType::Break,
            "const" => TokenType::Const,
            "continue" => TokenType::Continue,
            "else" => TokenType::Else,
            "enum" => TokenType::Enum,
            "false" => TokenType::False,
            "fn" => TokenType::Fn,
            "for" => TokenType::For,
            "if" => TokenType::If,
            "impl" => TokenType::Impl,
            "in" => TokenType::In,
            "let" => TokenType::Let,
            "loop" => TokenType::Loop,
            "match" => TokenType::Match,
            "mod" => TokenType::Mod,
            "move" => TokenType::Move,
            "mut" => TokenType::Mut,
            "pub" => TokenType::Pub,
            "ref" => TokenType::Ref,
            "return" => TokenType::Return,
            "self" => TokenType::SelfValue,
            "Self" => TokenType::SelfType,
            "static" => TokenType::Static,
            "struct" => TokenType::Struct,
            "super" => TokenType::Super,
            "trait" => TokenType::Trait,
            "true" => TokenType::True,
            "type" => TokenType::Type,
            "union" => TokenType::Union,
            "unsafe" => TokenType::Unsafe,
            "use" => TokenType::Use,
            "where" => TokenType::Where,
            "while" => TokenType::While,

            // Not a keyword, so it's an identifier
            _ => TokenType::Identifier(lexeme.clone()),
        };

        Ok(Some(Token::new(token_type, lexeme, span)))
    }

    // Helper methods

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    fn advance(&mut self) -> char {
        let ch = self.input[self.position];
        self.position += 1;

        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        ch
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.position]
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.position + 1 >= self.input.len() {
            None
        } else {
            Some(self.input[self.position + 1])
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), ' ' | '\t' | '\r' | '\n') {
            self.advance();
        }
    }

    fn skip_line_comment(&mut self) {
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> Result<()> {
        let mut depth = 1;

        while !self.is_at_end() && depth > 0 {
            if self.peek() == '/' && self.peek_next() == Some('*') {
                self.advance();
                self.advance();
                depth += 1;
            } else if self.peek() == '*' && self.peek_next() == Some('/') {
                self.advance();
                self.advance();
                depth -= 1;
            } else {
                self.advance();
            }
        }

        if depth > 0 {
            return Err(TlError::lexer(
                self.source.clone(),
                self.current_span(1),
                "Unterminated block comment",
            ));
        }

        Ok(())
    }

    fn get_lexeme(&self, start_pos: usize) -> String {
        self.input[start_pos..self.position].iter().collect()
    }

    fn span_from(&self, start_pos: usize) -> SourceSpan {
        SourceSpan::new(start_pos.into(), (self.position - start_pos))
    }

    fn current_span(&self, len: usize) -> SourceSpan {
        SourceSpan::new(self.position.into(), len)
    }
}

/// Convenience function to tokenize a string.
pub fn tokenize(source: String) -> Result<Vec<Token>> {
    let mut tokenizer = Tokenizer::new(source);
    tokenizer.tokenize()
}