// compiler/src/parser/tests.rs

//! Parser regression tests to catch crashes and ensure basic grammar rules.
//! Run with `cargo test -- --nocapture parser::tests`.

use crate::lexer::Lexer;
use crate::parser::{Parser, Result};
use crate::ast::{AST, Module, Statement, Expression, Literal};

fn parse_program(src: &str) -> Result<AST> {
    let mut parser = Parser::new(Lexer::new(src));
    parser.parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_module() {
        let ast = parse_program("module M;").unwrap();
        assert_eq!(ast.modules.len(), 1);
        let m = &ast.modules[0];
        assert_eq!(m.name, "M");
        assert!(m.declarations.is_empty());
    }

    #[test]
    fn test_empty_and_second_module() {
        let ast = parse_program("module A; module B;").unwrap();
        assert_eq!(ast.modules.len(), 2);
        assert_eq!(ast.modules[0].name, "A");
        assert_eq!(ast.modules[1].name, "B");
    }

    #[test]
    fn test_let_and_return_statements() {
        let src = r#"
            module Test;
            fn f() {
                let x: i32 = 42;
                return x;
            }
        "#;
        let ast = parse_program(src).unwrap();
        let decls = &ast.modules[0].declarations;
        // Expect one function declaration
        assert_eq!(decls.len(), 1);
        // Inside function body, test that statements parse
        let func = &decls[0];
        let stmts = func.body.statements.clone();
        assert!(matches!(stmts[0], Statement::Let(_)));
        assert!(matches!(stmts[1], Statement::Return(_)));
    }

    #[test]
    fn test_if_else_and_while() {
        let src = r#"
            module Loop;
            fn loopit() {
                if true { } else { }
                while false { }
            }
        "#;
        let ast = parse_program(src).unwrap();
        let decls = &ast.modules[0].declarations;
        let stmts = &decls[0].body.statements;
        assert!(matches!(stmts[0], Statement::If(_)));
        assert!(matches!(stmts[1], Statement::While(_)));
    }

    #[test]
    fn test_expressions_basic() {
        let src = r#"
            module Expr;
            fn e() {
                1 + 2 * (3 - 4);
                !false;
                foo(5, "bar");
            }
        "#;
        let ast = parse_program(src).unwrap();
        let stmts = &ast.modules[0].declarations[0].body.statements;
        // First is an expression statement with a binary expression
        if let Statement::Expr(expr_stmt) = &stmts[0] {
            if let Expression::Binary { .. } = &expr_stmt.expr {
                // OK
            } else {
                panic!("Expected binary expression");
            }
        } else {
            panic!("Expected ExprStmt");
        }
        // Second: unary
        if let Statement::Expr(expr_stmt) = &stmts[1] {
            if let Expression::Unary { .. } = &expr_stmt.expr {
            } else {
                panic!("Expected unary expression");
            }
        } else {
            panic!("Expected ExprStmt");
        }
        // Third: call
        if let Statement::Expr(expr_stmt) = &stmts[2] {
            if let Expression::Call { callee, args } = &expr_stmt.expr {
                assert!(matches!(*callee.clone(), Expression::Identifier(ref name) if name == "foo"));
                assert_eq!(args.len(), 2);
                assert!(matches!(args[1], Expression::Literal(Literal::String(ref s)) if s == "bar"));
            } else {
                panic!("Expected call expression");
            }
        } else {
            panic!("Expected ExprStmt");
        }
    }

    #[test]
    fn test_parse_errors_on_bad_syntax() {
        let result = parse_program("module Bad fn;");
        assert!(result.is_err(), "Expected parse error on missing semicolon");
    }
}
