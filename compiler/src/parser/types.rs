// compiler/src/parser/types.rs

use errors::CompileError;
use shared::ast::Stmt;
use shared::token::{Token, TokenType};

/// Parse any top‐level type declarations from the token stream,
/// returning a Vec<Stmt> (empty for now) or a Parse error.
pub fn parse_types(tokens: &[Token]) -> Result<Vec<Stmt>, CompileError> {
    let mut types = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i].kind {
            // In future we might support `type Name = ...;`
            // but for now, just skip ahead
            TokenType::Eof => break,
            _ => i += 1,
        }
    }
    Ok(types)
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::token::{Token, TokenType};

    fn mk_tok(kind: TokenType, lexeme: &str) -> Token {
        Token::new(kind, lexeme, 1)
    }

    #[test]
    fn parse_types_empty() {
        let tokens = &[mk_tok(TokenType::Eof, "")];
        let ts = parse_types(tokens).unwrap();
        assert!(ts.is_empty());
    }
}
