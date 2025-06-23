// File: compiler/src/parser/mod.rs

//! Main entry point for the T‑Lang parser.
//!
//! This module organizes the submodules responsible for parsing different
//! parts of the language: modules, declarations, statements, expressions,
//! patterns, and types.

mod modules;
mod declarations;
mod statements;
mod expressions;
mod patterns;
mod types;

pub use modules::parse_module;
