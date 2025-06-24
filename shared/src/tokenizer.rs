// File: shared/src/tokenizer.rs - COMPLETE REWRITE
// -----------------------------------------------------------------------------

//! Complete tokenizer implementation for T-Lang source code.

use crate::token::{Token, TokenType};
use errors::{Result, TlError};
use miette::SourceSpan;

/// Complete tokenizer for T-Lang with full feature support
pub struct Tokenizer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    source: String,
}

impl Tokenizer {
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

    /// Tokenize the complete input with comprehensive error handling
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            match self.scan_token() {
                Ok(Some(token)) => tokens.push(token),
                Ok(None) => continue, // Skip whitespace/comments
                Err(e) => return Err(e),
            }
        }

        // Always add EOF token
        tokens.push(Token::new(
            TokenType::Eof,
            String::new(),
            self.current_span(0),
        ));

        Ok(tokens)
    }

    /// Scan a single token with complete implementation
    fn scan_token(&mut self) -> Result<Option<Token>> {
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

            // Operators (with lookahead for compound operators)
            '+' => {
                if self.match_char('=') {
                    TokenType::PlusEq
                } else {
                    TokenType::Plus
                }
            },
            '-' => {
                if self.match_char('=') {
                    TokenType::MinusEq
                } else if self.match_char('>') {
                    TokenType::Arrow
                } else {
                    TokenType::Minus
                }
            },
            '*' => {
                if self.match_char('=') {
                    TokenType::StarEq
                } else {
                    TokenType::Star
                }
            },
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
            '%' => TokenType::Percent,

            // Comparison operators
            '=' => {
                if self.match_char('=') {
                    TokenType::EqualEqual
                } else if self.match_char('>') {
                    TokenType::FatArrow
                } else {
                    TokenType::Equal
                }
            },
            '!' => {
                if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                }
            },
            '<' => {
                if self.match_char('=') {
                    TokenType::LessEqual
                } else if self.match_char('<') {
                    TokenType::LessLess
                } else {
                    TokenType::Less
                }
            },
            '>' => {
                if self.match_char('=') {
                    TokenType::GreaterEqual
                } else if self.match_char('>') {
                    TokenType::GreaterGreater
                } else {
                    TokenType::Greater
                }
            },

            // Logical operators
            '&' => {
                if self.match_char('&') {
                    TokenType::AmpAmp
                } else {
                    TokenType::Amp
                }
            },
            '|' => {
                if self.match_char('|') {
                    TokenType::PipePipe
                } else {
                    TokenType::Pipe
                }
            },
            '^' => TokenType::Caret,
            '~' => TokenType::Tilde,

            // Punctuation
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

    /// Parse string literals with escape sequences
    fn string_literal(&mut self) -> Result<Option<Token>> {
        let start_pos = self.position - 1;
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
                    'u' => {
                        // Unicode escape sequence \u{xxxx}
                        if self.match_char('{') {
                            self.parse_unicode_escape()?
                        } else {
                            return Err(TlError::lexer(
                                self.source.clone(),
                                self.current_span(1),
                                "Invalid unicode escape sequence",
                            ));
                        }
                    },
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
                if self.peek() == '\n' {
                    self.line += 1;
                    self.column = 1;
                }
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

        // Consume closing quote
        self.advance();

        let lexeme = self.get_lexeme(start_pos);
        let span = self.span_from(start_pos);

        Ok(Some(Token::new(TokenType::String(value), lexeme, span)))
    }

    /// Parse character literals
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
                '\'' => '\'',
                '"' => '"',
                '0' => '\0',
                c => {
                    return Err(TlError::lexer(
                        self.source.clone(),
                        self.current_span(1),
                        format!("Invalid escape sequence in character literal: \\{}", c),
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

        self.advance(); // consume closing quote

        let lexeme = self.get_lexeme(start_pos);
        let span = self.span_from(start_pos);

        Ok(Some(Token::new(TokenType::Char(ch), lexeme, span)))
    }

    /// Parse number literals (integers and floats)
    fn number_literal(&mut self, start_pos: usize) -> Result<Option<Token>> {
        // Consume digits
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        let mut is_float = false;

        // Check for decimal point
        if self.peek() == '.' && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
            is_float = true;
            self.advance(); // consume '.'

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        // Check for scientific notation
        if matches!(self.peek(), 'e' | 'E') {
            is_float = true;
            self.advance(); // consume 'e' or 'E'

            if matches!(self.peek(), '+' | '-') {
                self.advance(); // consume sign
            }

            if !self.peek().is_ascii_digit() {
                return Err(TlError::lexer(
                    self.source.clone(),
                    self.current_span(1),
                    "Invalid number literal: expected digits after exponent",
                ));
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

    /// Parse identifiers and keywords
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

    /// Skip line comments (// ...)
    fn skip_line_comment(&mut self) {
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    /// Skip block comments (/* ... */) with nesting support
    fn skip_block_comment(&mut self) -> Result<()> {
        let mut nesting_level = 1;

        while !self.is_at_end() && nesting_level > 0 {
            if self.peek() == '/' && self.peek_next() == Some('*') {
                nesting_level += 1;
                self.advance();
                self.advance();
            } else if self.peek() == '*' && self.peek_next() == Some('/') {
                nesting_level -= 1;
                self.advance();
                self.advance();
            } else {
                if self.peek() == '\n' {
                    self.line += 1;
                    self.column = 1;
                }
                self.advance();
            }
        }

        if nesting_level > 0 {
            return Err(TlError::lexer(
                self.source.clone(),
                self.current_span(1),
                "Unterminated block comment",
            ));
        }

        Ok(())
    }

    /// Parse unicode escape sequence \u{xxxx}
    fn parse_unicode_escape(&mut self) -> Result<char> {
        let mut hex_digits = String::new();

        while !self.is_at_end() && self.peek() != '}' && hex_digits.len() < 6 {
            let ch = self.advance();
            if ch.is_ascii_hexdigit() {
                hex_digits.push(ch);
            } else {
                return Err(TlError::lexer(
                    self.source.clone(),
                    self.current_span(1),
                    "Invalid character in unicode escape sequence",
                ));
            }
        }

        if hex_digits.is_empty() {
            return Err(TlError::lexer(
                self.source.clone(),
                self.current_span(1),
                "Empty unicode escape sequence",
            ));
        }

        if self.peek() != '}' {
            return Err(TlError::lexer(
                self.source.clone(),
                self.current_span(1),
                "Unterminated unicode escape sequence",
            ));
        }

        self.advance(); // consume '}'

        let code_point = match u32::from_str_radix(&hex_digits, 16) {
            Ok(cp) => cp,
            Err(_) => {
                return Err(TlError::lexer(
                    self.source.clone(),
                    self.current_span(1),
                    "Invalid unicode escape sequence",
                ));
            }
        };

        match char::from_u32(code_point) {
            Some(ch) => Ok(ch),
            None => Err(TlError::lexer(
                self.source.clone(),
                self.current_span(1),
                "Invalid unicode code point",
            )),
        }
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
        while matches!(self.peek(), ' ' | '\r' | '\t' | '\n') {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            self.advance();
        }
    }

    fn get_lexeme(&self, start_pos: usize) -> String {
        self.input[start_pos..self.position].iter().collect()
    }

    fn span_from(&self, start_pos: usize) -> SourceSpan {
        let length = self.position - start_pos;
        SourceSpan::new(start_pos.into(), length)
    }

    fn current_span(&self, length: usize) -> SourceSpan {
        SourceSpan::new(self.position.into(), length)
    }
}

/// Convenience function for tokenizing source code
pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut tokenizer = Tokenizer::new(source.to_string());
    tokenizer.tokenize()
}