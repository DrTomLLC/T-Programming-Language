// compiler/src/parser/modules.rs

use crate::parser::{Parser, ParseError, Result};
use crate::parser::declarations::parse_declaration;
use crate::ast::{AST, Module};
use crate::lexer::{Keyword, TokenKind};

/// Parse the entire program into an AST containing modules.
pub fn parse_program(parser: &mut Parser) -> Result<AST> {
    let mut modules = Vec::new();
    while !parser.peek(TokenKind::Eof) {
        modules.push(parse_module(parser)?);
    }
    Ok(AST { modules })
}

/// Parse a single `module Name;` and its declarations.
///
/// Grammar:
/// ```ebnf
/// <Module> ::= "module" <Identifier> ";" { <Declaration> }
/// ```
pub fn parse_module(parser: &mut Parser) -> Result<Module> {
    parser.expect_keyword(Keyword::Module)?;
    let name = parser.expect_identifier()?;
    parser.expect(TokenKind::Semicolon)?;

    let mut declarations = Vec::new();
    while !parser.peek_keyword(Keyword::Module) && !parser.peek(TokenKind::Eof) {
        declarations.push(parse_declaration(parser)?);
    }

    Ok(Module { name, declarations })
}
