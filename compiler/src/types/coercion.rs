// compiler/src/types/coercion.rs
//! Type coercion rules for T-Lang.
//!
//! Handles automatic type conversions and subtyping relationships.
//! Designed for safety-critical systems with explicit coercion rules.

use shared::{Type, TypeKind, PrimitiveType, Result, TlError};
use miette::SourceSpan;

/// Type coercion engine for automatic type conversions.
pub struct CoercionRules {
    /// Source code for error reporting
    source: String,
}

/// Types of coercions that can be performed.
#[derive(Debug, Clone, PartialEq)]
pub enum CoercionKind {
    /// No coercion needed - types are identical
    Identity,
    /// Subtyping coercion (e.g., &mut T to &T)
    Subtyping,
    /// Numeric coercion (e.g., i32 to f64)
    Numeric,
    /// Reference coercion (e.g., &T to *const T)
    Reference,
    /// Array to slice coercion (e.g., [T; N] to [T])
    ArrayToSlice,
    /// Function item to function pointer
    FunctionItem,
    /// Closure to function pointer
    Closure,
    /// Never type coercion (! to any type)
    Never,
    /// Custom coercion (user-defined)
    Custom(String),
}

/// Result of a coercion attempt.
#[derive(Debug, Clone)]
pub struct CoercionResult {
    /// The kind of coercion that was applied
    pub kind: CoercionKind,
    /// The resulting type after coercion
    pub target_type: Type,
    /// Whether the coercion is safe (no loss of precision/information)
    pub is_safe: bool,
    /// Cost of the coercion (for overload resolution)
    pub cost: CoercionCost,
}

/// Cost associated with a coercion for overload resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoercionCost {
    /// No cost - perfect match
    Free = 0,
    /// Low cost - safe conversion
    Low = 1,
    /// Medium cost - some information might be lost
    Medium = 2,
    /// High cost - significant change in representation
    High = 3,
    /// Very high cost - potentially unsafe conversion
    VeryHigh = 4,
}

impl CoercionRules {
    /// Create a new coercion rules engine.
    pub fn new(source: String) -> Self {
        Self { source }
    }

    /// Check if one type can be coerced to another.
    pub fn can_coerce(&self, from: &Type, to: &Type) -> bool {
        self.try_coerce(from, to, SourceSpan::new(0.into(), 0)).is_ok()
    }

    /// Attempt to coerce one type to another.
    pub fn try_coerce(&self, from: &Type, to: &Type, span: SourceSpan) -> Result<CoercionResult> {
        // Check for identity (no coercion needed)
        if self.types_identical(from, to) {
            return Ok(CoercionResult {
                kind: CoercionKind::Identity,
                target_type: to.clone(),
                is_safe: true,
                cost: CoercionCost::Free,
            });
        }

        // Try different coercion rules in order of preference

        // 1. Never type coerces to anything
        if let TypeKind::Never = from.kind {
            return Ok(CoercionResult {
                kind: CoercionKind::Never,
                target_type: to.clone(),
                is_safe: true,
                cost: CoercionCost::Free,
            });
        }

        // 2. Subtyping coercions
        if let Some(result) = self.try_subtyping_coercion(from, to)? {
            return Ok(result);
        }

        // 3. Numeric coercions
        if let Some(result) = self.try_numeric_coercion(from, to, span)? {
            return Ok(result);
        }

        // 4. Reference coercions
        if let Some(result) = self.try_reference_coercion(from, to)? {
            return Ok(result);
        }

        // 5. Array to slice coercion
        if let Some(result) = self.try_array_to_slice_coercion(from, to)? {
            return Ok(result);
        }

        // 6. Function coercions
        if let Some(result) = self.try_function_coercion(from, to)? {
            return Ok(result);
        }

        // No coercion possible
        Err(TlError::type_error(
            self.source.clone(),
            span,
            format!("Cannot coerce type {:?} to {:?}", from.kind, to.kind),
        ))
    }

    /// Find the best common type for a set of types.
    pub fn find_common_type(&self, types: &[Type], span: SourceSpan) -> Result<Type> {
        if types.is_empty() {
            return Err(TlError::type_error(
                self.source.clone(),
                span,
                "Cannot find common type for empty type list".to_string(),
            ));
        }

        if types.len() == 1 {
            return Ok(types[0].clone());
        }

        // Start with the first type and try to find a common supertype
        let mut common = types[0].clone();

        for ty in &types[1..] {
            common = self.find_common_supertype(&common, ty, span)?;
        }

        Ok(common)
    }

    /// Check if two types are identical.
    fn types_identical(&self, a: &Type, b: &Type) -> bool {
        // For now, use structural equality
        // TODO: Handle type aliases and other equivalences
        a.kind == b.kind
    }

    /// Try subtyping coercion.
    fn try_subtyping_coercion(&self, from: &Type, to: &Type) -> Result<Option<CoercionResult>> {
        match (&from.kind, &to.kind) {
            // &mut T to &T (mutable reference to immutable reference)
            (TypeKind::Reference { target: from_target, mutable: true, .. },
                TypeKind::Reference { target: to_target, mutable: false, .. }) => {
                if self.types_identical(from_target, to_target) {
                    return Ok(Some(CoercionResult {
                        kind: CoercionKind::Subtyping,
                        target_type: to.clone(),
                        is_safe: true,
                        cost: CoercionCost::Free,
                    }));
                }
            }

            // TODO: Add other subtyping rules (trait objects, lifetimes, etc.)
            _ => {}
        }

        Ok(None)
    }

    /// Try numeric coercion.
    fn try_numeric_coercion(&self, from: &Type, to: &Type, span: SourceSpan) -> Result<Option<CoercionResult>> {
        if let (TypeKind::Primitive(from_prim), TypeKind::Primitive(to_prim)) = (&from.kind, &to.kind) {
            if let Some((kind, cost, is_safe)) = self.numeric_conversion_info(from_prim, to_prim) {
                return Ok(Some(CoercionResult {
                    kind,
                    target_type: to.clone(),
                    is_safe,
                    cost,
                }));
            }
        }

        Ok(None)
    }

    /// Get information about numeric conversions.
    fn numeric_conversion_info(&self, from: &PrimitiveType, to: &PrimitiveType) -> Option<(CoercionKind, CoercionCost, bool)> {
        use PrimitiveType::*;

        match (from, to) {
            // Integer widening (safe)
            (I8, I16) | (I8, I32) | (I8, I64) | (I8, I128) =>
                Some((CoercionKind::Numeric, CoercionCost::Low, true)),
            (I16, I32) | (I16, I64) | (I16, I128) =>
                Some((CoercionKind::Numeric, CoercionCost::Low, true)),
            (I32, I64) | (I32, I128) =>
                Some((CoercionKind::Numeric, CoercionCost::Low, true)),
            (I64, I128) =>
                Some((CoercionKind::Numeric, CoercionCost::Low, true)),

            (U8, U16) | (U8, U32) | (U8, U64) | (U8, U128) =>
                Some((CoercionKind::Numeric, CoercionCost::Low, true)),
            (U16, U32) | (U16, U64) | (U16, U128) =>
                Some((CoercionKind::Numeric, CoercionCost::Low, true)),
            (U32, U64) | (U32, U128) =>
                Some((CoercionKind::Numeric, CoercionCost::Low, true)),
            (U64, U128) =>
                Some((CoercionKind::Numeric, CoercionCost::Low, true)),

            // Float widening (safe)
            (F32, F64) =>
                Some((CoercionKind::Numeric, CoercionCost::Low, true)),

            // Integer to float (potentially lossy for large integers)
            (I8, F32) | (I8, F64) | (I16, F32) | (I16, F64) =>
                Some((CoercionKind::Numeric, CoercionCost::Low, true)),
            (I32, F32) | (I32, F64) | (I64, F64) =>
                Some((CoercionKind::Numeric, CoercionCost::Medium, false)),

            (U8, F32) | (U8, F64) | (U16, F32) | (U16, F64) =>
                Some((CoercionKind::Numeric, CoercionCost::Low, true)),
            (U32, F32) | (U32, F64) | (U64, F64) =>
                Some((CoercionKind::Numeric, CoercionCost::Medium, false)),

            // Size types to appropriate integer types
            (ISize, I64) | (USize, U64) =>
                Some((CoercionKind::Numeric, CoercionCost::Free, true)),

            _ => None,
        }
    }

    /// Try reference coercion.
    fn try_reference_coercion(&self, from: &Type, to: &Type) -> Result<Option<CoercionResult>> {
        match (&from.kind, &to.kind) {
            // &T to *const T
            (TypeKind::Reference { target: from_target, mutable: false, .. },
                TypeKind::Pointer { target: to_target, mutable: false }) => {
                if self.types_identical(from_target, to_target) {
                    return Ok(Some(CoercionResult {
                        kind: CoercionKind::Reference,
                        target_type: to.clone(),
                        is_safe: false, // Pointer conversion is unsafe
                        cost: CoercionCost::High,
                    }));
                }
            }

            // &mut T to *mut T
            (TypeKind::Reference { target: from_target, mutable: true, .. },
                TypeKind::Pointer { target: to_target, mutable: true }) => {
                if self.types_identical(from_target, to_target) {
                    return Ok(Some(CoercionResult {
                        kind: CoercionKind::Reference,
                        target_type: to.clone(),
                        is_safe: false, // Pointer conversion is unsafe
                        cost: CoercionCost::High,
                    }));
                }
            }

            // &mut T to *const T
            (TypeKind::Reference { target: from_target, mutable: true, .. },
                TypeKind::Pointer { target: to_target, mutable: false }) => {
                if self.types_identical(from_target, to_target) {
                    return Ok(Some(CoercionResult {
                        kind: CoercionKind::Reference,
                        target_type: to.clone(),
                        is_safe: false, // Pointer conversion is unsafe
                        cost: CoercionCost::High,
                    }));
                }
            }

            _ => {}
        }

        Ok(None)
    }

    /// Try array to slice coercion.
    fn try_array_to_slice_coercion(&self, from: &Type, to: &Type) -> Result<Option<CoercionResult>> {
        match (&from.kind, &to.kind) {
            // [T; N] to [T]
            (TypeKind::Array { element: from_elem, .. },
                TypeKind::Slice { element: to_elem }) => {
                if self.types_identical(from_elem, to_elem) {
                    return Ok(Some(CoercionResult {
                        kind: CoercionKind::ArrayToSlice,
                        target_type: to.clone(),
                        is_safe: true,
                        cost: CoercionCost::Free,
                    }));
                }
            }

            _ => {}
        }

        Ok(None)
    }

    /// Try function coercion.
    fn try_function_coercion(&self, from: &Type, to: &Type) -> Result<Option<CoercionResult>> {
        match (&from.kind, &to.kind) {
            // Function types with identical signatures
            (TypeKind::Function { params: from_params, return_type: from_ret, .. },
                TypeKind::Function { params: to_params, return_type: to_ret, .. }) => {
                if from_params.len() == to_params.len() &&
                    self.types_identical(from_ret, to_ret) &&
                    from_params.iter().zip(to_params.iter()).all(|(a, b)| self.types_identical(a, b)) {
                    return Ok(Some(CoercionResult {
                        kind: CoercionKind::FunctionItem,
                        target_type: to.clone(),
                        is_safe: true,
                        cost: CoercionCost::Free,
                    }));
                }
            }

            _ => {}
        }

        Ok(None)
    }

    /// Find common supertype of two types.
    fn find_common_supertype(&self, a: &Type, b: &Type, span: SourceSpan) -> Result<Type> {
        // If types are identical, return either one
        if self.types_identical(a, b) {
            return Ok(a.clone());
        }

        // Try coercing a to b
        if self.can_coerce(a, b) {
            return Ok(b.clone());
        }

        // Try coercing b to a
        if self.can_coerce(b, a) {
            return Ok(a.clone());
        }

        // Special cases for common supertypes
        match (&a.kind, &b.kind) {
            // Numeric types - find the "largest" type that can represent both
            (TypeKind::Primitive(prim_a), TypeKind::Primitive(prim_b)) => {
                if let Some(common_prim) = self.find_common_numeric_type(prim_a, prim_b) {
                    return Ok(Type::new(TypeKind::Primitive(common_prim), span));
                }
            }

            // References with different mutability - prefer immutable
            (TypeKind::Reference { target: target_a, mutable: mut_a, lifetime: life_a },
                TypeKind::Reference { target: target_b, mutable: mut_b, lifetime: life_b }) => {
                if self.types_identical(target_a, target_b) && life_a == life_b {
                    return Ok(Type::new(TypeKind::Reference {
                        target: target_a.clone(),
                        mutable: *mut_a && *mut_b, // Both must be mutable for result to be mutable
                        lifetime: life_a.clone(),
                    }, span));
                }
            }

            _ => {}
        }

        // No common supertype found
        Err(TlError::type_error(
            self.source.clone(),
            span,
            format!("No common supertype for {:?} and {:?}", a.kind, b.kind),
        ))
    }

    /// Find common numeric type.
    fn find_common_numeric_type(&self, a: &PrimitiveType, b: &PrimitiveType) -> Option<PrimitiveType> {
        use PrimitiveType::*;

        // If types are the same, return that type
        if a == b {
            return Some(*a);
        }

        // Define a hierarchy for numeric types
        let int_hierarchy = [I8, I16, I32, I64, I128];
        let uint_hierarchy = [U8, U16, U32, U64, U128];
        let float_hierarchy = [F32, F64];

        // Find positions in hierarchies
        let a_int_pos = int_hierarchy.iter().position(|&t| t == *a);
        let b_int_pos = int_hierarchy.iter().position(|&t| t == *b);
        let a_uint_pos = uint_hierarchy.iter().position(|&t| t == *a);
        let b_uint_pos = uint_hierarchy.iter().position(|&t| t == *b);
        let a_float_pos = float_hierarchy.iter().position(|&t| t == *a);
        let b_float_pos = float_hierarchy.iter().position(|&t| t == *b);

        // Both are signed integers
        if let (Some(pos_a), Some(pos_b)) = (a_int_pos, b_int_pos) {
            return Some(int_hierarchy[pos_a.max(pos_b)]);
        }

        // Both are unsigned integers
        if let (Some(pos_a), Some(pos_b)) = (a_uint_pos, b_uint_pos) {
            return Some(uint_hierarchy[pos_a.max(pos_b)]);
        }

        // Both are floats
        if let (Some(pos_a), Some(pos_b)) = (a_float_pos, b_float_pos) {
            return Some(float_hierarchy[pos_a.max(pos_b)]);
        }

        // Mixed integer types - prefer signed if possible
        if a_int_pos.is_some() && b_uint_pos.is_some() {
            // Try to find a signed type that can represent the unsigned type
            if let Some(uint_pos) = b_uint_pos {
                if uint_pos < int_hierarchy.len() - 1 {
                    return Some(int_hierarchy[uint_pos + 1]);
                }
            }
        }

        if a_uint_pos.is_some() && b_int_pos.is_some() {
            if let Some(uint_pos) = a_uint_pos {
                if uint_pos < int_hierarchy.len() - 1 {
                    return Some(int_hierarchy[uint_pos + 1]);
                }
            }
        }

        // Integer to float - use appropriate float type
        if (a_int_pos.is_some() || a_uint_pos.is_some()) && b_float_pos.is_some() {
            return Some(*b);
        }

        if (b_int_pos.is_some() || b_uint_pos.is_some()) && a_float_pos.is_some() {
            return Some(*a);
        }

        // No common type found
        None
    }
}

impl CoercionResult {
    /// Check if this coercion is better than another for overload resolution.
    pub fn is_better_than(&self, other: &CoercionResult) -> bool {
        match self.cost.cmp(&other.cost) {
            std::cmp::Ordering::Less => true,
            std::cmp::Ordering::Greater => false,
            std::cmp::Ordering::Equal => {
                // If costs are equal, prefer safe coercions
                self.is_safe && !other.is_safe
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::types::*;

    fn i32_type() -> Type {
        Type::new(TypeKind::Primitive(PrimitiveType::I32), SourceSpan::new(0.into(), 0))
    }

    fn i64_type() -> Type {
        Type::new(TypeKind::Primitive(PrimitiveType::I64), SourceSpan::new(0.into(), 0))
    }

    fn f64_type() -> Type {
        Type::new(TypeKind::Primitive(PrimitiveType::F64), SourceSpan::new(0.into(), 0))
    }

    #[test]
    fn test_identity_coercion() {
        let rules = CoercionRules::new("test".to_string());
        let i32_a = i32_type();
        let i32_b = i32_type();

        let result = rules.try_coerce(&i32_a, &i32_b, SourceSpan::new(0.into(), 0)).unwrap();
        assert_eq!(result.kind, CoercionKind::Identity);
        assert_eq!(result.cost, CoercionCost::Free);
    }

    #[test]
    fn test_numeric_coercion() {
        let rules = CoercionRules::new("test".to_string());
        let i32_t = i32_type();
        let i64_t = i64_type();

        let result = rules.try_coerce(&i32_t, &i64_t, SourceSpan::new(0.into(), 0)).unwrap();
        assert_eq!(result.kind, CoercionKind::Numeric);
        assert!(result.is_safe);
        assert_eq!(result.cost, CoercionCost::Low);
    }

    #[test]
    fn test_common_type() {
        let rules = CoercionRules::new("test".to_string());
        let types = vec![i32_type(), i64_type()];

        let common = rules.find_common_type(&types, SourceSpan::new(0.into(), 0)).unwrap();
        assert_eq!(common.kind, TypeKind::Primitive(PrimitiveType::I64));
    }

    #[test]
    fn test_coercion_failure() {
        let rules = CoercionRules::new("test".to_string());
        let i32_t = i32_type();
        let f64_t = f64_type();

        // i32 to f64 should work
        assert!(rules.can_coerce(&i32_t, &f64_t));

        // f64 to i32 should not work (lossy)
        assert!(!rules.can_coerce(&f64_t, &i32_t));
    }
}