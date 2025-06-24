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
            '.' => {
                if self.peek().is_ascii_digit() {
                    // Start of float literal like .5
                    return self.number_token(start_pos, true);
                } else if self.match_char('.') {
                    if self.match_char('.') {
                        TokenType::DotDotDot
                    } else {
                        TokenType::DotDot
                    }
                } else {
                    TokenType::Dot
                }
            }

            // Potentially multi-character tokens
            '+' => {
                if self.match_char('=') {
                    TokenType::PlusEq
                } else if self.match_char('+') {
                    TokenType::PlusPlus
                } else {
                    TokenType::Plus
                }
            }
            '-' => {
                if self.match_char('=') {
                    TokenType::MinusEq
                } else if self.match_char('-') {
                    TokenType::MinusMinus
                } else if self.match_char('>') {
                    TokenType::Arrow
                } else {
                    TokenType::Minus
                }
            }
            '*' => {
                if self.match_char('=') {
                    TokenType::StarEq
                } else {
                    TokenType::Star
                }
            }
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
            }
            '%' => {
                if self.match_char('=') {
                    TokenType::PercentEq
                } else {
                    TokenType::Percent
                }
            }
            '=' => {
                if self.match_char('=') {
                    TokenType::EqualEqual
                } else if self.match_char('>') {
                    TokenType::FatArrow
                } else {
                    TokenType::Equal
                }
            }
            '!' => {
                if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                }
            }
            '<' => {
                if self.match_char('=') {
                    TokenType::LessEqual
                } else if self.match_char('<') {
                    if self.match_char('=') {
                        TokenType::LeftShiftEq
                    } else {
                        TokenType::LeftShift
                    }
                } else {
                    TokenType::Less
                }
            }
            '>' => {
                if self.match_char('=') {
                    TokenType::GreaterEqual
                } else if self.match_char('>') {
                    if self.match_char('=') {
                        TokenType::RightShiftEq
                    } else {
                        TokenType::RightShift
                    }
                } else {
                    TokenType::Greater
                }
            }
            '&' => {
                if self.match_char('&') {
                    TokenType::AndAnd
                } else if self.match_char('=') {
                    TokenType::AndEq
                } else {
                    TokenType::And
                }
            }
            '|' => {
                if self.match_char('|') {
                    TokenType::OrOr
                } else if self.match_char('=') {
                    TokenType::OrEq
                } else {
                    TokenType::Or
                }
            }
            '^' => {
                if self.match_char('=') {
                    TokenType::CaretEq
                } else {
                    TokenType::Caret
                }
            }
            ':' => {
                if self.match_char(':') {
                    TokenType::ColonColon
                } else {
                    TokenType::Colon
                }
            }
            '?' => TokenType::Question,
            '@' => TokenType::At,
            '#' => TokenType::Hash,
            '$' => TokenType::Dollar,
            '~' => TokenType::Tilde,

            // String literals
            '"' => return self.string_token(start_pos),
            '\'' => return self.char_token(start_pos),

            // Raw string literals
            'r' if self.peek() == '"' || self.peek() == '#' => {
                return self.raw_string_token(start_pos);
            }

            // Numbers
            '0'..='9' => return self.number_token(start_pos, false),

            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => return self.identifier_or_keyword(start_pos),

            // Unexpected character
            unexpected => {
                return Err(TlError::lexer(
                    self.source.clone(),
                    self.span_from(start_pos),
                    format!("Unexpected character: '{}'", unexpected),
                ));
            }
        };

        let lexeme = self.get_lexeme(start_pos);
        let span = self.span_from(start_pos);
        Ok(Some(Token::new(token_type, lexeme, span)))
    }

    /// Parse a string literal.
    fn string_token(&mut self, start_pos: usize) -> Result<Option<Token>> {
        let mut value = String::new();
        let mut terminated = false;

        while !self.is_at_end() && !terminated {
            let ch = self.advance();
            match ch {
                '"' => terminated = true,
                '\\' => {
                    if self.is_at_end() {
                        break;
                    }
                    let escaped = self.advance();
                    match escaped {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '\\' => value.push('\\'),
                        '\'' => value.push('\''),
                        '"' => value.push('"'),
                        '0' => value.push('\0'),
                        'x' => {
                            // Hex escape sequence \xNN
                            let hex = self.read_hex_escape(2)?;
                            if let Some(ch) = char::from_u32(hex) {
                                value.push(ch);
                            } else {
                                return Err(TlError::lexer(
                                    self.source.clone(),
                                    self.span_from(start_pos),
                                    "Invalid hex escape sequence",
                                ));
                            }
                        }
                        'u' => {
                            // Unicode escape sequence \u{...}
                            if self.match_char('{') {
                                let hex = self.read_unicode_escape()?;
                                if let Some(ch) = char::from_u32(hex) {
                                    value.push(ch);
                                } else {
                                    return Err(TlError::lexer(
                                        self.source.clone(),
                                        self.span_from(start_pos),
                                        "Invalid unicode escape sequence",
                                    ));
                                }
                            } else {
                                return Err(TlError::lexer(
                                    self.source.clone(),
                                    self.span_from(start_pos),
                                    "Expected '{' after \\u",
                                ));
                            }
                        }
                        other => {
                            return Err(TlError::lexer(
                                self.source.clone(),
                                self.span_from(start_pos),
                                format!("Invalid escape sequence: \\{}", other),
                            ));
                        }
                    }
                }
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                    value.push(ch);
                }
                other => value.push(other),
            }
        }

        if !terminated {
            return Err(TlError::lexer(
                self.source.clone(),
                self.span_from(start_pos),
                "Unterminated string literal",
            ));
        }

        let span = self.span_from(start_pos);
        Ok(Some(Token::new(TokenType::String(value), self.get_lexeme(start_pos), span)))
    }

    /// Parse a character literal.
    fn char_token(&mut self, start_pos: usize) -> Result<Option<Token>> {
        if self.is_at_end() {
            return Err(TlError::lexer(
                self.source.clone(),
                self.span_from(start_pos),
                "Unterminated character literal",
            ));
        }

        let ch = self.advance();
        let value = match ch {
            '\\' => {
                if self.is_at_end() {
                    return Err(TlError::lexer(
                        self.source.clone(),
                        self.span_from(start_pos),
                        "Unterminated character literal",
                    ));
                }
                let escaped = self.advance();
                match escaped {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    '0' => '\0',
                    'x' => {
                        let hex = self.read_hex_escape(2)?;
                        char::from_u32(hex).ok_or_else(|| {
                            TlError::lexer(
                                self.source.clone(),
                                self.span_from(start_pos),
                                "Invalid hex escape in character literal",
                            )
                        })?
                    }
                    'u' => {
                        if !self.match_char('{') {
                            return Err(TlError::lexer(
                                self.source.clone(),
                                self.span_from(start_pos),
                                "Expected '{' after \\u",
                            ));
                        }
                        let hex = self.read_unicode_escape()?;
                        char::from_u32(hex).ok_or_else(|| {
                            TlError::lexer(
                                self.source.clone(),
                                self.span_from(start_pos),
                                "Invalid unicode escape in character literal",
                            )
                        })?
                    }
                    other => {
                        return Err(TlError::lexer(
                            self.source.clone(),
                            self.span_from(start_pos),
                            format!("Invalid escape sequence in character literal: \\{}", other),
                        ));
                    }
                }
            }
            other => other,
        };

        if !self.match_char('\'') {
            return Err(TlError::lexer(
                self.source.clone(),
                self.span_from(start_pos),
                "Expected closing quote for character literal",
            ));
        }

        let span = self.span_from(start_pos);
        Ok(Some(Token::new(TokenType::Char(value), self.get_lexeme(start_pos), span)))
    }

    /// Parse a raw string literal (r"..." or r#"..."#).
    fn raw_string_token(&mut self, start_pos: usize) -> Result<Option<Token>> {
        // Count the number of # characters
        let mut hash_count = 0;
        while self.match_char('#') {
            hash_count += 1;
        }

        if !self.match_char('"') {
            return Err(TlError::lexer(
                self.source.clone(),
                self.span_from(start_pos),
                "Expected '\"' after raw string prefix",
            ));
        }

        let mut value = String::new();
        let mut found_end = false;

        while !self.is_at_end() && !found_end {
            if self.peek() == '"' {
                self.advance(); // consume '"'

                // Check if we have the right number of # characters
                let mut matching_hashes = 0;
                while matching_hashes < hash_count && self.peek() == '#' {
                    self.advance();
                    matching_hashes += 1;
                }

                if matching_hashes == hash_count {
                    found_end = true;
                } else {
                    // False alarm, add the consumed characters to the value
                    value.push('"');
                    for _ in 0..matching_hashes {
                        value.push('#');
                    }
                }
            } else {
                let ch = self.advance();
                if ch == '\n' {
                    self.line += 1;
                    self.column = 1;
                }
                value.push(ch);
            }
        }

        if !found_end {
            return Err(TlError::lexer(
                self.source.clone(),
                self.span_from(start_pos),
                "Unterminated raw string literal",
            ));
        }

        let span = self.span_from(start_pos);
        Ok(Some(Token::new(TokenType::String(value), self.get_lexeme(start_pos), span)))
    }

    /// Parse a number literal (integer or float).
    fn number_token(&mut self, start_pos: usize, starts_with_dot: bool) -> Result<Option<Token>> {
        let mut has_dot = starts_with_dot;
        let mut has_exponent = false;

        // If we start with a dot, we've already consumed it
        if !starts_with_dot {
            // Handle different number bases
            if self.input.get(self.position.saturating_sub(1)) == Some(&'0') {
                match self.peek() {
                    'x' | 'X' => {
                        self.advance(); // consume 'x'
                        return self.hex_number_token(start_pos);
                    }
                    'o' | 'O' => {
                        self.advance(); // consume 'o'
                        return self.octal_number_token(start_pos);
                    }
                    'b' | 'B' => {
                        self.advance(); // consume 'b'
                        return self.binary_number_token(start_pos);
                    }
                    _ => {} // Regular decimal number
                }
            }

            // Consume digits
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        // Handle fractional part
        if !has_dot && self.peek() == '.' && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
            has_dot = true;
            self.advance(); // consume '.'
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        // Handle exponent
        if matches!(self.peek(), 'e' | 'E') {
            has_exponent = true;
            self.advance(); // consume 'e' or 'E'

            if matches!(self.peek(), '+' | '-') {
                self.advance(); // consume sign
            }

            if !self.peek().is_ascii_digit() {
                return Err(TlError::lexer(
                    self.source.clone(),
                    self.span_from(start_pos),
                    "Expected digits after exponent",
                ));
            }

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let lexeme = self.get_lexeme(start_pos);
        let span = self.span_from(start_pos);

        let token_type = if has_dot || has_exponent {
            // Parse as float
            let value: f64 = lexeme.parse().map_err(|_| {
                TlError::lexer(
                    self.source.clone(),
                    span,
                    format!("Invalid float literal: {}", lexeme),
                )
            })?;
            TokenType::Float(value)
        } else {
            // Parse as integer
            let value: i64 = lexeme.parse().map_err(|_| {
                TlError::lexer(
                    self.source.clone(),
                    span,
                    format!("Invalid integer literal: {}", lexeme),
                )
            })?;
            TokenType::Integer(value)
        };

        Ok(Some(Token::new(token_type, lexeme, span)))
    }

    /// Parse a hexadecimal number literal.
    fn hex_number_token(&mut self, start_pos: usize) -> Result<Option<Token>> {
        if !self.peek().is_ascii_hexdigit() {
            return Err(TlError::lexer(
                self.source.clone(),
                self.span_from(start_pos),
                "Expected hexadecimal digits after '0x'",
            ));
        }

        while self.peek().is_ascii_hexdigit() {
            self.advance();
        }

        let lexeme = self.get_lexeme(start_pos);
        let span = self.span_from(start_pos);

        // Remove '0x' prefix for parsing
        let hex_part = &lexeme[2..];
        let value = i64::from_str_radix(hex_part, 16).map_err(|_| {
            TlError::lexer(
                self.source.clone(),
                span,
                format!("Invalid hexadecimal literal: {}", lexeme),
            )
        })?;

        Ok(Some(Token::new(TokenType::Integer(value), lexeme, span)))
    }

    /// Parse an octal number literal.
    fn octal_number_token(&mut self, start_pos: usize) -> Result<Option<Token>> {
        if !matches!(self.peek(), '0'..='7') {
            return Err(TlError::lexer(
                self.source.clone(),
                self.span_from(start_pos),
                "Expected octal digits after '0o'",
            ));
        }

        while matches!(self.peek(), '0'..='7') {
            self.advance();
        }

        let lexeme = self.get_lexeme(start_pos);
        let span = self.span_from(start_pos);

        // Remove '0o' prefix for parsing
        let octal_part = &lexeme[2..];
        let value = i64::from_str_radix(octal_part, 8).map_err(|_| {
            TlError::lexer(
                self.source.clone(),
                span,
                format!("Invalid octal literal: {}", lexeme),
            )
        })?;

        Ok(Some(Token::new(TokenType::Integer(value), lexeme, span)))
    }

    /// Parse a binary number literal.
    fn binary_number_token(&mut self, start_pos: usize) -> Result<Option<Token>> {
        if !matches!(self.peek(), '0' | '1') {
            return Err(TlError::lexer(
                self.source.clone(),
                self.span_from(start_pos),
                "Expected binary digits after '0b'",
            ));
        }

        while matches!(self.peek(), '0' | '1') {
            self.advance();
        }

        let lexeme = self.get_lexeme(start_pos);
        let span = self.span_from(start_pos);

        // Remove '0b' prefix for parsing
        let binary_part = &lexeme[2..];
        let value = i64::from_str_radix(binary_part, 2).map_err(|_| {
            TlError::lexer(
                self.source.clone(),
                span,
                format!("Invalid binary literal: {}", lexeme),
            )
        })?;

        Ok(Some(Token::new(TokenType::Integer(value), lexeme, span)))
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

    /// Read a hexadecimal escape sequence (\xNN).
    fn read_hex_escape(&mut self, digits: usize) -> Result<u32> {
        let mut value = 0;
        for _ in 0..digits {
            if self.is_at_end() {
                return Err(TlError::lexer(
                    self.source.clone(),
                    self.current_span(1),
                    "Incomplete hex escape sequence",
                ));
            }
            let ch = self.advance();
            if let Some(digit) = ch.to_digit(16) {
                value = value * 16 + digit;
            } else {
                return Err(TlError::lexer(
                    self.source.clone(),
                    self.current_span(1),
                    format!("Invalid hex digit: {}", ch),
                ));
            }
        }
        Ok(value)
    }

    /// Read a Unicode escape sequence (\u{...}).
    fn read_unicode_escape(&mut self) -> Result<u32> {
        let mut value = 0;
        let mut digit_count = 0;

        while !self.is_at_end() && self.peek() != '}' {
            let ch = self.advance();
            if let Some(digit) = ch.to_digit(16) {
                value = value * 16 + digit;
                digit_count += 1;
                if digit_count > 6 {
                    return Err(TlError::lexer(
                        self.source.clone(),
                        self.current_span(1),
                        "Unicode escape sequence too long",
                    ));
                }
            } else {
                return Err(TlError::lexer(
                    self.source.clone(),
                    self.current_span(1),
                    format!("Invalid hex digit in unicode escape: {}", ch),
                ));
            }
        }

        if !self.match_char('}') {
            return Err(TlError::lexer(
                self.source.clone(),
                self.current_span(1),
                "Expected '}' to close unicode escape",
            ));
        }

        if digit_count == 0 {
            return Err(TlError::lexer(
                self.source.clone(),
                self.current_span(1),
                "Empty unicode escape sequence",
            ));
        }

        Ok(value)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize_str(input: &str) -> Result<Vec<Token>> {
        tokenize(input.to_string())
    }

    #[test]
    fn test_empty_input() {
        let tokens = tokenize_str("").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].token_type, TokenType::Eof));
    }

    #[test]
    fn test_single_char_tokens() {
        let tokens = tokenize_str("(){}[],.;").unwrap();
        assert_eq!(tokens.len(), 9); // 8 tokens + EOF
        assert!(matches!(tokens[0].token_type, TokenType::LParen));
        assert!(matches!(tokens[1].token_type, TokenType::RParen));
        assert!(matches!(tokens[2].token_type, TokenType::LBrace));
        assert!(matches!(tokens[3].token_type, TokenType::RBrace));
    }

    #[test]
    fn test_operators() {
        let tokens = tokenize_str("+ - * / % = == != < <= > >=").unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::Plus));
        assert!(matches!(tokens[1].token_type, TokenType::Minus));
        assert!(matches!(tokens[2].token_type, TokenType::Star));
        assert!(matches!(tokens[3].token_type, TokenType::Slash));
        assert!(matches!(tokens[4].token_type, TokenType::Percent));
        assert!(matches!(tokens[5].token_type, TokenType::Equal));
        assert!(matches!(tokens[6].token_type, TokenType::EqualEqual));
        assert!(matches!(tokens[7].token_type, TokenType::BangEqual));
        assert!(matches!(tokens[8].token_type, TokenType::Less));
        assert!(matches!(tokens[9].token_type, TokenType::LessEqual));
        assert!(matches!(tokens[10].token_type, TokenType::Greater));
        assert!(matches!(tokens[11].token_type, TokenType::GreaterEqual));
    }

    #[test]
    fn test_compound_assignment() {
        let tokens = tokenize_str("+= -= *= /= %=").unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::PlusEq));
        assert!(matches!(tokens[1].token_type, TokenType::MinusEq));
        assert!(matches!(tokens[2].token_type, TokenType::StarEq));
        assert!(matches!(tokens[3].token_type, TokenType::SlashEq));
        assert!(matches!(tokens[4].token_type, TokenType::PercentEq));
    }

    #[test]
    fn test_keywords() {
        let tokens = tokenize_str("fn let if else while for").unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::Fn));
        assert!(matches!(tokens[1].token_type, TokenType::Let));
        assert!(matches!(tokens[2].token_type, TokenType::If));
        assert!(matches!(tokens[3].token_type, TokenType::Else));
        assert!(matches!(tokens[4].token_type, TokenType::While));
        assert!(matches!(tokens[5].token_type, TokenType::For));
    }

    #[test]
    fn test_identifiers() {
        let tokens = tokenize_str("hello world _underscore CamelCase").unwrap();
        for i in 0..4 {
            if let TokenType::Identifier(name) = &tokens[i].token_type {
                match i {
                    0 => assert_eq!(name, "hello"),
                    1 => assert_eq!(name, "world"),
                    2 => assert_eq!(name, "_underscore"),
                    3 => assert_eq!(name, "CamelCase"),
                    _ => unreachable!(),
                }
            } else {
                panic!("Expected identifier");
            }
        }
    }

    #[test]
    fn test_integers() {
        let tokens = tokenize_str("123 0 42").unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::Integer(123)));
        assert!(matches!(tokens[1].token_type, TokenType::Integer(0)));
        assert!(matches!(tokens[2].token_type, TokenType::Integer(42)));
    }

    #[test]
    fn test_floats() {
        let tokens = tokenize_str("3.14 0.5 .25 1e10 2.5e-3").unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::Float(x) if (x - 3.14).abs() < f64::EPSILON));
        assert!(matches!(tokens[1].token_type, TokenType::Float(x) if (x - 0.5).abs() < f64::EPSILON));
        assert!(matches!(tokens[2].token_type, TokenType::Float(x) if (x - 0.25).abs() < f64::EPSILON));
        assert!(matches!(tokens[3].token_type, TokenType::Float(x) if (x - 1e10).abs() < f64::EPSILON));
        assert!(matches!(tokens[4].token_type, TokenType::Float(x) if (x - 2.5e-3).abs() < f64::EPSILON));
    }

    #[test]
    fn test_hex_numbers() {
        let tokens = tokenize_str("0x10 0xFF 0x0").unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::Integer(16)));
        assert!(matches!(tokens[1].token_type, TokenType::Integer(255)));
        assert!(matches!(tokens[2].token_type, TokenType::Integer(0)));
    }

    #[test]
    fn test_binary_numbers() {
        let tokens = tokenize_str("0b1010 0b1111 0b0").unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::Integer(10)));
        assert!(matches!(tokens[1].token_type, TokenType::Integer(15)));
        assert!(matches!(tokens[2].token_type, TokenType::Integer(0)));
    }

    #[test]
    fn test_octal_numbers() {
        let tokens = tokenize_str("0o10 0o17 0o0").unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::Integer(8)));
        assert!(matches!(tokens[1].token_type, TokenType::Integer(15)));
        assert!(matches!(tokens[2].token_type, TokenType::Integer(0)));
    }

    #[test]
    fn test_strings() {
        let tokens = tokenize_str(r#""hello" "world\n" "escaped\"quote""#).unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::String(ref s) if s == "hello"));
        assert!(matches!(tokens[1].token_type, TokenType::String(ref s) if s == "world\n"));
        assert!(matches!(tokens[2].token_type, TokenType::String(ref s) if s == "escaped\"quote"));
    }

    #[test]
    fn test_characters() {
        let tokens = tokenize_str("'a' 'Z' '\\n' '\\''").unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::Char('a')));
        assert!(matches!(tokens[1].token_type, TokenType::Char('Z')));
        assert!(matches!(tokens[2].token_type, TokenType::Char('\n')));
        assert!(matches!(tokens[3].token_type, TokenType::Char('\'')));
    }

    #[test]
    fn test_raw_strings() {
        let tokens = tokenize_str(r###"r"raw" r#"raw with "quotes""# r##"nested #"##"###).unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::String(ref s) if s == "raw"));
        assert!(matches!(tokens[1].token_type, TokenType::String(ref s) if s == "raw with \"quotes\""));
        assert!(matches!(tokens[2].token_type, TokenType::String(ref s) if s == "nested #"));
    }

    #[test]
    fn test_comments() {
        let tokens = tokenize_str("// line comment\n/* block comment */ hello").unwrap();
        // Comments should be skipped, so we should only get 'hello' + EOF
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].token_type, TokenType::Identifier(ref s) if s == "hello"));
    }

    #[test]
    fn test_nested_block_comments() {
        let tokens = tokenize_str("/* outer /* inner */ still in outer */ hello").unwrap();
        assert_eq!(tokens.len(), 2); // 'hello' + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Identifier(ref s) if s == "hello"));
    }

    #[test]
    fn test_whitespace_handling() {
        let tokens = tokenize_str("  \t\n  hello  \n\t  world  ").unwrap();
        assert_eq!(tokens.len(), 3); // 'hello', 'world', EOF
        assert!(matches!(tokens[0].token_type, TokenType::Identifier(ref s) if s == "hello"));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(ref s) if s == "world"));
    }

    #[test]
    fn test_error_unterminated_string() {
        let result = tokenize_str(r#""unterminated"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_invalid_escape() {
        let result = tokenize_str(r#""\q""#);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_unterminated_char() {
        let result = tokenize_str("'a");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_invalid_number() {
        let result = tokenize_str("0x"); // Hex prefix without digits
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_expression() {
        let source = "fn main() { let x = 42.5 + (3 * 4); }";
        let tokens = tokenize_str(source).unwrap();

        // Just verify it parses without panicking and has reasonable token count
        assert!(tokens.len() > 10);
        assert!(matches!(tokens[0].token_type, TokenType::Fn));
        assert!(matches!(tokens.last().unwrap().token_type, TokenType::Eof));
    }
}