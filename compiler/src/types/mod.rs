// compiler/src/types/mod.rs
//! Type system implementation for T-Lang.
//!
//! Provides comprehensive type checking, inference, and analysis for safety-critical systems.
//! Designed to catch type errors early and ensure memory safety.

pub mod checker;
pub mod inference;
pub mod coercion;
mod coercion;

pub use checker::{TypeChecker, FunctionSignature, TypeDefinition, TypeConstraint};
pub use inference::{TypeInferer, InferenceContext, TypeVariable};
pub use coercion::{CoercionRules, CoercionKind};

use shared::{Type, TypeKind, PrimitiveType, Result, TlError};
use miette::SourceSpan;

/// Type checking entry point for programs.
pub fn check_program(program: &mut shared::Program, source: String) -> Result<()> {
    let mut checker = TypeChecker::new(source);
    checker.check_program(program)
}

/// Type check a single expression.
pub fn check_expression(expr: &mut shared::Expr, source: String) -> Result<Type> {
    let mut checker = TypeChecker::new(source);
    checker.check_expr(expr)
}

/// Utility functions for working with types.
pub mod utils {
    use super::*;

    /// Check if a type is numeric.
    pub fn is_numeric_type(ty: &Type) -> bool {
        match &ty.kind {
            TypeKind::Primitive(prim) => matches!(
                prim,
                PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32 | 
                PrimitiveType::I64 | PrimitiveType::I128 | PrimitiveType::ISize |
                PrimitiveType::U8 | PrimitiveType::U16 | PrimitiveType::U32 | 
                PrimitiveType::U64 | PrimitiveType::U128 | PrimitiveType::USize |
                PrimitiveType::F32 | PrimitiveType::F64
            ),
            _ => false,
        }
    }

    /// Check if a type is an integer type.
    pub fn is_integer_type(ty: &Type) -> bool {
        match &ty.kind {
            TypeKind::Primitive(prim) => matches!(
                prim,
                PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32 | 
                PrimitiveType::I64 | PrimitiveType::I128 | PrimitiveType::ISize |
                PrimitiveType::U8 | PrimitiveType::U16 | PrimitiveType::U32 | 
                PrimitiveType::U64 | PrimitiveType::U128 | PrimitiveType::USize
            ),
            _ => false,
        }
    }

    /// Check if a type is a floating-point type.
    pub fn is_float_type(ty: &Type) -> bool {
        match &ty.kind {
            TypeKind::Primitive(prim) => matches!(prim, PrimitiveType::F32 | PrimitiveType::F64),
            _ => false,
        }
    }

    /// Check if a type is signed.
    pub fn is_signed_type(ty: &Type) -> bool {
        match &ty.kind {
            TypeKind::Primitive(prim) => matches!(
                prim,
                PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32 | 
                PrimitiveType::I64 | PrimitiveType::I128 | PrimitiveType::ISize |
                PrimitiveType::F32 | PrimitiveType::F64
            ),
            _ => false,
        }
    }

    /// Get the size in bits of a primitive type.
    pub fn type_size_bits(ty: &Type) -> Option<u32> {
        match &ty.kind {
            TypeKind::Primitive(prim) => match prim {
                PrimitiveType::I8 | PrimitiveType::U8 => Some(8),
                PrimitiveType::I16 | PrimitiveType::U16 => Some(16),
                PrimitiveType::I32 | PrimitiveType::U32 | PrimitiveType::F32 => Some(32),
                PrimitiveType::I64 | PrimitiveType::U64 | PrimitiveType::F64 => Some(64),
                PrimitiveType::I128 | PrimitiveType::U128 => Some(128),
                PrimitiveType::ISize | PrimitiveType::USize => Some(64), // Assume 64-bit target
                PrimitiveType::Bool => Some(1),
                PrimitiveType::Char => Some(32), // Unicode scalar value
                _ => None,
            },
            _ => None,
        }
    }

    /// Check if two types are structurally equivalent.
    pub fn types_equivalent(a: &Type, b: &Type) -> bool {
        match (&a.kind, &b.kind) {
            (TypeKind::Primitive(a), TypeKind::Primitive(b)) => a == b,

            (TypeKind::Reference { target: a_target, mutable: a_mut, .. },
                TypeKind::Reference { target: b_target, mutable: b_mut, .. }) => {
                a_mut == b_mut && types_equivalent(a_target, b_target)
            }

            (TypeKind::Array { element: a_elem, size: a_size },
                TypeKind::Array { element: b_elem, size: b_size }) => {
                a_size == b_size && types_equivalent(a_elem, b_elem)
            }

            (TypeKind::Function { params: a_params, return_type: a_ret, .. },
                TypeKind::Function { params: b_params, return_type: b_ret, .. }) => {
                a_params.len() == b_params.len() &&
                    types_equivalent(a_ret, b_ret) &&
                    a_params.iter().zip(b_params.iter()).all(|(a, b)| types_equivalent(a, b))
            }

            (TypeKind::Named { path: a_path, generics: a_generics },
                TypeKind::Named { path: b_path, generics: b_generics }) => {
                a_path == b_path &&
                    a_generics.len() == b_generics.len() &&
                    a_generics.iter().zip(b_generics.iter()).all(|(a, b)| types_equivalent(a, b))
            }

            _ => false,
        }
    }

    /// Create a unit type.
    pub fn unit_type() -> Type {
        Type::new(TypeKind::Primitive(PrimitiveType::Unit), SourceSpan::new(0.into(), 0))
    }

    /// Create a bool type.
    pub fn bool_type() -> Type {
        Type::new(TypeKind::Primitive(PrimitiveType::Bool), SourceSpan::new(0.into(), 0))
    }

    /// Create an i32 type.
    pub fn i32_type() -> Type {
        Type::new(TypeKind::Primitive(PrimitiveType::I32), SourceSpan::new(0.into(), 0))
    }

    /// Create an f64 type.
    pub fn f64_type() -> Type {
        Type::new(TypeKind::Primitive(PrimitiveType::F64), SourceSpan::new(0.into(), 0))
    }

    /// Create a string type.
    pub fn string_type() -> Type {
        Type::new(TypeKind::Primitive(PrimitiveType::Str), SourceSpan::new(0.into(), 0))
    }
}

/// Built-in type constants for convenience.
pub mod builtin {
    use super::*;
    use super::utils::*;

    /// The unit type: ()
    pub static UNIT: once_cell::sync::Lazy<Type> = once_cell::sync::Lazy::new(unit_type);

    /// The boolean type: bool
    pub static BOOL: once_cell::sync::Lazy<Type> = once_cell::sync::Lazy::new(bool_type);

    /// The default integer type: i32
    pub static INT: once_cell::sync::Lazy<Type> = once_cell::sync::Lazy::new(i32_type);

    /// The default float type: f64
    pub static FLOAT: once_cell::sync::Lazy<Type> = once_cell::sync::Lazy::new(f64_type);

    /// The string type: str
    pub static STRING: once_cell::sync::Lazy<Type> = once_cell::sync::Lazy::new(string_type);
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::utils::*;
    use shared::{Expr, ExprKind, Literal};

    #[test]
    fn test_numeric_type_detection() {
        let i32_type = i32_type();
        let f64_type = f64_type();
        let bool_type = bool_type();

        assert!(is_numeric_type(&i32_type));
        assert!(is_numeric_type(&f64_type));
        assert!(!is_numeric_type(&bool_type));

        assert!(is_integer_type(&i32_type));
        assert!(!is_integer_type(&f64_type));
        assert!(!is_integer_type(&bool_type));

        assert!(!is_float_type(&i32_type));
        assert!(is_float_type(&f64_type));
        assert!(!is_float_type(&bool_type));
    }

    #[test]
    fn test_type_equivalence() {
        let i32_a = i32_type();
        let i32_b = i32_type();
        let f64_type = f64_type();

        assert!(types_equivalent(&i32_a, &i32_b));
        assert!(!types_equivalent(&i32_a, &f64_type));
    }

    #[test]
    fn test_type_sizes() {
        let i32_type = i32_type();
        let f64_type = f64_type();
        let bool_type = bool_type();

        assert_eq!(type_size_bits(&i32_type), Some(32));
        assert_eq!(type_size_bits(&f64_type), Some(64));
        assert_eq!(type_size_bits(&bool_type), Some(1));
    }

    #[test]
    fn test_simple_expression_typing() {
        let source = "42".to_string();
        let mut expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42)),
            SourceSpan::new(0.into(), 2),
        );

        let result = check_expression(&mut expr, source);
        assert!(result.is_ok());

        let expr_type = result.unwrap();
        assert!(is_integer_type(&expr_type));
    }
}