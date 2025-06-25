// shared/src/ast/program.rs
//! Program definition for T-Lang AST.

use miette::SourceSpan;
use super::Item;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: SourceSpan,
}