// shared/src/ast/mod.rs
//! Abstract Syntax Tree definitions for T-Lang.
//!
//! This module provides a complete, type-safe representation of T-Lang programs.
//! Designed for safety-critical systems with explicit memory management,
//! comprehensive type information, and detailed source location tracking.

use miette::SourceSpan;
use serde::{Deserialize, Serialize};

pub mod types;
pub mod expr;
pub mod stmt;

// Re-export commonly used types for convenience
pub use expr::{Expr, ExprKind, Literal, Pattern, PatternKind, BinaryOp, UnaryOp, Block};
pub use stmt::{Stmt, StmtKind, Item, ItemKind, Visibility, Attribute};
pub use types::{Type, TypeKind, PrimitiveType, SafetyLevel};

/// The root of a T-Lang program: a collection of items (modules, functions, types, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: SourceSpan,
}

/// A module in the T-Lang module system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub items: Vec<Item>,
    pub span: SourceSpan,
}

/// Source span for tracking locations in the original source code.
/// This is essential for error reporting and IDE support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    pub fn merge(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl From<Span> for SourceSpan {
    fn from(span: Span) -> Self {
        SourceSpan::new(span.start.into(), span.len())
    }
}

impl From<SourceSpan> for Span {
    fn from(span: SourceSpan) -> Self {
        let start = span.offset();
        let len = span.len();
        Self::new(start, start + len)
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

/// Node identifier for tracking AST nodes during compilation.
/// Used for type inference, dependency analysis, and incremental compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

impl NodeId {
    pub const DUMMY: NodeId = NodeId(u32::MAX);

    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Compilation phase marker for AST nodes.
/// Tracks which compilation phases have been applied to this node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CompilationPhase {
    pub parsed: bool,
    pub name_resolved: bool,
    pub type_checked: bool,
    pub safety_checked: bool,
    pub optimized: bool,
}

impl Default for CompilationPhase {
    fn default() -> Self {
        Self {
            parsed: true,
            name_resolved: false,
            type_checked: false,
            safety_checked: false,
            optimized: false,
        }
    }
}

/// Metadata attached to AST nodes for compilation tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub id: NodeId,
    pub phase: CompilationPhase,
    pub source_file: Option<String>,
}

impl Default for NodeMetadata {
    fn default() -> Self {
        Self {
            id: NodeId::DUMMY,
            phase: CompilationPhase::default(),
            source_file: None,
        }
    }
}

impl Program {
    /// Create a new empty program.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            span: SourceSpan::new(0.into(), 0),
        }
    }

    /// Add an item to the program.
    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }

    /// Get all functions in the program.
    pub fn functions(&self) -> impl Iterator<Item = &Item> {
        self.items.iter().filter(|item| {
            matches!(item.kind, ItemKind::Function { .. })
        })
    }

    /// Get all modules in the program.
    pub fn modules(&self) -> impl Iterator<Item = &Item> {
        self.items.iter().filter(|item| {
            matches!(item.kind, ItemKind::Module { .. })
        })
    }

    /// Check if the program is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Module {
    /// Create a new module.
    pub fn new(name: String, span: SourceSpan) -> Self {
        Self {
            name,
            items: Vec::new(),
            span,
        }
    }

    /// Add an item to the module.
    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for getting spans from AST nodes.
pub trait HasSpan {
    fn span(&self) -> SourceSpan;
}

impl HasSpan for Expr {
    fn span(&self) -> SourceSpan {
        self.span
    }
}

impl HasSpan for Stmt {
    fn span(&self) -> SourceSpan {
        self.span
    }
}

impl HasSpan for Item {
    fn span(&self) -> SourceSpan {
        self.span
    }
}

impl HasSpan for Type {
    fn span(&self) -> SourceSpan {
        self.span
    }
}

impl HasSpan for Pattern {
    fn span(&self) -> SourceSpan {
        self.span
    }
}