// compiler/src/parser/expr.rs

use shared::tokenizer::{tokenize, Token};
use shared::ast::{BinaryOp, Expr, Stmt, UnaryOp};
use crate::parser::error::ParseError;

/// A simple recursive-descent parser over a token stream.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(raw_tokens: Vec<Token>) -> Self {
        let tokens = normalize_tokens(raw_tokens);
        Parser { tokens, pos: 0 }
    }

    pub fn from_source(source: &str) -> Result<Self, ParseError> {
        let raw = tokenize(source)?;
        Ok(Parser::new(raw))
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        eprintln!("DEBUG> [parser] entering parse()");
        let mut stmts = Vec::new();
        while self.current().is_some() {
            let lexeme = self.current().unwrap().lexeme.clone();
            let before = self.pos;
            let stmt = self.parse_statement()?;
            if self.pos == before {
                return Err(ParseError::UnexpectedToken(lexeme));
            }
            stmts.push(stmt);
        }
        eprintln!("DEBUG> [parser] parsed {} statements", stmts.len());
        for (i, stmt) in stmts.iter().enumerate() {
            eprintln!("DEBUG> stmt[{}] = {:?}", i, stmt);
        }
        Ok(stmts)
    }

    pub fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        eprintln!("DEBUG> [parser] entering parse_statement()");
        if self.current_is("let") {
            self.advance();
            let name = match self.current() {
                Some(tok) if is_identifier(&tok.lexeme) => {
                    let s = tok.lexeme.clone();
                    self.advance();
                    s
                }
                Some(_) => return Err(ParseError::ExpectedIdentifier),
                None => return Err(ParseError::UnexpectedEOF),
            };
            self.expect("=")?;
            let value = self.parse_expression()?;
            self.expect(";")?;
            Ok(Stmt::Let(name, value))
        } else if self.current_is("if") {
            self.advance();
            let cond = self.parse_expression()?;
            let then_branch = self.parse_block()?;
            let else_branch = if self.current_is("else") {
                self.advance();
                Some(self.parse_block()?)
            } else {
                None
            };
            Ok(Stmt::If { cond, then_branch, else_branch: else_branch.expect("REASON") })
        } else if self.current_is("while") {
            self.advance();
            let cond = self.parse_expression()?;
            let body = self.parse_block()?;
            Ok(Stmt::While { cond, body })
        } else if self.current_is("{") {
            Ok(Stmt::Block(self.parse_block()?))
        } else {
            let start = self.pos;
            let expr = self.parse_expression()?;
            if self.pos == start {
                return Err(ParseError::UnexpectedToken(
                    self.current()
                        .map(|t| t.lexeme.clone())
                        .unwrap_or_else(|| "<EOF>".into())
                ));
            }
            self.expect(";")?;
            Ok(Stmt::Expr(expr))
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.expect("{")?;
        let mut stmts = Vec::new();
        while !self.current_is("}") {
            stmts.push(self.parse_statement()?);
        }
        self.expect("}")?;
        Ok(stmts)
    }

    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_logic_or()
    }

    fn parse_logic_or(&mut self) -> Result<Expr, ParseError> {
        let mut e = self.parse_logic_and()?;
        while self.current_is("or") {
            self.advance();
            let r = self.parse_logic_and()?;
            e = Expr::Binary { left: Box::new(e), op: BinaryOp::Or, right: Box::new(r) };
        }
        Ok(e)
    }

    fn parse_logic_and(&mut self) -> Result<Expr, ParseError> {
        let mut e = self.parse_equality()?;
        while self.current_is("and") {
            self.advance();
            let r = self.parse_equality()?;
            e = Expr::Binary { left: Box::new(e), op: BinaryOp::And, right: Box::new(r) };
        }
        Ok(e)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut e = self.parse_comparison()?;
        while self.current_is("==") || self.current_is("!=") {
            let op = if self.current_is("==") { BinaryOp::EqualEqual } else { BinaryOp::NotEqual };
            self.advance();
            let r = self.parse_comparison()?;
            e = Expr::Binary { left: Box::new(e), op, right: Box::new(r) };
        }
        Ok(e)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut e = self.parse_term()?;
        while ["<", "<=", ">", ">="].iter().any(|&s| self.current_is(s)) {
            let op = match self.current().unwrap().lexeme.as_str() {
                "<" => BinaryOp::Less,
                "<=" => BinaryOp::LessEqual,
                ">" => BinaryOp::Greater,
                _ => BinaryOp::GreaterEqual,
            };
            self.advance();
            let r = self.parse_term()?;
            e = Expr::Binary { left: Box::new(e), op, right: Box::new(r) };
        }
        Ok(e)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut e = self.parse_factor()?;
        while self.current_is("+") || self.current_is("-") {
            let op = if self.current_is("+") { BinaryOp::Add } else { BinaryOp::Sub };
            self.advance();
            let r = self.parse_factor()?;
            e = Expr::Binary { left: Box::new(e), op, right: Box::new(r) };
        }
        Ok(e)
    }

    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let mut e = self.parse_unary()?;
        while self.current_is("*") || self.current_is("/") {
            let op = if self.current_is("*") { BinaryOp::Mul } else { BinaryOp::Div };
            self.advance();
            let r = self.parse_unary()?;
            e = Expr::Binary { left: Box::new(e), op, right: Box::new(r) };
        }
        Ok(e)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.current_is("-") {
            self.advance();
            let sub = self.parse_unary()?;
            return Ok(Expr::Unary { op: UnaryOp::Negate, expr: Box::new(sub) });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        if self.current_is("(") {
            self.advance();
            let e = self.parse_expression()?;
            self.expect(")")?;
            return Ok(e);
        }
        if let Some(tok) = self.current() {
            let lex = &tok.lexeme;
            if lex == "[" {
                self.advance();
                let mut elems = Vec::new();
                if !self.current_is("]") {
                    loop {
                        elems.push(self.parse_expression()?);
                        if self.current_is(",") { self.advance(); continue; }
                        break;
                    }
                }
                self.expect("]")?;
                return Ok(Expr::ListLiteral(elems));
            }
            if let Ok(n) = lex.parse::<f64>() {
                self.advance();
                return Ok(Expr::LiteralNumber(n));
            }
            if lex == "true" || lex == "false" {
                let b = lex == "true";
                self.advance();
                return Ok(Expr::LiteralBool(b));
            }
            if lex.starts_with('"') && lex.ends_with('"') && lex.len() >= 2 {
                let content = lex[1..lex.len()-1].to_string();
                self.advance();
                return Ok(Expr::LiteralString(content));
            }
            if is_identifier(lex) {
                let name = lex.clone();
                self.advance();
                if self.current_is("(") {
                    self.advance();
                    let mut args = Vec::new();
                    if !self.current_is(")") {
                        loop {
                            args.push(self.parse_expression()?);
                            if self.current_is(",") { self.advance(); continue; }
                            break;
                        }
                    }
                    self.expect(")")?;
                    return Ok(Expr::Call(name, args));
                }
                return Ok(Expr::Variable(name));
            }
        }
        Err(ParseError::UnexpectedToken(
            self.current().map(|t| t.lexeme.clone()).unwrap_or_else(|| "<EOF>".into())
        ))
    }

    pub(crate) fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub(crate) fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    pub(crate) fn current_is(&self, s: &str) -> bool {
        self.current().is_some_and(|t| t.lexeme == s)
    }

    pub(crate) fn expect(&mut self, s: &str) -> Result<(), ParseError> {
        if self.current_is(s) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::ExpectedToken(s.into()))
        }
    }
}

pub fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => (),
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

fn normalize_tokens(raw: Vec<Token>) -> Vec<Token> {
    let two_char = ["==", "!=", "<=", ">="];
    let single: &[char] = &['(', ')', '{', '}', '[', ']', ';', ',', '+', '-', '*', '/',];
    let mut out = Vec::new();

    for tok in raw {
        let chars: Vec<char> = tok.lexeme.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if i + 1 < chars.len() {
                let pair = &tok.lexeme[i..i + 2];
                if two_char.contains(&pair) {
                    out.push(Token { lexeme: pair.to_string(), line: tok.line, col: tok.col + i });
                    i += 2;
                    continue;
                }
            }
            let c = chars[i];
            if single.contains(&c) {
                out.push(Token { lexeme: c.to_string(), line: tok.line, col: tok.col + i });
                i += 1;
            } else {
                let start = i;
                while i < chars.len()
                    && !single.contains(&chars[i])
                    && !(i + 1 < chars.len() && two_char.iter().any(|op| op.starts_with(chars[i])))
                {
                    i += 1;
                }
                let chunk: String = chars[start..i].iter().collect();
                out.push(Token { lexeme: chunk, line: tok.line, col: tok.col + start });
            }
        }
    }

    out
}
