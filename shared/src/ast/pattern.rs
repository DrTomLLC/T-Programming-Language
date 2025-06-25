// shared/src/ast/pattern.rs
//! Pattern definitions for T-Lang AST.

use crate::{Span, span::HasSpan};
use super::Literal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
}

impl HasSpan for Pattern {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternKind {
    Wildcard,
    Identifier(String),
    Literal(Literal),
    Tuple(Vec<Pattern>),
    Array(Vec<Pattern>),
    Struct {
        name: String,
        fields: Vec<(String, Pattern)>,
    },
    Enum {
        name: String,
        variant: String,
        data: Option<Box<Pattern>>,
    },
    Reference(Box<Pattern>),
    Range {
        start: Option<Box<Pattern>>,
        end: Option<Box<Pattern>>,
        inclusive: bool,
    },
}

impl Pattern {
    pub fn identifier(name: String, span: Span) -> Self {
        Self {
            kind: PatternKind::Identifier(name),
            span,
        }
    }
}

/// Extract all identifiers bound by a pattern.
pub fn bound_identifiers(pattern: &Pattern) -> Vec<String> {
    let mut identifiers = Vec::new();
    collect_identifiers(pattern, &mut identifiers);
    identifiers
}

fn collect_identifiers(pattern: &Pattern, identifiers: &mut Vec<String>) {
    match &pattern.kind {
        PatternKind::Identifier(name) => {
            identifiers.push(name.clone());
        }
        PatternKind::Tuple(patterns) | PatternKind::Array(patterns) => {
            for pattern in patterns {
                collect_identifiers(pattern, identifiers);
            }
        }
        PatternKind::Struct { fields, .. } => {
            for (_, pattern) in fields {
                collect_identifiers(pattern, identifiers);
            }
        }
        PatternKind::Enum { data, .. } => {
            if let Some(pattern) = data {
                collect_identifiers(pattern, identifiers);
            }
        }
        PatternKind::Reference(pattern) => {
            collect_identifiers(pattern, identifiers);
        }
        _ => {}
    }
}