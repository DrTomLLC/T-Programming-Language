//! Shared utilities for T-Lang: AST definitions, token types, and tokenizer.

pub mod ast;
pub mod token;
pub mod tokenizer;
pub use token::{Token as RawToken, TokenType};
pub use ast::{Expr, Stmt, BinaryOp, UnaryOp, Span};
pub use tokenizer::tokenize;
pub use ast::AST;
