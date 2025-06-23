// compiler/src/parser/expressions.rs

use crate::parser::{Parser, Result};
use crate::lexer::{TokenKind, Keyword};
use crate::ast::{Expression, Literal, UnaryOp, BinaryOp};

/// Parse an expression with operator precedence.
pub fn parse_expression(parser: &mut Parser) -> Result<Expression> {
    parse_assignment(parser)
}

fn parse_assignment(parser: &mut Parser) -> Result<Expression> {
    let lhs = parse_binary(parser, 0)?;
    if parser.peek(TokenKind::Equals) {
        parser.bump();
        let rhs = parse_assignment(parser)?;
        if let Expression::Identifier(name) = lhs {
            Ok(Expression::Assignment {
                target: name,
                value: Box::new(rhs),
            })
        } else {
            Err(parser.error("Invalid assignment target"))
        }
    } else {
        Ok(lhs)
    }
}

fn parse_binary(parser: &mut Parser, min_prec: u8) -> Result<Expression> {
    let mut expr = parse_unary(parser)?;
    while let Some((op, prec, right_assoc)) = parser.peek_binary_op() {
        if prec < min_prec {
            break;
        }
        parser.bump();
        let next_min = if right_assoc { prec } else { prec + 1 };
        let rhs = parse_binary(parser, next_min)?;
        expr = Expression::Binary {
            left: Box::new(expr),
            op,
            right: Box::new(rhs),
        };
    }
    Ok(expr)
}

fn parse_unary(parser: &mut Parser) -> Result<Expression> {
    if parser.peek(TokenKind::Minus) {
        parser.bump();
        let expr = parse_unary(parser)?;
        Ok(Expression::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(expr),
        })
    } else if parser.peek(TokenKind::Bang) {
        parser.bump();
        let expr = parse_unary(parser)?;
        Ok(Expression::Unary {
            op: UnaryOp::Not,
            expr: Box::new(expr),
        })
    } else {
        parse_call(parser)
    }
}

fn parse_call(parser: &mut Parser) -> Result<Expression> {
    let mut expr = parse_primary(parser)?;
    loop {
        if parser.peek(TokenKind::OpenParen) {
            parser.bump();
            let mut args = Vec::new();
            if !parser.peek(TokenKind::CloseParen) {
                loop {
                    args.push(parse_expression(parser)?);
                    if parser.peek(TokenKind::Comma) {
                        parser.bump();
                    } else {
                        break;
                    }
                }
            }
            parser.expect(TokenKind::CloseParen)?;
            expr = Expression::Call {
                callee: Box::new(expr),
                args,
            };
        } else {
            break;
        }
    }
    Ok(expr)
}

fn parse_primary(parser: &mut Parser) -> Result<Expression> {
    if parser.peek(TokenKind::OpenParen) {
        parser.bump();
        let expr = parse_expression(parser)?;
        parser.expect(TokenKind::CloseParen)?;
        Ok(Expression::Grouping(Box::new(expr)))
    } else if parser.peek_keyword(Keyword::True) {
        parser.bump();
        Ok(Expression::Literal(Literal::Boolean(true)))
    } else if parser.peek_keyword(Keyword::False) {
        parser.bump();
        Ok(Expression::Literal(Literal::Boolean(false)))
    } else if parser.peek(TokenKind::Number) {
        let tok = parser.bump();
        // TODO: distinguish integer vs float literals here
        let value = tok.lexeme.parse().map_err(|_| parser.error("Invalid number literal"))?;
        Ok(Expression::Literal(Literal::Integer(value)))
    } else if parser.peek(TokenKind::String) {
        let tok = parser.bump();
        Ok(Expression::Literal(Literal::String(tok.lexeme.clone())))
    } else if parser.peek(TokenKind::Identifier) {
        let name = parser.expect_identifier()?;
        Ok(Expression::Identifier(name))
    } else {
        Err(parser.error("Expected expression"))
    }
}
