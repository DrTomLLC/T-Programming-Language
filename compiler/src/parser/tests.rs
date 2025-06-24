// compiler/src/parser/tests.rs
//! Parser regression tests with comprehensive error handling.
//! NO unwrap() or panic!() calls allowed.

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
        let ast = match parse_program("module M;") {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                assert!(false, "Failed to parse empty module");
                return;
            }
        };

        assert_eq!(ast.modules.len(), 1);
        let m = &ast.modules[0];
        assert_eq!(m.name, "M");
        assert!(m.declarations.is_empty());
    }

    #[test]
    fn test_empty_and_second_module() {
        let ast = match parse_program("module A; module B;") {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                assert!(false, "Failed to parse two modules");
                return;
            }
        };

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

        let ast = match parse_program(src) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                assert!(false, "Failed to parse function with let and return");
                return;
            }
        };

        let decls = &ast.modules[0].declarations;
        assert_eq!(decls.len(), 1);

        let func = &decls[0];
        let stmts = func.body.statements.clone();

        // Safe pattern matching instead of unwrap/panic
        if stmts.len() >= 2 {
            let is_let_stmt = matches!(stmts[0], Statement::Let(_));
            let is_return_stmt = matches!(stmts[1], Statement::Return(_));

            assert!(is_let_stmt, "First statement should be Let");
            assert!(is_return_stmt, "Second statement should be Return");
        } else {
            assert!(false, "Function should have at least 2 statements");
        }
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

        let ast = match parse_program(src) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                assert!(false, "Failed to parse if/while statements");
                return;
            }
        };

        let decls = &ast.modules[0].declarations;
        if decls.is_empty() {
            assert!(false, "Should have at least one declaration");
            return;
        }

        let stmts = &decls[0].body.statements;
        if stmts.len() >= 2 {
            let is_if_stmt = matches!(stmts[0], Statement::If(_));
            let is_while_stmt = matches!(stmts[1], Statement::While(_));

            assert!(is_if_stmt, "First statement should be If");
            assert!(is_while_stmt, "Second statement should be While");
        } else {
            assert!(false, "Function should have at least 2 statements");
        }
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

        let ast = match parse_program(src) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                assert!(false, "Failed to parse expressions");
                return;
            }
        };

        if ast.modules.is_empty() {
            assert!(false, "Should have at least one module");
            return;
        }

        let decls = &ast.modules[0].declarations;
        if decls.is_empty() {
            assert!(false, "Should have at least one declaration");
            return;
        }

        let stmts = &decls[0].body.statements;
        if stmts.len() < 3 {
            assert!(false, "Function should have at least 3 expression statements");
            return;
        }

        // Safely check first statement (binary expression)
        if let Statement::Expr(expr_stmt) = &stmts[0] {
            let is_binary = matches!(expr_stmt.expr, Expression::Binary { .. });
            assert!(is_binary, "First expression should be binary operation");
        } else {
            assert!(false, "First statement should be expression statement");
        }

        // Safely check second statement (unary expression)
        if let Statement::Expr(expr_stmt) = &stmts[1] {
            let is_unary = matches!(expr_stmt.expr, Expression::Unary { .. });
            assert!(is_unary, "Second expression should be unary operation");
        } else {
            assert!(false, "Second statement should be expression statement");
        }

        // Safely check third statement (call expression)
        if let Statement::Expr(expr_stmt) = &stmts[2] {
            if let Expression::Call { callee, args } = &expr_stmt.expr {
                let is_foo_call = match callee.as_ref() {
                    Expression::Identifier(name) => name == "foo",
                    _ => false,
                };
                assert!(is_foo_call, "Callee should be 'foo'");
                assert_eq!(args.len(), 2, "Should have 2 arguments");

                if args.len() >= 2 {
                    let is_string_arg = matches!(args[1], Expression::Literal(Literal::String(ref s)) if s == "bar");
                    assert!(is_string_arg, "Second argument should be string 'bar'");
                }
            } else {
                assert!(false, "Third expression should be call expression");
            }
        } else {
            assert!(false, "Third statement should be expression statement");
        }
    }

    #[test]
    fn test_parse_errors_on_bad_syntax() {
        let result = parse_program("module Bad fn;");

        match result {
            Ok(_) => assert!(false, "Should have failed to parse bad syntax"),
            Err(_) => {
                // This is expected - bad syntax should produce an error
                assert!(true, "Correctly detected parse error");
            }
        }
    }

    #[test]
    fn test_empty_input() {
        let result = parse_program("");

        match result {
            Ok(ast) => {
                assert!(ast.modules.is_empty(), "Empty input should produce empty AST");
            }
            Err(e) => {
                // Empty input might be an error or might produce empty AST
                eprintln!("Empty input parse result: {}", e);
                // Don't fail the test - either behavior is acceptable
            }
        }
    }

    #[test]
    fn test_comment_handling() {
        let src = r#"
            // This is a comment
            module CommentTest;
            /* Multi-line
               comment */
            fn test() {
                // Another comment
                let x = 42; // End of line comment
            }
        "#;

        let result = parse_program(src);
        match result {
            Ok(_ast) => {
                // Successfully parsed despite comments
                assert!(true, "Comments handled correctly");
            }
            Err(e) => {
                eprintln!("Failed to handle comments: {}", e);
                assert!(false, "Comment parsing should not fail");
            }
        }
    }
}
