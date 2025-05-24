// shared/src/tokenizer.rs
use crate::token::{Token, TokenType};
use errors::TlError;

/// Convert the input source string into a sequence of typed tokens or a lex‑error.
pub fn tokenize(source: &str) -> Result<Vec<Token>, TlError> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    let mut line = 1_usize;
    let mut col = 1_usize;

    while let Some(&c) = chars.peek() {
        // Skip and track whitespace
        if c.is_whitespace() {
            chars.next();
            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
            continue;
        }

        // remember where this lexeme starts
        let start_col = col;

        // build up the raw lexeme string
        let lexeme = if c.is_alphabetic() || c == '_' {
            let mut s = String::new();
            while let Some(&pc) = chars.peek() {
                if pc.is_alphanumeric() || pc == '_' {
                    s.push(pc);
                    chars.next();
                    col += 1;
                } else {
                    break;
                }
            }
            s
        } else if c.is_ascii_digit() {
            let mut s = String::new();
            while let Some(&pc) = chars.peek() {
                if pc.is_ascii_digit() || pc == '.' {
                    s.push(pc);
                    chars.next();
                    col += 1;
                } else {
                    break;
                }
            }
            s
        } else if c == '"' {
            let mut s = String::new();
            s.push(c);
            chars.next();
            col += 1;
            while let Some(&pc) = chars.peek() {
                s.push(pc);
                chars.next();
                col += 1;
                if pc == '"' {
                    break;
                }
            }
            s
        } else {
            // two‑char operators?
            let mut pair = String::new();
            pair.push(c);
            if let Some(nc) = chars.clone().nth(1) {
                let two = format!("{}{}", c, nc);
                if ["==", "!=", "<=", ">="].contains(&two.as_str()) {
                    chars.next(); col += 1;
                    chars.next(); col += 1;
                    two
                } else {
                    chars.next(); col += 1;
                    pair
                }
            } else {
                chars.next(); col += 1;
                pair
            }
        };

        // classify
        let kind = match lexeme.as_str() {
            "let"    => TokenType::Let,
            "const"  => TokenType::Const,
            "fn"     => TokenType::Fn,
            "if"     => TokenType::If,
            "else"   => TokenType::Else,
            "while"  => TokenType::While,
            "return" => TokenType::Return,
            "true"   => TokenType::True,
            "false"  => TokenType::False,
            "("      => TokenType::LParen,
            ")"      => TokenType::RParen,
            "{"      => TokenType::LBrace,
            "}"      => TokenType::RBrace,
            ";"      => TokenType::Semicolon,
            ","      => TokenType::Comma,
            "+"      => TokenType::Plus,
            "-"      => TokenType::Minus,
            "*"      => TokenType::Star,
            "/"      => TokenType::Slash,
            "="      => TokenType::Equals,
            ">"      => TokenType::Greater,
            "<"      => TokenType::Less,
            "=="     => TokenType::EqualEqual,
            "!="     => TokenType::BangEqual,
            ">="     => TokenType::GreaterEqual,
            "<="     => TokenType::LessEqual,
            _ if lexeme.starts_with('"') && lexeme.ends_with('"') => TokenType::String,
            _ if lexeme.parse::<f64>().is_ok()                    => TokenType::Number,
            _ if is_identifier(&lexeme)                          => TokenType::Identifier,
            _                                                    => TokenType::Unknown,
        };

        // push with full positional info
        tokens.push(Token::new(kind, &lexeme, line, start_col));
    }

    // final EOF token
    tokens.push(Token::new(TokenType::Eof, "", line, col));
    Ok(tokens)
}

/// Identifiers: start with letter or `_`, then alphanumeric or `_`
fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => (),
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}
