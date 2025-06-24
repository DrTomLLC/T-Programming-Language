// shared/src/ast/stmt.rs
//! Statement AST nodes for T-Lang.

use super::{Expr, Pattern, Type};
use miette::SourceSpan;
use serde::{Deserialize, Serialize};

/// A statement in T-Lang.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: SourceSpan,
}

/// All possible statement kinds in T-Lang.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StmtKind {
    /// Expression statement: expr;
    Expr(Expr),

    /// Let binding: let pat: Type = expr;
    Let {
        pattern: Pattern,
        ty: Option<Type>,
        initializer: Option<Expr>,
        mutable: bool,
    },

    /// Item definition (function, struct, enum, etc.)
    Item(super::Item),

    /// Macro invocation: macro_name!(args);
    Macro {
        path: Vec<String>,
        args: Vec<super::MacroArg>,
    },
}

impl Stmt {
    /// Create an expression statement.
    pub fn expr(expr: Expr, span: SourceSpan) -> Self {
        Self {
            kind: StmtKind::Expr(expr),
            span,
        }
    }

    /// Create a let statement.
    pub fn let_stmt(
        pattern: Pattern,
        ty: Option<Type>,
        initializer: Option<Expr>,
        span: SourceSpan,
    ) -> Self {
        Self {
            kind: StmtKind::Let {
                pattern,
                ty,
                initializer,
                mutable: false,
            },
            span,
        }
    }

    /// Create a mutable let statement.
    pub fn let_mut(
        pattern: Pattern,
        ty: Option<Type>,
        initializer: Option<Expr>,
        span: SourceSpan,
    ) -> Self {
        Self {
            kind: StmtKind::Let {
                pattern,
                ty,
                initializer,
                mutable: true,
            },
            span,
        }
    }

    /// Create an item statement.
    pub fn item(item: super::Item, span: SourceSpan) -> Self {
        Self {
            kind: StmtKind::Item(item),
            span,
        }
    }

    /// Check if this statement is an expression statement.
    pub fn is_expr(&self) -> bool {
        matches!(self.kind, StmtKind::Expr(_))
    }

    /// Check if this statement is a let binding.
    pub fn is_let(&self) -> bool {
        matches!(self.kind, StmtKind::Let { .. })
    }

    /// Check if this statement is an item definition.
    pub fn is_item(&self) -> bool {
        matches!(self.kind, StmtKind::Item(_))
    }

    /// Check if this statement introduces new bindings.
    pub fn introduces_bindings(&self) -> bool {
        matches!(self.kind, StmtKind::Let { .. })
    }

    /// Get all identifiers bound by this statement.
    pub fn bound_identifiers(&self) -> Vec<String> {
        match &self.kind {
            StmtKind::Let { pattern, .. } => {
                crate::patterns::bound_identifiers(pattern)
            }
            _ => Vec::new(),
        }
    }
}