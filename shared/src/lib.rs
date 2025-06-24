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
mod utils;
mod tir;

// Re-export commonly used types at the crate root
pub use ast::{
    Program, Module, Item, ItemKind, Stmt, StmtKind, Expr, ExprKind,
    Type, TypeKind, Pattern, PatternKind, Literal, BinaryOp, UnaryOp,
    Visibility, SafetyLevel, Span, NodeId, CompilationPhase, NodeMetadata,
    StructFields, StructField, EnumVariant, FnParam, GenericParam,
    TraitItem, ImplItem, ExternItem, Attribute, MacroArg, MacroRule,
    HasSpan
};
pub use token::{Token, TokenType, TokenStream};
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
    use miette::SourceSpan;

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
    pub fn span_text<'a>(source: &'a str, span: SourceSpan) -> &'a str {
        let start = span.offset().min(source.len());
        let end = (span.offset() + span.len()).min(source.len());
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

    /// Convert a SourceSpan to line/column information.
    pub fn span_to_line_col(source: &str, span: SourceSpan) -> ((usize, usize), (usize, usize)) {
        let start = line_col_from_offset(source, span.offset());
        let end = line_col_from_offset(source, span.offset() + span.len());
        (start, end)
    }

    /// Extract all lines that intersect with the given span.
    pub fn lines_in_span(source: &str, span: SourceSpan) -> Vec<&str> {
        let start_offset = span.offset();
        let end_offset = span.offset() + span.len();

        let mut lines = Vec::new();
        let mut current_line_start = 0;

        for (i, ch) in source.char_indices() {
            if ch == '\n' || i == source.len() - 1 {
                let line_end = if ch == '\n' { i } else { i + 1 };

                // Check if this line intersects with the span
                if current_line_start < end_offset && line_end > start_offset {
                    lines.push(&source[current_line_start..line_end]);
                }

                current_line_start = i + 1;

                // Early exit if we've passed the span
                if current_line_start > end_offset {
                    break;
                }
            }
        }

        lines
    }
}

/// Utilities for working with identifiers and names.
pub mod names {
    /// Check if a string is a valid T-Lang identifier.
    pub fn is_valid_identifier(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // First character must be letter or underscore
        let mut chars = name.chars();
        let first = chars.next().unwrap();
        if !first.is_alphabetic() && first != '_' {
            return false;
        }

        // Remaining characters can be alphanumeric or underscore
        chars.all(|c| c.is_alphanumeric() || c == '_')
    }

    /// Check if a string is a reserved keyword.
    pub fn is_keyword(name: &str) -> bool {
        matches!(
            name,
            "as" | "async" | "await" | "break" | "const" | "continue" | "else" |
            "enum" | "false" | "fn" | "for" | "if" | "impl" | "in" | "let" |
            "loop" | "match" | "mod" | "move" | "mut" | "pub" | "ref" |
            "return" | "self" | "Self" | "static" | "struct" | "super" |
            "trait" | "true" | "type" | "union" | "unsafe" | "use" | "where" | "while"
        )
    }

    /// Mangle a name for use in generated code.
    pub fn mangle_name(name: &str) -> String {
        if is_keyword(name) {
            format!("r#{}", name)
        } else {
            name.to_string()
        }
    }

    /// Generate a unique temporary variable name.
    pub fn temp_var_name(base: &str, counter: usize) -> String {
        format!("__{}__{}", base, counter)
    }

    /// Check if a name looks like a generated temporary variable.
    pub fn is_temp_var(name: &str) -> bool {
        name.starts_with("__") && name.contains("__")
    }
}

/// Utilities for working with types and type checking.
pub mod types {
    use crate::{Type, TypeKind};

    /// Check if a type is a primitive type.
    pub fn is_primitive(ty: &Type) -> bool {
        matches!(ty.kind, TypeKind::Primitive(_))
    }

    /// Check if a type is numeric.
    pub fn is_numeric(ty: &Type) -> bool {
        matches!(
            ty.kind,
            TypeKind::Primitive(p) if matches!(p,
                crate::ast::PrimitiveType::I8 | crate::ast::PrimitiveType::I16 |
                crate::ast::PrimitiveType::I32 | crate::ast::PrimitiveType::I64 |
                crate::ast::PrimitiveType::I128 | crate::ast::PrimitiveType::U8 |
                crate::ast::PrimitiveType::U16 | crate::ast::PrimitiveType::U32 |
                crate::ast::PrimitiveType::U64 | crate::ast::PrimitiveType::U128 |
                crate::ast::PrimitiveType::F32 | crate::ast::PrimitiveType::F64
            )
        )
    }

    /// Check if a type is an integer type.
    pub fn is_integer(ty: &Type) -> bool {
        matches!(
            ty.kind,
            TypeKind::Primitive(p) if matches!(p,
                crate::ast::PrimitiveType::I8 | crate::ast::PrimitiveType::I16 |
                crate::ast::PrimitiveType::I32 | crate::ast::PrimitiveType::I64 |
                crate::ast::PrimitiveType::I128 | crate::ast::PrimitiveType::U8 |
                crate::ast::PrimitiveType::U16 | crate::ast::PrimitiveType::U32 |
                crate::ast::PrimitiveType::U64 | crate::ast::PrimitiveType::U128
            )
        )
    }

    /// Check if a type is a floating point type.
    pub fn is_float(ty: &Type) -> bool {
        matches!(
            ty.kind,
            TypeKind::Primitive(p) if matches!(p,
                crate::ast::PrimitiveType::F32 | crate::ast::PrimitiveType::F64
            )
        )
    }

    /// Check if a type is a boolean type.
    pub fn is_bool(ty: &Type) -> bool {
        matches!(ty.kind, TypeKind::Primitive(crate::ast::PrimitiveType::Bool))
    }

    /// Check if a type is the unit type.
    pub fn is_unit(ty: &Type) -> bool {
        matches!(ty.kind, TypeKind::Primitive(crate::ast::PrimitiveType::Unit))
    }

    /// Get the size in bits of a primitive type.
    pub fn primitive_size_bits(prim: &crate::ast::PrimitiveType) -> Option<usize> {
        use crate::ast::PrimitiveType::*;
        match prim {
            Bool => Some(1),
            I8 | U8 => Some(8),
            I16 | U16 => Some(16),
            I32 | U32 | F32 => Some(32),
            I64 | U64 | F64 => Some(64),
            I128 | U128 => Some(128),
            Char => Some(32), // Unicode scalar value
            Unit => Some(0),
            Str => None, // Variable size
        }
    }
}

/// Utilities for working with expressions and evaluation.
pub mod exprs {
    use crate::{Expr, ExprKind, Literal, BinaryOp, UnaryOp};

    /// Check if an expression is a literal.
    pub fn is_literal(expr: &Expr) -> bool {
        matches!(expr.kind, ExprKind::Literal(_))
    }

    /// Check if an expression is a constant (compile-time evaluable).
    pub fn is_constant(expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Literal(_) => true,
            ExprKind::Binary { lhs, op, rhs } => {
                is_constant(lhs) && is_constant(rhs) && can_const_eval_binop(*op)
            }
            ExprKind::Unary { op, operand } => {
                is_constant(operand) && can_const_eval_unop(*op)
            }
            ExprKind::Grouping(inner) => is_constant(inner),
            _ => false,
        }
    }

    /// Check if a binary operator can be evaluated at compile time.
    pub fn can_const_eval_binop(op: BinaryOp) -> bool {
        matches!(
            op,
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div |
            BinaryOp::Mod | BinaryOp::And | BinaryOp::Or | BinaryOp::Xor |
            BinaryOp::Shl | BinaryOp::Shr | BinaryOp::Eq | BinaryOp::Ne |
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge |
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr
        )
    }

    /// Check if a unary operator can be evaluated at compile time.
    pub fn can_const_eval_unop(op: UnaryOp) -> bool {
        matches!(op, UnaryOp::Not | UnaryOp::Neg | UnaryOp::Plus)
    }

    /// Try to extract a boolean value from a literal expression.
    pub fn literal_as_bool(expr: &Expr) -> Option<bool> {
        if let ExprKind::Literal(Literal::Bool(b)) = &expr.kind {
            Some(*b)
        } else {
            None
        }
    }

    /// Try to extract an integer value from a literal expression.
    pub fn literal_as_int(expr: &Expr) -> Option<i64> {
        if let ExprKind::Literal(Literal::Integer(i)) = &expr.kind {
            Some(*i)
        } else {
            None
        }
    }

    /// Try to extract a float value from a literal expression.
    pub fn literal_as_float(expr: &Expr) -> Option<f64> {
        if let ExprKind::Literal(Literal::Float(f)) = &expr.kind {
            Some(*f)
        } else {
            None
        }
    }

    /// Try to extract a string value from a literal expression.
    pub fn literal_as_string(expr: &Expr) -> Option<&str> {
        if let ExprKind::Literal(Literal::String(s)) = &expr.kind {
            Some(s)
        } else {
            None
        }
    }
}

/// Utilities for working with patterns and pattern matching.
pub mod patterns {
    use crate::{Pattern, PatternKind};

    /// Check if a pattern is irrefutable (always matches).
    pub fn is_irrefutable(pattern: &Pattern) -> bool {
        match &pattern.kind {
            PatternKind::Wildcard => true,
            PatternKind::Identifier { .. } => true,
            PatternKind::Tuple(patterns) => patterns.iter().all(is_irrefutable),
            PatternKind::Struct { fields, .. } => {
                fields.iter().all(|(_, pattern)| is_irrefutable(pattern))
            }
            PatternKind::Literal(_) => false,
            PatternKind::Range { .. } => false,
            PatternKind::Or(patterns) => patterns.iter().any(is_irrefutable),
            PatternKind::Ref(inner) => is_irrefutable(inner),
        }
    }

    /// Get all identifiers bound by a pattern.
    pub fn bound_identifiers(pattern: &Pattern) -> Vec<String> {
        let mut idents = Vec::new();
        collect_bound_identifiers(pattern, &mut idents);
        idents
    }

    fn collect_bound_identifiers(pattern: &Pattern, idents: &mut Vec<String>) {
        match &pattern.kind {
            PatternKind::Identifier { name, .. } => {
                idents.push(name.clone());
            }
            PatternKind::Tuple(patterns) => {
                for pattern in patterns {
                    collect_bound_identifiers(pattern, idents);
                }
            }
            PatternKind::Struct { fields, .. } => {
                for (_, pattern) in fields {
                    collect_bound_identifiers(pattern, idents);
                }
            }
            PatternKind::Or(patterns) => {
                // For OR patterns, all alternatives must bind the same identifiers
                if let Some(first) = patterns.first() {
                    collect_bound_identifiers(first, idents);
                }
            }
            PatternKind::Ref(inner) => {
                collect_bound_identifiers(inner, idents);
            }
            PatternKind::Wildcard | PatternKind::Literal(_) | PatternKind::Range { .. } => {
                // These don't bind identifiers
            }
        }
    }
}

/// Testing utilities.
#[cfg(test)]
pub mod test_utils {
    use crate::*;
    use miette::SourceSpan;

    /// Create a dummy source span for testing.
    pub fn dummy_span() -> SourceSpan {
        SourceSpan::new(0.into(), 0)
    }

    /// Create a test identifier expression.
    pub fn test_ident(name: &str) -> Expr {
        Expr {
            kind: ExprKind::Identifier(name.to_string()),
            span: dummy_span(),
        }
    }

    /// Create a test integer literal.
    pub fn test_int(value: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(value)),
            span: dummy_span(),
        }
    }

    /// Create a test string literal.
    pub fn test_string(value: &str) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::String(value.to_string())),
            span: dummy_span(),
        }
    }

    /// Create a test boolean literal.
    pub fn test_bool(value: bool) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Bool(value)),
            span: dummy_span(),
        }
    }

    /// Create a test binary expression.
    pub fn test_binary(lhs: Expr, op: BinaryOp, rhs: Expr) -> Expr {
        Expr {
            kind: ExprKind::Binary {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            },
            span: dummy_span(),
        }
    }

    /// Create a test function call.
    pub fn test_call(func: Expr, args: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Call {
                func: Box::new(func),
                args,
            },
            span: dummy_span(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_extension_detection() {
        assert!(is_tlang_file("t"));
        assert!(is_tlang_file("tlang"));
        assert!(!is_tlang_file("rs"));
        assert!(!is_tlang_file("c"));
    }

    #[test]
    fn test_line_col_calculation() {
        let source = "line 1\nline 2\nline 3";

        assert_eq!(source::line_col_from_offset(source, 0), (1, 1));
        assert_eq!(source::line_col_from_offset(source, 6), (1, 7)); // end of "line 1"
        assert_eq!(source::line_col_from_offset(source, 7), (2, 1)); // start of "line 2"
        assert_eq!(source::line_col_from_offset(source, 14), (3, 1)); // start of "line 3"
    }

    #[test]
    fn test_span_text_extraction() {
        let source = "hello world";
        let span = SourceSpan::new(6.into(), 5); // "world"

        assert_eq!(source::span_text(source, span), "world");
    }

    #[test]
    fn test_identifier_validation() {
        assert!(names::is_valid_identifier("hello"));
        assert!(names::is_valid_identifier("_private"));
        assert!(names::is_valid_identifier("CamelCase"));
        assert!(names::is_valid_identifier("snake_case"));
        assert!(names::is_valid_identifier("name123"));

        assert!(!names::is_valid_identifier(""));
        assert!(!names::is_valid_identifier("123abc"));
        assert!(!names::is_valid_identifier("hello-world"));
        assert!(!names::is_valid_identifier("hello.world"));
    }

    #[test]
    fn test_keyword_detection() {
        assert!(names::is_keyword("fn"));
        assert!(names::is_keyword("let"));
        assert!(names::is_keyword("if"));
        assert!(names::is_keyword("while"));

        assert!(!names::is_keyword("hello"));
        assert!(!names::is_keyword("world"));
        assert!(!names::is_keyword("function"));
    }

    #[test]
    fn test_name_mangling() {
        assert_eq!(names::mangle_name("hello"), "hello");
        assert_eq!(names::mangle_name("fn"), "r#fn");
        assert_eq!(names::mangle_name("let"), "r#let");
    }

    #[test]
    fn test_temp_var_generation() {
        assert_eq!(names::temp_var_name("temp", 0), "__temp__0");
        assert_eq!(names::temp_var_name("local", 42), "__local__42");

        assert!(names::is_temp_var("__temp__0"));
        assert!(names::is_temp_var("__local__42"));
        assert!(!names::is_temp_var("normal_var"));
        assert!(!names::is_temp_var("__single"));
    }

    #[test]
    fn test_constant_expression_detection() {
        use test_utils::*;

        assert!(exprs::is_constant(&test_int(42)));
        assert!(exprs::is_constant(&test_bool(true)));
        assert!(exprs::is_constant(&test_string("hello")));

        let add_expr = test_binary(test_int(1), BinaryOp::Add, test_int(2));
        assert!(exprs::is_constant(&add_expr));

        let call_expr = test_call(test_ident("func"), vec![test_int(1)]);
        assert!(!exprs::is_constant(&call_expr));
    }

    #[test]
    fn test_literal_value_extraction() {
        use test_utils::*;

        assert_eq!(exprs::literal_as_int(&test_int(42)), Some(42));
        assert_eq!(exprs::literal_as_bool(&test_bool(true)), Some(true));
        assert_eq!(exprs::literal_as_string(&test_string("hello")), Some("hello"));

        assert_eq!(exprs::literal_as_int(&test_bool(true)), None);
        assert_eq!(exprs::literal_as_bool(&test_int(42)), None);
    }
}