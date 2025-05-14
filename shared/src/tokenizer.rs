// shared/src/tokenizer.rs
//! Shared tokenizer for T-Lang — with debug instrumentation.

use thiserror::Error;

/// A single token, carrying its lexeme and position.
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub lexeme: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Error, PartialEq)]
pub enum LexError {
    #[error("Unexpected character `{0}` at line {1}, col {2}")]
    UnexpectedChar(char, usize, usize),
    #[error("Unterminated string literal starting at line {0}, col {1}")]
    UnterminatedString(usize, usize),
}

/// Turn a source string into a `Vec<Token>`, or return a `LexError`.
pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> {
    eprintln!("DEBUG> tokenize: starting ({} chars)", source.len());
    let mut tokens = Vec::new();
    let mut line = 1;
    let mut col = 1;
    let mut chars = source.chars().peekable();

    while let Some(&ch) = chars.peek() {
        eprintln!("DEBUG> tokenize: at line {}, col {}, char {:?}", line, col, ch);
        match ch {
            // **Skip ALL ASCII whitespace**, including CR
            ' ' | '\t' | '\r' | '\n' => {
                chars.next();
                if ch == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
            }

            // Single-character punctuation
            c @ ('(' | ')' | '{' | '}' | '[' | ']' | ';' | ',' | '+' | '-' | '*' | '/') => {
                tokens.push(Token {
                    lexeme: c.to_string(),
                    line,
                    col,
                });
                eprintln!("DEBUG> tokenize: punct token {:?}", c);
                chars.next();
                col += 1;
            }

            // String literals
            '"' => {
                let start_col = col;
                chars.next(); // consume opening "
                col += 1;
                let mut s = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '"' {
                        chars.next();
                        col += 1;
                        break;
                    }
                    s.push(next_ch);
                    chars.next();
                    col += 1;
                }
                if chars.peek().is_none() && !s.ends_with('"') {
                    return Err(LexError::UnterminatedString(line, start_col));
                }
                let lit = format!("\"{}\"", s);
                tokens.push(Token { lexeme: lit, line, col: start_col });
                eprintln!("DEBUG> tokenize: string literal {:?}", s);
            }

            // Identifiers or keywords
            c if c.is_alphabetic() || c == '_' => {
                let start_col = col;
                let mut ident = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_alphanumeric() || next_ch == '_' {
                        ident.push(next_ch);
                        chars.next();
                        col += 1;
                    } else {
                        break;
                    }
                }
                tokens.push(Token { lexeme: ident.clone(), line, col: start_col });
                eprintln!("DEBUG> tokenize: identifier {:?}", ident);
            }

            // Numbers
            c if c.is_ascii_digit() => {
                let start_col = col;
                let mut num = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_ascii_digit() || next_ch == '.' {
                        num.push(next_ch);
                        chars.next();
                        col += 1;
                    } else {
                        break;
                    }
                }
                tokens.push(Token { lexeme: num.clone(), line, col: start_col });
                eprintln!("DEBUG> tokenize: number {:?}", num);
            }

            // Two-char operators
            '!' | '=' | '<' | '>' if {
                let mut look = chars.clone();
                look.next();
                matches!(look.peek(), Some(&'='))
            } => {
                let first = chars.next().unwrap();
                let second = chars.next().unwrap();
                let op = format!("{}{}", first, second);
                tokens.push(Token { lexeme: op.clone(), line, col });
                eprintln!("DEBUG> tokenize: two-char op {:?}", op);
                col += 2;
            }

            // Anything else is an error
            c => {
                return Err(LexError::UnexpectedChar(c, line, col));
            }
        }
    }

    eprintln!("DEBUG> tokenize: finished with {} tokens", tokens.len());
    Ok(tokens)
}
