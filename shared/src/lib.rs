// shared/src/lib.rs
//! Shared types and utilities for the T-Lang compiler.
//!
//! This crate contains the core data structures used throughout the T-Lang
//! compiler pipeline: AST nodes, tokens, and common utilities.
//!
//! All types are designed to be:
//! - Serializable for caching and debugging
//! - Clone-able for analysis passes
//! - Error-safe with no panics or unwraps

pub mod ast;
pub mod token;
pub mod tokenizer;
// Re-export commonly used types at the crate root
pub use ast::{
    Program, Module, Item, ItemKind, Stmt, StmtKind, Expr, ExprKind,
    Type, TypeKind, Pattern, PatternKind, Literal, BinaryOp, UnaryOp,
    Visibility, SafetyLevel, Span
};
pub use token::{Token, TokenType};
pub use tokenizer::{tokenize, Tokenizer};

// Re-export error handling
pub use errors::{Result, TlError};

/// Current version of the T-Lang language specification.
pub const TLANG_VERSION: &str = "0.1.0";

/// Maximum recursion depth for parsing to prevent stack overflow.
pub const MAX_RECURSION_DEPTH: usize = 256;

/// Maximum number of errors to collect before stopping compilation.
pub const MAX_ERRORS: usize = 100;

/// File extensions for T-Lang source files.
pub const TLANG_EXTENSIONS: &[&str] = &["t", "tlang"];

/// Check if a file extension indicates a T-Lang source file.
pub fn is_tlang_file(extension: &str) -> bool {
    TLANG_EXTENSIONS.contains(&extension)
}

/// Utility functions for working with source code.
pub mod source {
    use crate::Span;

    /// Calculate line and column numbers from a byte offset.
    pub fn line_col_from_offset(source: &str, offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;

        for (i, ch) in source.char_indices() {
            if i >= offset {
                break;
            }

            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (line, col)
    }

    /// Get the text content of a span from source code.
    pub fn span_text<'a>(source: &'a str, span: Span) -> &'a str {
        let start = span.start.min(source.len());
        let end = span.end.min(source.len());
        &source[start..end]
    }

    /// Get the line containing the given offset.
    pub fn line_containing_offset(source: &str, offset: usize) -> Option<&str> {
        let line_start = source[..offset].rfind('\n').map_or(0, |i| i + 1);
        let line_end = source[offset..].find('\n').map_or(source.len(), |i| offset + i);

        if line_start <= line_end {
            Some(&source[line_start..line_end])
        } else {
            None
        }
    }
}

/// Utilities for working with identifiers and names.
pub mod names {
    /// Check if a string is a valid T-Lang identifier.
    pub fn is_valid_identifier(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        let mut chars = name.chars();
        let first = chars.next().unwrap();

        // First character must be letter or underscore
        if !first.is_alphabetic() && first != '_' {
            return false;
        }

        // Remaining characters must be alphanumeric or underscore
        chars.all(|c| c.is_alphanumeric() || c == '_')
    }

    /// Check if a string is a reserved keyword.
    pub fn is_keyword(name: &str) -> bool {
        matches!(
            name,
            "as" | "async" | "await" | "break" | "const" | "continue" | "else" | "enum" |
            "false" | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" |
            "mod" | "move" | "mut" | "pub" | "ref" | "return" | "self" | "Self" |
            "static" | "struct" | "super" | "trait" | "true" | "type" | "union" |
            "unsafe" | "use" | "where" | "while"
        )
    }

    /// Escape an identifier if it conflicts with a keyword.
    pub fn escape_keyword(name: &str) -> String {
        if is_keyword(name) {
            format!("r#{}", name)
        } else {
            name.to_string()
        }
    }
}

/// Utilities for working with paths and modules.
pub mod paths {
    /// Join path segments with the T-Lang path separator (::).
    pub fn join_path(segments: &[String]) -> String {
        segments.join("::")
    }

    /// Split a path string into segments.
    pub fn split_path(path: &str) -> Vec<String> {
        path.split("::").map(|s| s.to_string()).collect()
    }

    /// Check if a path is absolute (starts with ::).
    pub fn is_absolute_path(path: &str) -> bool {
        path.starts_with("::")
    }

    /// Make a path absolute by prepending ::.
    pub fn make_absolute(path: &str) -> String {
        if is_absolute_path(path) {
            path.to_string()
        } else {
            format!("::{}", path)
        }
    }
}

/// Common type aliases used throughout the compiler.
pub mod types {
    use std::collections::HashMap;
    use crate::{Span, TokenType};

    /// A map from names to their definitions.
    pub type NameMap<T> = HashMap<String, T>;

    /// A set of source spans.
    pub type SpanSet = std::collections::HashSet<Span>;

    /// Operator precedence table.
    pub type PrecedenceTable = HashMap<TokenType, (u8, bool)>; // (precedence, right_associative)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_tlang_file() {
        assert!(is_tlang_file("t"));
        assert!(is_tlang_file("tlang"));
        assert!(!is_tlang_file("rs"));
        assert!(!is_tlang_file("txt"));
    }

    #[test]
    fn test_valid_identifier() {
        assert!(names::is_valid_identifier("hello"));
        assert!(names::is_valid_identifier("_private"));
        assert!(names::is_valid_identifier("test123"));
        assert!(names::is_valid_identifier("_"));

        assert!(!names::is_valid_identifier(""));
        assert!(!names::is_valid_identifier("123hello"));
        assert!(!names::is_valid_identifier("hello-world"));
        assert!(!names::is_valid_identifier("hello world"));
    }

    #[test]
    fn test_keyword_detection() {
        assert!(names::is_keyword("fn"));
        assert!(names::is_keyword("let"));
        assert!(names::is_keyword("if"));
        assert!(!names::is_keyword("hello"));
        assert!(!names::is_keyword("function"));
    }

    #[test]
    fn test_path_operations() {
        assert_eq!(paths::join_path(&["std".to_string(), "io".to_string()]), "std::io");
        assert_eq!(paths::split_path("std::io::Write"), vec!["std", "io", "Write"]);
        assert!(paths::is_absolute_path("::std::io"));
        assert!(!paths::is_absolute_path("std::io"));
        assert_eq!(paths::make_absolute("std::io"), "::std::io");
    }

    #[test]
    fn test_line_col_from_offset() {
        let source = "hello\nworld\ntest";
        assert_eq!(source::line_col_from_offset(source, 0), (1, 1));
        assert_eq!(source::line_col_from_offset(source, 6), (2, 1));
        assert_eq!(source::line_col_from_offset(source, 12), (3, 1));
    }
}