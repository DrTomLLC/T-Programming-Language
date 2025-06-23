// File: compiler/src/sema.rs
//
// A production-ready semantic checker for T‑Lang over the in-crate AST.
// Performs exhaustive matching on every Statement and Expression variant,
// returning detailed TlError on unexpected constructs to ensure safety
// and future-proofing. As AST expands, add new arms accordingly.

use crate::ast::{Program, Statement, Expression};
use errors::TlError;

/// Entry point: performs semantic checks on the entire program.
pub fn check_program(program: &Program) -> Result<(), TlError> {
    check_statements(&program.stmts)
}

/// Entry point: check a slice of top-level statements.
pub fn check_statements(stmts: &[Statement]) -> Result<(), TlError> {
    for (idx, stmt) in stmts.iter().enumerate() {
        check_statement(stmt).map_err(|e| {
            TlError::semantic(
                "<input>",
                "",
                (0, 0),
                format!("Error in statement[{}]: {:?}: {}", idx, stmt, e),
            )
        })?;
    }
    Ok(())
}

fn check_statement(stmt: &Statement) -> Result<(), TlError> {
    match stmt {
        Statement::Print(expr) => check_expression(expr),
        // If new Statement variants are added, compilation will warn here
    }
}

fn check_expression(expr: &Expression) -> Result<(), TlError> {
    match expr {
        Expression::StringLiteral(s) => {
            if s.is_empty() {
                Err(TlError::semantic(
                    "<input>",
                    "",
                    (0, 0),
                    "StringLiteral cannot be empty",
                ))
            } else {
                Ok(())
            }
        }
        // If new Expression variants are added, compilation will warn here
    }
}
