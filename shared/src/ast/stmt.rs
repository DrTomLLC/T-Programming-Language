// shared/src/ast/stmt.rs
//! Statement definitions for T-Lang AST.

use crate::{Span, span::HasSpan};
use super::{Expr, Pattern, Type, Item};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

impl HasSpan for Stmt {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StmtKind {
    Let {
        pattern: Pattern,
        ty: Option<Type>,
        initializer: Option<Expr>,
        mutable: bool,
    },
    Item(Item),
    Expr(Expr),
    Semi(Expr),
    Return { value: Option<Expr> },
    Macro {
        path: Vec<String>,
        args: Vec<MacroArg>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MacroArg {
    Literal(super::Literal),
    Identifier(String),
    Path(Vec<String>),
    Expression(Expr),
    Type(Type),
    Pattern(Pattern),
    Statement(Stmt),
    TokenStream(Vec<crate::Token>),
}