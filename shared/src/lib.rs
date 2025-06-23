//! Shared utilities for T-Lang: AST definitions, token types, and tokenizer.

// File: shared/src/lib.rs

//! Central AST & key definitions for T-Lang, re-exported.

pub mod ast;
pub mod keys;

// Re-export everything from both modules:
pub use ast::*;
pub use keys::*;
pub mod token;
pub mod tokenizer;
pub mod tir;

pub use token::{Token as RawToken, TokenType};
pub use tokenizer::tokenize;
pub mod fs;

pub use keys::KeyModifiers;
// Re-export AST, Item, and Statement (alias of Item)
pub use ast::{AST, Item, Statement};