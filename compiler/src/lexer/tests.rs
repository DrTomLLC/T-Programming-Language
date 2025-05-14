// compiler/src/lexer/tests.rs

use shared::tokenizer::tokenize;
use shared::token::TokenType;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_keyword() {
        let tokens = tokenize("let");
        assert_eq!(tokens[0].kind, TokenType::Let);
        assert_eq!(tokens[1].kind, TokenType::Eof);
    }

    #[test]
    fn test_number_token() {
        let tokens = tokenize("123");
        assert_eq!(tokens[0].kind, TokenType::Number);
    }

    #[test]
    fn test_identifier_token() {
        let tokens = tokenize("foobar");
        assert_eq!(tokens[0].kind, TokenType::Identifier);
    }

    #[test]
    fn test_multiple_tokens() {
        let tokens = tokenize("let x = 42");
        assert_eq!(tokens[0].kind, TokenType::Let);
        assert_eq!(tokens[1].kind, TokenType::Identifier);
        assert_eq!(tokens[2].kind, TokenType::Equals);
        assert_eq!(tokens[3].kind, TokenType::Number);
    }

    #[test]
    fn test_symbol_token() {
        let tokens = tokenize("= + - * / ( ) { } , ;");
        assert_eq!(tokens[0].kind, TokenType::Equals);
        assert_eq!(tokens[1].kind, TokenType::Plus);
        assert_eq!(tokens[2].kind, TokenType::Minus);
        assert_eq!(tokens[3].kind, TokenType::Star);
        assert_eq!(tokens[4].kind, TokenType::Slash);
        assert_eq!(tokens[5].kind, TokenType::LParen);
        assert_eq!(tokens[6].kind, TokenType::RParen);
        assert_eq!(tokens[7].kind, TokenType::LBrace);
        assert_eq!(tokens[8].kind, TokenType::RBrace);
        assert_eq!(tokens[9].kind, TokenType::Comma);
        assert_eq!(tokens[10].kind, TokenType::Semicolon);
        assert_eq!(tokens[11].kind, TokenType::Eof);
    }
}
