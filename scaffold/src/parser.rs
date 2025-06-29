//! scaffold/src/parser.rs - Hardcoded parser for `fn main() -> i32 { return 42; }`
//!
//! This is intentionally hardcoded and brittle - it only needs to parse ONE program.

use crate::ast::*;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String),
    UnexpectedEof,
    InvalidSyntax(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken(token) => write!(f, "Unexpected token: {token}"),
            ParseError::UnexpectedEof => write!(f, "Unexpected end of file"),
            ParseError::InvalidSyntax(msg) => write!(f, "Invalid syntax: {msg}"),
        }
    }
}

impl std::error::Error for ParseError {}

pub struct Parser {
    tokens: Vec<String>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        // Strip BOM if present
        let input = input.strip_prefix('\u{FEFF}').unwrap_or(input);

        // Extremely basic tokenization - just split on whitespace and common delimiters
        let mut tokens = Vec::new();
        let mut current_token = String::new();

        for ch in input.chars() {
            match ch {
                ' ' | '\t' | '\n' | '\r' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                }
                '(' | ')' | '{' | '}' | ';' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                    tokens.push(ch.to_string());
                }
                '-' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                    // Look ahead for ">"
                    tokens.push(ch.to_string());
                }
                '>' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                    tokens.push(ch.to_string());
                }
                _ => current_token.push(ch),
            }
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut program = Program::new();

        // Parse exactly: fn main() -> i32 { return 42; }
        self.expect_token("fn")?;
        let name = self.expect_identifier()?;
        self.expect_token("(")?;
        self.expect_token(")")?;
        self.expect_token("-")?;
        self.expect_token(">")?;
        let return_type = self.expect_identifier()?;
        self.expect_token("{")?;
        self.expect_token("return")?;
        let value = self.expect_number()?;
        self.expect_token(";")?;
        self.expect_token("}")?;

        // Build the AST
        let literal = Literal::Integer(value);
        let expr = Expression::Literal(literal);
        let stmt = Statement::Return(expr);
        let block = Block {
            statements: vec![stmt],
        };

        let mut function = Function::new(name);
        function.return_type = Some(Type { name: return_type });
        function.body = block;

        program.functions.push(function);

        Ok(program)
    }

    fn current_token(&self) -> Option<&String> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<String> {
        let token = self.current_token().cloned();
        self.pos += 1;
        token
    }

    fn expect_token(&mut self, expected: &str) -> Result<(), ParseError> {
        match self.advance() {
            Some(token) if token == expected => Ok(()),
            Some(token) => Err(ParseError::UnexpectedToken(token)),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        match self.advance() {
            Some(token) if token.chars().all(|c| c.is_alphanumeric() || c == '_') => {
                Ok(token)
            }
            Some(token) => Err(ParseError::UnexpectedToken(token)),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    fn expect_number(&mut self) -> Result<i64, ParseError> {
        match self.advance() {
            Some(token) => {
                token.parse::<i64>()
                    .map_err(|_| ParseError::InvalidSyntax(format!("Expected number, got {token}")))
            }
            None => Err(ParseError::UnexpectedEof),
        }
    }
}