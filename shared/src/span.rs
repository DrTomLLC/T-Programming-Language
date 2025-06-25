// shared/src/span.rs
//! Span utilities for source location tracking.

use miette::SourceSpan;
use serde::{Deserialize, Serialize};

/// A span in source code with default support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub len: usize,
}

impl Span {
    pub fn new(start: usize, len: usize) -> Self {
        Self { start, len }
    }

    pub fn end(&self) -> usize {
        self.start + self.len
    }
}

impl Default for Span {
    fn default() -> Self {
        Self { start: 0, len: 0 }
    }
}

impl From<Span> for SourceSpan {
    fn from(span: Span) -> Self {
        SourceSpan::from((span.start, span.len))
    }
}

impl From<SourceSpan> for Span {
    fn from(span: SourceSpan) -> Self {
        Self {
            start: span.offset(),
            len: span.len(),
        }
    }
}

impl From<(usize, usize)> for Span {
    fn from((start, len): (usize, usize)) -> Self {
        Self { start, len }
    }
}

pub trait HasSpan {
    fn span(&self) -> Span;
}