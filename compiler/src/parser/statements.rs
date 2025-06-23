use crate::parser::{Parser, Result};
use crate::lexer::{TokenKind, Keyword};
use crate::ast::{Statement, LetStmt, ReturnStmt, IfStmt, WhileStmt, ExprStmt, Block};
use crate::parser::expressions::parse_expression;

/// Parse a single statement: let, return, if, while, or expression statement.
pub fn parse_statement(parser: &mut Parser) -> Result<Statement> {
    if parser.peek_keyword(Keyword::Let) {
        let stmt = parse_let_stmt(parser)?;
        Ok(Statement::Let(stmt))
    } else if parser.peek_keyword(Keyword::Return) {
        let stmt = parse_return_stmt(parser)?;
        Ok(Statement::Return(stmt))
    } else if parser.peek_keyword(Keyword::If) {
        let stmt = parse_if_stmt(parser)?;
        Ok(Statement::If(stmt))
    } else if parser.peek_keyword(Keyword::While) {
        let stmt = parse_while_stmt(parser)?;
        Ok(Statement::While(stmt))
    } else {
        let stmt = parse_expr_stmt(parser)?;
        Ok(Statement::Expr(stmt))
    }
}

/// Parse a block `{ ... }` of statements.
pub fn parse_block(parser: &mut Parser) -> Result<Block> {
    parser.expect(TokenKind::OpenBrace)?;
    let mut statements = Vec::new();
    while !parser.peek(TokenKind::CloseBrace) {
        statements.push(parse_statement(parser)?);
    }
    parser.expect(TokenKind::CloseBrace)?;
    Ok(Block { statements })
}

fn parse_let_stmt(parser: &mut Parser) -> Result<LetStmt> {
    parser.expect_keyword(Keyword::Let)?;
    let name = parser.expect_identifier()?;
    let ty = if parser.peek(TokenKind::Colon) {
        parser.bump();
        Some(parser.parse_type()?)
    } else {
        None
    };
    let initializer = if parser.peek(TokenKind::Equal) {
        parser.bump();
        Some(parse_expression(parser)?)
    } else {
        None
    };
    parser.expect(TokenKind::Semicolon)?;
    Ok(LetStmt { name, ty, initializer })
}

fn parse_return_stmt(parser: &mut Parser) -> Result<ReturnStmt> {
    parser.expect_keyword(Keyword::Return)?;
    let expr = if !parser.peek(TokenKind::Semicolon) {
        Some(parse_expression(parser)?)
    } else {
        None
    };
    parser.expect(TokenKind::Semicolon)?;
    Ok(ReturnStmt { expr })
}

fn parse_if_stmt(parser: &mut Parser) -> Result<IfStmt> {
    parser.expect_keyword(Keyword::If)?;
    let condition = parse_expression(parser)?;
    let then_branch = parse_block(parser)?;
    let else_branch = if parser.peek_keyword(Keyword::Else) {
        parser.bump();
        if parser.peek_keyword(Keyword::If) {
            let nested = parse_if_stmt(parser)?;
            Some(Box::new(Statement::If(nested)))
        } else {
            let block = parse_block(parser)?;
            Some(Box::new(Statement::Expr(ExprStmt { expr: crate::ast::Expression::Block(block) })))
        }
    } else {
        None
    };
    Ok(IfStmt { condition, then_branch, else_branch })
}

fn parse_while_stmt(parser: &mut Parser) -> Result<WhileStmt> {
    parser.expect_keyword(Keyword::While)?;
    let condition = parse_expression(parser)?;
    let body = parse_block(parser)?;
    Ok(WhileStmt { condition, body })
}

fn parse_expr_stmt(parser: &mut Parser) -> Result<ExprStmt> {
    let expr = parse_expression(parser)?;
    parser.expect(TokenKind::Semicolon)?;
    Ok(ExprStmt { expr })
}
