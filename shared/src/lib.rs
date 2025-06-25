// shared/src/lib.rs
//! Shared types and utilities for the T-Lang compiler.

// Re-export error types
pub use errors::{TlError, ErrorCode, Result, ErrorCollector};

// Declare modules
pub mod ast;
pub mod token;
pub mod tokenizer;
pub mod tir;
pub mod source_span;
pub mod span;

// Re-export commonly used types
pub use ast::*;
pub use token::*;
pub use tokenizer::tokenize;
pub use span::{Span, HasSpan};