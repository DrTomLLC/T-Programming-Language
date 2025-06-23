// compiler/src/parser/utils.rs

use shared::token::Token;
use errors::CompileError;

/// Valid identifiers: start with a letter or `_`, then alphanumeric or `_`.
pub fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Split multi‑char ops and punctuation, preserving string literals.
/// Works on a vector of raw `Token`s by their lexemes.
pub fn normalize_tokens(raw: Vec<Token>) -> Result<Vec<Token>, CompileError> {
    let two_char_ops = ["==", "!=", "<=", ">=", "||", "&&"];
    let single_chars: &[char] = &['(', ')', '{', '}', '[', ']', ';', '.', ',', '+', '-', '*', '/', '=',];

    let mut out = Vec::new();
    for tok in raw {
        let lex = tok.lexeme.clone();
        // preserve string literals
        if lex.starts_with('"') && lex.ends_with('"') && lex.len() >= 2 {
            out.push(tok);
            continue;
        }
        let mut i = 0;
        while i < lex.len() {
            // check two-char operators
            if i + 1 < lex.len() {
                let pair = &lex[i..i + 2];
                if two_char_ops.contains(&pair) {
                    let new_tok = Token::new(tok.kind.clone(), pair, tok.line);
                    out.push(new_tok);
                    i += 2;
                    continue;
                }
            }
            // single char tokens
            let c = lex.chars().nth(i)
                .ok_or_else(|| CompileError::Parse { pos: i, msg: "Invalid lexeme index".to_string() })?;
            if single_chars.contains(&c) {
                let s = c.to_string();
                let new_tok = Token::new(tok.kind.clone(), &s, tok.line);
                out.push(new_tok);
                i += 1;
            } else {
                // collect chunk
                let start = i;
                while i < lex.len() {
                    let cc = lex.chars().nth(i).unwrap();
                    if single_chars.contains(&cc)
                        || (i + 1 < lex.len() && two_char_ops.iter().any(|&op| op == &lex[i..i + 2]))
                    {
                        break;
                    }
                    i += 1;
                }
                let chunk = &lex[start..i];
                let new_tok = Token::new(tok.kind.clone(), chunk, tok.line);
                out.push(new_tok);
            }
        }
    }
    Ok(out)
}
