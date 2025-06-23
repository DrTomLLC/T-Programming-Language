// compiler/src/parser/entry.rs

//! The `entry` module: top‑level parser driver for T‑Lang.
//! Turns a vector of RawToken into an AST or returns a detailed compile‑time error.

//! The entry‑point for parsing a token stream into AST statements.

use miette::SourceSpan;
use errors::TlError;
use errors::parse as parse_err;
use shared::token::{Token, TokenType};
use shared::ast::{Stmt, Span};
#[allow(dead_code)]
/// A thin wrapper around our parse errors.
pub type ParseError = TlError;


/// Parse a flat Vec<Token> into a Vec<Stmt>.  This is the only
/// free‑function entry‑point your `compiler/src/lib.rs` should call.
pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, ParseError> {
    let mut p = Parser::new(tokens);
    p.parse_all()
}

//////////////////////////////////////////////////////////////////////////
/// Internal recursive‑descent parser
//////////////////////////////////////////////////////////////////////////

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Construct from a full token stream.
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// Parse until EOF.
    pub fn parse_all(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        while !self.peek_is(TokenType::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    /// Peek at the current token’s kind.
    fn peek_kind(&self) -> TokenType {
        self.tokens
            .get(self.pos)
            .map(|t| t.kind.clone())
            .unwrap_or(TokenType::Eof)
    }

    fn peek_is(&self, kind: TokenType) -> bool {
        self.peek_kind() == kind
    }

    /// Advance and return the token if it matches `kind`.
    #[allow(dead_code)]
    fn advance_if(&mut self, kind: TokenType) -> Option<Token> {
        if self.peek_is(kind.clone()) {
            let t = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(t)
        } else {
            None
        }
    }

    /// Expect and consume `kind`, or error.
    #[allow(dead_code)]
    fn expect(&mut self, kind: TokenType) -> Result<Token, ParseError> {
        if let Some(tok) = self.advance_if(kind.clone()) {
            Ok(tok)
        } else {
            let _span = self.current_span();
            Err(parse_err::missing_token(
                "<input>",
                &"<source>",
                self.current_source_span(),
                &format!("{:?}", kind),
            ))
        }
    }

    /// Return the span of the current token.
    fn current_span(&self) -> Span {
        let t = &self.tokens[self.pos];
        Span {
            start: t.col,
            len: t.lexeme.len(),
        }
    }

    fn current_source_span(&self) -> SourceSpan {
        let t = &self.tokens[self.pos];
        (t.col, t.lexeme.len()).into()
    }

    /// Parse one statement.  (Here’s where you dispatch to functions,
    /// if/while, let/const, expr, etc.)
    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        //–– For brevity, stubbed out. Replace this with your real
        //    `parse_let`, `parse_if`, `parse_while`, `parse_expr_stmt`, etc.
        unimplemented!("parse_stmt still to be filled in");
    }
}
