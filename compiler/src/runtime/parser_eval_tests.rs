// compiler/src/runtime/parser_eval_tests.rs

//! Integration tests: parse then evaluate snippets.

use shared::tokenizer::tokenize;
use shared::ast::Stmt;
use crate::parser::Parser;
use crate::runtime::env::Environment;
use crate::runtime::error::RuntimeError;
use crate::runtime::value::Value;
use crate::runtime::eval::Interpreter;

#[cfg(test)]
mod parser_eval_tests {
    use super::*;

    #[test]
    fn if_expr_evaluates() -> Result<(), RuntimeError> {
        let tokens = tokenize("if (1 < 2) { 3; } else { 4; };")
            .map_err(|e| RuntimeError::LexError(e.to_string()))?;
        let mut p = Parser::new(tokens);
        let stmts = p.parse() .map_err(|e| RuntimeError::ParseError(e.to_string()))?;
        let mut env = Environment::new();
        let mut last = Value::Unit;
        for stmt in stmts {
            last = Evaluator::new(&mut env).eval_stmt(stmt)?;
        }
        assert_eq!(last, Value::Number(3.0));
        Ok(())
    }

    #[test]
    fn while_loop_increments() -> Result<(), RuntimeError> {
        let tokens = tokenize("let i = 0; while (i < 3) { i = i + 1; }; i;")
            .map_err(|e| RuntimeError::LexError(e.to_string()))?;
        let mut p = Parser::new(tokens);
        let stmts = p.parse().map_err(|e| RuntimeError::ParseError(e.to_string()))?;
        let mut env = Environment::new();
        let mut last = Value::Unit;
        for stmt in stmts {
            last = Evaluator::new(&mut env).eval_stmt(stmt)?;
        }
        assert_eq!(last, Value::Number(3.0));
        Ok(())
    }
}
#[test]
fn eval_string_literal_statement() {
    let mut env = Environment::new();
    let mut evaluator = Evaluator::new(&mut env);
    let stmts = Parser::from_source(r#""hello";"#)
        .unwrap()
        .parse()
        .unwrap();
    // Should be exactly one Expr-stmt returning the string "hello"
    match evaluator.eval_stmt(stmts.into_iter().next().unwrap()) {
        Ok(val) => assert_eq!(val.to_string(), r#""hello""#),
        Err(e)  => panic!("Expected string literal eval, got error: {}", e),
    }
}
