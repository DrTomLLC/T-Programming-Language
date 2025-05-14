// compiler/src/runtime/tests.rs

use super::eval::Interpreter;
use super::value::Value;
use super::error::RuntimeError;
use super::env::Environment;
use shared::ast::{Expr, Stmt, BinaryOp};


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_basic_arithmetic_and_let() {
        let mut env = Environment::new();
        let mut ev = Evaluator::new(&mut env);

        let expr = Expr::Block(vec![
            Stmt::Let("x".into(), Expr::LiteralNumber(5.0)),
            Stmt::Expr(Expr::Binary {
                left: Box::new(Expr::Variable("x".into())),
                op: BinaryOp::Add,
                right: Box::new(Expr::LiteralNumber(2.0)),
            }),
        ]);

        assert_eq!(ev.eval_expr(expr), Ok(Value::Number(7.0)));
    }

    #[test]
    fn test_eval_while_loop_sum() {
        let mut env = Environment::new();
        let mut ev = Evaluator::new(&mut env);

        let expr = Expr::Block(vec![
            Stmt::Let("sum".into(), Expr::LiteralNumber(0.0)),
            Stmt::While {
                cond: Expr::Binary {
                    left: Box::new(Expr::Variable("sum".into())),
                    op: BinaryOp::Less,
                    right: Box::new(Expr::LiteralNumber(5.0)),
                },
                body: vec![
                    Stmt::Assign(
                        "sum".into(),
                        Expr::Binary {
                            left: Box::new(Expr::Variable("sum".into())),
                            op: BinaryOp::Add,
                            right: Box::new(Expr::LiteralNumber(1.0)),
                        },
                    )
                ],
            },
            Stmt::Expr(Expr::Variable("sum".into())),
        ]);

        assert_eq!(ev.eval_expr(expr), Ok(Value::Number(5.0)));
    }

    #[test]
    fn test_block_shadowing() {
        let mut env = Environment::new();
        let mut ev = Evaluator::new(&mut env);

        let expr = Expr::Block(vec![
            Stmt::Let("x".into(), Expr::LiteralNumber(10.0)),
            Stmt::Expr(Expr::Block(vec![
                Stmt::Let(
                    "x".into(),
                    Expr::Binary {
                        left: Box::new(Expr::Variable("x".into())),
                        op: BinaryOp::Mul,
                        right: Box::new(Expr::LiteralNumber(2.0)),
                    },
                ),
                Stmt::Expr(Expr::Variable("x".into())),
            ])),
        ]);

        assert_eq!(ev.eval_expr(expr), Ok(Value::Number(20.0)));
    }

    #[test]
    fn test_comparisons_and_logical() {
        let mut env = Environment::new();
        let mut ev = Evaluator::new(&mut env);

        // numeric comparisons
        let lt = Expr::Binary {
            left: Box::new(Expr::LiteralNumber(3.0)),
            op: BinaryOp::Less,
            right: Box::new(Expr::LiteralNumber(4.0)),
        };
        assert_eq!(ev.eval_expr(lt), Ok(Value::Bool(true)));

        let eq = Expr::Binary {
            left: Box::new(Expr::LiteralNumber(5.0)),
            op: BinaryOp::EqualEqual,
            right: Box::new(Expr::LiteralNumber(5.0)),
        };
        assert_eq!(ev.eval_expr(eq), Ok(Value::Bool(true)));

        let ne = Expr::Binary {
            left: Box::new(Expr::LiteralNumber(5.0)),
            op: BinaryOp::NotEqual,
            right: Box::new(Expr::LiteralNumber(6.0)),
        };
        assert_eq!(ev.eval_expr(ne), Ok(Value::Bool(true)));

        let ge = Expr::Binary {
            left: Box::new(Expr::LiteralNumber(5.0)),
            op: BinaryOp::GreaterEqual,
            right: Box::new(Expr::LiteralNumber(5.0)),
        };
        assert_eq!(ev.eval_expr(ge), Ok(Value::Bool(true)));

        let le = Expr::Binary {
            left: Box::new(Expr::LiteralNumber(2.0)),
            op: BinaryOp::LessEqual,
            right: Box::new(Expr::LiteralNumber(3.0)),
        };
        assert_eq!(ev.eval_expr(le), Ok(Value::Bool(true)));

        // logical operators
        let and = Expr::Binary {
            left: Box::new(Expr::LiteralBool(true)),
            op: BinaryOp::And,
            right: Box::new(Expr::LiteralBool(false)),
        };
        assert_eq!(ev.eval_expr(and), Ok(Value::Bool(false)));

        let or = Expr::Binary {
            left: Box::new(Expr::LiteralBool(true)),
            op: BinaryOp::Or,
            right: Box::new(Expr::LiteralBool(false)),
        };
        assert_eq!(ev.eval_expr(or), Ok(Value::Bool(true)));
    }

    #[test]
    fn test_if_and_string_literals() {
        let mut env = Environment::new();
        let mut ev = Evaluator::new(&mut env);

        let expr = Expr::Block(vec![
            Stmt::Let("x".into(), Expr::LiteralNumber(10.0)),
            Stmt::Expr(Expr::If {
                condition: Box::new(Expr::Binary {
                    left: Box::new(Expr::Variable("x".into())),
                    op: BinaryOp::GreaterEqual,
                    right: Box::new(Expr::LiteralNumber(10.0)),
                }),
                then_branch: Box::new(Expr::LiteralString("done".into())),
                else_branch: Box::new(Expr::LiteralString("not done".into())),
            }),
        ]);

        assert_eq!(ev.eval_expr(expr), Ok(Value::Str("done".into())));
    }

    #[test]
    fn test_undefined_variable_error() {
        let mut env = Environment::new();
        let mut ev = Evaluator::new(&mut env);

        assert_eq!(
            ev.eval_expr(Expr::Variable("foo".into())),
            Err(RuntimeError::UndefinedVariable("foo".into()))
        );
    }

    #[test]
    fn test_wrong_arity_error() {
        let mut env = Environment::new();
        let mut ev = Evaluator::new(&mut env);

        // declare an empty function f()
        ev.eval_stmt(Stmt::Function("f".into(), vec![], vec![]))
            .unwrap();

        // now call it with one argument
        let wrong_call = Expr::Call("f".into(), vec![Expr::LiteralNumber(1.0)]);

        assert_eq!(
            ev.eval_expr(wrong_call),
            Err(RuntimeError::WrongArity("f".into(), 0, 1))
        );
    }
}
