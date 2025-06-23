// shared/src/ast/types.rs
//! Type system AST nodes for T-Lang.
//! Designed for safety-critical systems with explicit ownership and lifetimes.

use miette::SourceSpan;
use serde::{Deserialize, Serialize};

/// A complete type in the T-Lang type system.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Type {
    pub kind: TypeKind,
    pub span: SourceSpan,
}

/// The different kinds of types in T-Lang.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeKind {
    /// Primitive types: i32, f64, bool, char, etc.
    Primitive(PrimitiveType),

    /// Array with compile-time known size: [T; N]
    Array {
        element: Box<Type>,
        size: ArraySize,
    },

    /// Slice with runtime size: [T]
    Slice {
        element: Box<Type>,
    },

    /// Reference with optional lifetime: &'a T or &T
    Reference {
        target: Box<Type>,
        lifetime: Option<Lifetime>,
        mutable: bool,
    },

    /// Pointer (unsafe): *const T or *mut T
    Pointer {
        target: Box<Type>,
        mutable: bool,
    },

    /// Function type: fn(T1, T2) -> T3
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
        safety: SafetyLevel,
    },

    /// Tuple: (T1, T2, T3)
    Tuple(Vec<Type>),

    /// Named type: struct, enum, or type alias
    Named {
        path: Vec<String>,
        generics: Vec<Type>,
    },

    /// Generic type parameter: T, U, etc.
    Generic {
        name: String,
        bounds: Vec<TypeBound>,
    },

    /// Associated type: Self::Item
    Associated {
        base: Box<Type>,
        name: String,
    },

    /// Never type: !
    Never,

    /// Unknown type (for inference)
    Unknown(u32), // ID for type inference
}

/// Primitive types in T-Lang.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveType {
    // Signed integers
    I8, I16, I32, I64, I128, ISize,

    // Unsigned integers
    U8, U16, U32, U64, U128, USize,

    // Floating point
    F32, F64,

    // Other primitives
    Bool,
    Char,
    Str,
    Unit, // ()
}

/// Array size specification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArraySize {
    /// Literal number: [T; 5]
    Literal(u64),

    /// Constant expression: [T; N]
    Const(String),

    /// Inferred from context: [T; _]
    Inferred,
}

/// Lifetime annotations for references.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Lifetime {
    pub name: String,
    pub span: SourceSpan,
}

/// Safety levels for functions and operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SafetyLevel {
    /// Memory safe, no undefined behavior possible
    Safe,

    /// Requires unsafe block, manual verification
    Unsafe,

    /// Real-time safe, bounded execution time
    Realtime,

    /// Hardware verified, formal proofs required
    Critical,
}

/// Type bounds for generics: T: Clone + Send
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeBound {
    pub trait_path: Vec<String>,
    pub span: SourceSpan,
}

impl Type {
    /// Create a new type with the given kind and span.
    pub fn new(kind: TypeKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    /// Create a primitive type.
    pub fn primitive(prim: PrimitiveType, span: SourceSpan) -> Self {
        Self::new(TypeKind::Primitive(prim), span)
    }

    /// Create a reference type.
    pub fn reference(target: Type, lifetime: Option<Lifetime>, mutable: bool) -> Self {
        let span = target.span;
        Self::new(TypeKind::Reference {
            target: Box::new(target),
            lifetime,
            mutable,
        }, span)
    }

    /// Create a function type.
    pub fn function(params: Vec<Type>, return_type: Type, safety: SafetyLevel) -> Self {
        let span = return_type.span;
        Self::new(TypeKind::Function {
            params,
            return_type: Box::new(return_type),
            safety,
        }, span)
    }

    /// Check if this type is considered safe.
    pub fn is_safe(&self) -> bool {
        match &self.kind {
            TypeKind::Pointer { .. } => false,
            TypeKind::Function { safety, .. } => matches!(safety, SafetyLevel::Safe | SafetyLevel::Realtime | SafetyLevel::Critical),
            _ => true,
        }
    }

    /// Check if this type can be copied.
    pub fn is_copy(&self) -> bool {
        match &self.kind {
            TypeKind::Primitive(_) => true,
            TypeKind::Reference { .. } => true,
            TypeKind::Pointer { .. } => true,
            TypeKind::Tuple(types) => types.iter().all(|t| t.is_copy()),
            _ => false,
        }
    }
}