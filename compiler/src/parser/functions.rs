// compiler/src/parser/functions.rs

use errors::CompileError;
use shared::ast::Stmt;
use shared::token::{Token, TokenType};
use crate::parser::statements::Parser;

/// Parse only top-level function declarations from a token stream.
pub fn parse_functions(tokens: &[Token]) -> Result<Vec<Stmt>, CompileError> {
    let mut funcs = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i].kind {
            TokenType::Fn => {
                // fn name(params) { body }
                i += 1;
                // function name
                let name = match tokens.get(i) {
                    Some(t) if t.kind == TokenType::Identifier => t.lexeme.clone(),
                    _ => {
                        return Err(CompileError::Parse { pos: i, msg: "expected function name".to_string() });
                    }
                };
                i += 1;
                // parameters
                if tokens.get(i).map(|t| t.kind.clone()) != Some(TokenType::LParen) {
                    return Err(CompileError::Parse { pos: i, msg: "expected `(` after function name".to_string() });
                }
                i += 1;
                let mut params = Vec::new();
                // empty param list or identifiers separated by commas
                while let Some(t) = tokens.get(i) {
                    if t.kind == TokenType::RParen {
                        break;
                    }
                    if t.kind == TokenType::Identifier {
                        params.push(t.lexeme.clone());
                        i += 1;
                        if tokens.get(i).map(|t| t.kind.clone()) == Some(TokenType::Comma) {
                            i += 1;
                        }
                        continue;
                    }
                    return Err(CompileError::Parse { pos: i, msg: "invalid parameter".to_string() });
                }
                // consume ')'
                if tokens.get(i).map(|t| t.kind.clone()) != Some(TokenType::RParen) {
                    return Err(CompileError::Parse { pos: i, msg: "missing `)` after parameters".to_string() });
                }
                i += 1;
                // expect '{'
                if tokens.get(i).map(|t| t.kind.clone()) != Some(TokenType::LBrace) {
                    return Err(CompileError::Parse { pos: i, msg: "expected `{` to start the function body".to_string() });
                }
                // find matching '}'
                let mut brace_count = 1;
                let body_start = i + 1;
                i += 1;
                while i < tokens.len() && brace_count > 0 {
                    match tokens[i].kind {
                        TokenType::LBrace => brace_count += 1,
                        TokenType::RBrace => brace_count -= 1,
                        _ => {}
                    }
                    i += 1;
                }
                if brace_count != 0 {
                    return Err(CompileError::Parse { pos: i, msg: "unclosed function body".to_string() });
                }
                let body_end = i - 1; // position of matching '}'
                // parse body statements
                let body_tokens = tokens[body_start..body_end].to_vec();
                let mut p = Parser::from_tokens(body_tokens);
                let stmts = p.parse_all()?;
                funcs.push(Stmt::Function(name, params, stmts));
            }
            TokenType::Eof => break,
            _ => i += 1,
        }
    }
    Ok(funcs)
}
