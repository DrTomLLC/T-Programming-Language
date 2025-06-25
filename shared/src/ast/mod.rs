// shared/src/ast/mod.rs
//! Abstract Syntax Tree definitions for T-Lang.

pub mod expr;
pub mod stmt;
pub mod item;
pub mod pattern;
pub mod types;
pub mod literal;
pub mod visitor;
pub mod program;
mod literal;
mod pattern;

// Re-export all AST types
pub use expr::*;
pub use stmt::*;
pub use item::*;
pub use pattern::*;
pub use types::*;
pub use literal::*;
pub use visitor::*;
pub use program::*;

// Re-export span traits
pub use crate::span::HasSpan;