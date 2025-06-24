// File: compiler/src/parser/mod.rs - COMPLETE REWRITE
// -----------------------------------------------------------------------------

//! Complete parser implementation for T-Lang.
//! Integrates all parser modules into a cohesive parsing system.

pub mod entry;
pub mod statements;
pub mod expressions;
pub mod declarations;
pub mod modules;
pub mod types;
pub mod patterns;
pub mod tests;

use shared::{Program, tokenize, Result, TlError, Token};
use crate::parser::entry::Parser;

/// Parse source code into a Program AST - COMPLETE IMPLEMENTATION
pub fn parse_source(source: &str) -> Result<Program> {
    // 1. Tokenize the source
    let tokens = tokenize(source)?;

    // 2. Parse tokens into AST
    let mut parser = Parser::new(tokens);
    let statements = parser.parse_all()?;

    // 3. Convert statements to program items
    let items = statements.into_iter().filter_map(|stmt| {
        match stmt.kind {
            shared::ast::StmtKind::Item(item) => Some(item),
            // For non-item statements, wrap in a main function
            _ => {
                Some(shared::ast::Item {
                    kind: shared::ast::ItemKind::Function {
                        name: "main".to_string(),
                        params: Vec::new(),
                        return_type: None,
                        body: shared::ast::Block {
                            statements: vec![stmt],
                            expr: None,
                            span: stmt.span,
                        },
                        safety: shared::ast::SafetyLevel::Safe,
                    },
                    attrs: Vec::new(),
                    vis: shared::ast::Visibility::Public,
                    span: stmt.span,
                })
            }
        }
    }).collect();

    Ok(Program {
        items,
        span: miette::SourceSpan::new(0.into(), source.len()),
    })
}

/// Parse a single expression from source
pub fn parse_expression(source: &str) -> Result<shared::ast::Expr> {
    let tokens = tokenize(source)?;
    let mut parser = Parser::new(tokens);
    parser.parse_expression()
}

/// Parse a type annotation from source
pub fn parse_type(source: &str) -> Result<shared::ast::Type> {
    let tokens = tokenize(source)?;
    let mut parser = Parser::new(tokens);
    parser.parse_type()
}

// Re-export key parser types
pub use entry::{Parser, ParseError};
pub use statements::parse_statement;
pub use expressions::parse_expression as parse_expr_internal;
pub use declarations::parse_declaration;
pub use modules::parse_module;
