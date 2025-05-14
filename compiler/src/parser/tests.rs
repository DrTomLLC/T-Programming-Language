// compiler/src/parser/tests.rs

#[cfg(test)]
mod tests {
    use shared::ast::{BinaryOp, Expr, Stmt, UnaryOp};
    use shared::tokenizer::tokenize;
    use crate::parser::error::ParseError;
    use crate::parser::Parser;

    #[test]
    fn parse_literal_number() -> Result<(), ParseError> {
        let tokens = tokenize("123")?;
        let expr = Parser::new(tokens).parse_expression()?;
        assert_eq!(expr, Expr::LiteralNumber(123.0));
        Ok(())
    }

    #[test]
    fn parse_unary_and_grouping() -> Result<(), ParseError> {
        let tokens = tokenize("-(1 + 2)")?;
        let expr = Parser::new(tokens).parse_expression()?;

        // expect Expr::Unary { op: Negate, expr: Box::new(Expr::Grouping(Box::new(inner))) }
        if let Expr::Unary { op: UnaryOp::Negate, expr } = expr {
            if let Expr::Grouping(inner_box) = *expr {
                if let Expr::Binary { left, op: BinaryOp::Add, right } = *inner_box {
                    assert_eq!(*left, Expr::LiteralNumber(1.0));
                    assert_eq!(*right, Expr::LiteralNumber(2.0));
                    return Ok(());
                }
            }
        }
        panic!("parse_unary_and_grouping did not produce the expected AST");
    }

    #[test]
    fn parse_binary_precedence() -> Result<(), ParseError> {
        let tokens = tokenize("1 + 2 * 3")?;
        let expr = Parser::new(tokens).parse_expression()?;

        // must be 1 + (2 * 3)
        if let Expr::Binary { left, op: BinaryOp::Add, right } = expr {
            assert_eq!(*left, Expr::LiteralNumber(1.0));
            if let Expr::Binary { left: l2, op: BinaryOp::Mul, right: r2 } = *right {
                assert_eq!(*l2, Expr::LiteralNumber(2.0));
                assert_eq!(*r2, Expr::LiteralNumber(3.0));
                return Ok(());
            }
        }
        panic!("parse_binary_precedence did not produce the expected AST");
    }

    #[test]
    fn parse_statement_let_and_expr() -> Result<(), ParseError> {
        let tokens = tokenize("let x = 5;")?;
        let stmt = Parser::new(tokens).parse_statement()?;
        assert_eq!(stmt, Stmt::Let("x".into(), Expr::LiteralNumber(5.0)));

        let tokens = tokenize("x;")?;
        let stmt2 = Parser::new(tokens).parse_statement()?;
        assert_eq!(stmt2, Stmt::Expr(Expr::Variable("x".into())));

        Ok(())
    }
}
