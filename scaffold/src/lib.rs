//! scaffold/src/lib.rs
//! T-Lang Scaffold Compiler Library
//!
//! Phase 2 Week 1 Day 1: Error System Integration
//! This library exposes all scaffold compiler components

pub mod ast;
pub mod parser;
pub mod typechecker;
pub mod codegen;
pub mod compile;

// NEW: Phase 2 Week 1 Day 1 additions
pub mod error_bridge;
pub mod diagnostics;

// Re-export commonly used types for convenience
pub use ast::*;
pub use parser::{Parser, ParseError};
pub use typechecker::{TypeChecker, TypeError};
pub use codegen::{CodeGenerator, CodegenError};
pub use compile::{Compiler, CompileError};

// Re-export new error system types
pub use error_bridge::{ScaffoldResult, convert_parse_error, convert_type_error};
pub use diagnostics::ScaffoldDiagnostics;