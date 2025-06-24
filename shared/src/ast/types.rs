// shared/src/ast/types.rs
//! Type system AST nodes for T-Lang.

use miette::SourceSpan;
use serde::{Deserialize, Serialize};

/// A type in T-Lang.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Type {
    pub kind: TypeKind,
    pub span: SourceSpan,
}

/// All possible type kinds in T-Lang.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeKind {
    /// Primitive types: i32, f64, bool, etc.
    Primitive(PrimitiveType),

    /// Path types: std::vec::Vec, MyStruct, etc.
    Path {
        segments: Vec<String>,
        generics: Vec<Type>,
    },

    /// Reference types: &T, &mut T
    Reference {
        ty: Box<Type>,
        mutable: bool,
        lifetime: Option<String>,
    },

    /// Pointer types: *const T, *mut T
    Pointer {
        ty: Box<Type>,
        mutable: bool,
    },

    /// Array types: [T; N]
    Array {
        element: Box<Type>,
        size: Option<Box<super::Expr>>,
    },

    /// Slice types: [T]
    Slice {
        element: Box<Type>,
    },

    /// Tuple types: (T1, T2, ...)
    Tuple(Vec<Type>),

    /// Function types: fn(T1, T2) -> T3
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
        is_unsafe: bool,
        is_extern: bool,
        abi: Option<String>,
    },

    /// Generic parameter: T
    Generic(String),

    /// Associated type: T::Item
    Associated {
        ty: Box<Type>,
        name: String,
    },

    /// Trait object: dyn Trait
    TraitObject {
        traits: Vec<TraitBound>,
        lifetime: Option<String>,
    },

    /// Impl trait: impl Trait
    ImplTrait {
        bounds: Vec<TraitBound>,
    },

    /// Never type: !
    Never,

    /// Inferred type: _
    Infer,
}

/// Primitive types in T-Lang.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveType {
    // Boolean
    Bool,

    // Signed integers
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,

    // Unsigned integers
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,

    // Floating point
    F32,
    F64,

    // Character and string
    Char,
    Str,

    // Unit type
    Unit,
}

/// Trait bounds for types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitBound {
    pub path: Vec<String>,
    pub generics: Vec<Type>,
    pub span: SourceSpan,
}

/// Type parameter bounds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeBound {
    Trait(TraitBound),
    Lifetime(String),
}

impl Type {
    /// Create a primitive type.
    pub fn primitive(prim: PrimitiveType, span: SourceSpan) -> Self {
        Self {
            kind: TypeKind::Primitive(prim),
            span,
        }
    }

    /// Create a path type.
    pub fn path(segments: Vec<String>, span: SourceSpan) -> Self {
        Self {
            kind: TypeKind::Path {
                segments,
                generics: Vec::new(),
            },
            span,
        }
    }

    /// Create a simple identifier type.
    pub fn identifier(name: String, span: SourceSpan) -> Self {
        Self::path(vec![name], span)
    }

    /// Create a reference type.
    pub fn reference(ty: Type, mutable: bool, span: SourceSpan) -> Self {
        Self {
            kind: TypeKind::Reference {
                ty: Box::new(ty),
                mutable,
                lifetime: None,
            },
            span,
        }
    }

    /// Create a pointer type.
    pub fn pointer(ty: Type, mutable: bool, span: SourceSpan) -> Self {
        Self {
            kind: TypeKind::Pointer {
                ty: Box::new(ty),
                mutable,
            },
            span,
        }
    }

    /// Create an array type.
    pub fn array(element: Type, size: Option<super::Expr>, span: SourceSpan) -> Self {
        Self {
            kind: TypeKind::Array {
                element: Box::new(element),
                size: size.map(Box::new),
            },
            span,
        }
    }

    /// Create a slice type.
    pub fn slice(element: Type, span: SourceSpan) -> Self {
        Self {
            kind: TypeKind::Slice {
                element: Box::new(element),
            },
            span,
        }
    }

    /// Create a tuple type.
    pub fn tuple(types: Vec<Type>, span: SourceSpan) -> Self {
        Self {
            kind: TypeKind::Tuple(types),
            span,
        }
    }

    /// Create a function type.
    pub fn function(params: Vec<Type>, return_type: Type, span: SourceSpan) -> Self {
        Self {
            kind: TypeKind::Function {
                params,
                return_type: Box::new(return_type),
                is_unsafe: false,
                is_extern: false,
                abi: None,
            },
            span,
        }
    }

    /// Create a generic type.
    pub fn generic(name: String, span: SourceSpan) -> Self {
        Self {
            kind: TypeKind::Generic(name),
            span,
        }
    }

    /// Create an inferred type.
    pub fn infer(span: SourceSpan) -> Self {
        Self {
            kind: TypeKind::Infer,
            span,
        }
    }

    /// Create the unit type.
    pub fn unit(span: SourceSpan) -> Self {
        Self::primitive(PrimitiveType::Unit, span)
    }

    /// Create the bool type.
    pub fn bool(span: SourceSpan) -> Self {
        Self::primitive(PrimitiveType::Bool, span)
    }

    /// Create the i32 type.
    pub fn i32(span: SourceSpan) -> Self {
        Self::primitive(PrimitiveType::I32, span)
    }

    /// Create the f64 type.
    pub fn f64(span: SourceSpan) -> Self {
        Self::primitive(PrimitiveType::F64, span)
    }

    /// Create the str type.
    pub fn str(span: SourceSpan) -> Self {
        Self::primitive(PrimitiveType::Str, span)
    }

    /// Check if this is a primitive type.
    pub fn is_primitive(&self) -> bool {
        matches!(self.kind, TypeKind::Primitive(_))
    }

    /// Check if this is a reference type.
    pub fn is_reference(&self) -> bool {
        matches!(self.kind, TypeKind::Reference { .. })
    }

    /// Check if this is a mutable reference.
    pub fn is_mut_reference(&self) -> bool {
        matches!(self.kind, TypeKind::Reference { mutable: true, .. })
    }

    /// Check if this is a pointer type.
    pub fn is_pointer(&self) -> bool {
        matches!(self.kind, TypeKind::Pointer { .. })
    }

    /// Check if this is an array type.
    pub fn is_array(&self) -> bool {
        matches!(self.kind, TypeKind::Array { .. })
    }

    /// Check if this is a slice type.
    pub fn is_slice(&self) -> bool {
        matches!(self.kind, TypeKind::Slice { .. })
    }

    /// Check if this is a tuple type.
    pub fn is_tuple(&self) -> bool {
        matches!(self.kind, TypeKind::Tuple(_))
    }

    /// Check if this is a function type.
    pub fn is_function(&self) -> bool {
        matches!(self.kind, TypeKind::Function { .. })
    }

    /// Check if this is the unit type.
    pub fn is_unit(&self) -> bool {
        matches!(self.kind, TypeKind::Primitive(PrimitiveType::Unit))
    }

    /// Check if this is the never type.
    pub fn is_never(&self) -> bool {
        matches!(self.kind, TypeKind::Never)
    }

    /// Check if this is an inferred type.
    pub fn is_infer(&self) -> bool {
        matches!(self.kind, TypeKind::Infer)
    }

    /// Get the element type if this is an array or slice.
    pub fn element_type(&self) -> Option<&Type> {
        match &self.kind {
            TypeKind::Array { element, .. } | TypeKind::Slice { element } => Some(element),
            _ => None,
        }
    }

    /// Get the return type if this is a function type.
    pub fn return_type(&self) -> Option<&Type> {
        match &self.kind {
            TypeKind::Function { return_type, .. } => Some(return_type),
            _ => None,
        }
    }

    /// Get the parameter types if this is a function type.
    pub fn param_types(&self) -> Option<&[Type]> {
        match &self.kind {
            TypeKind::Function { params, .. } => Some(params),
            _ => None,
        }
    }

    /// Get the referenced type if this is a reference or pointer.
    pub fn deref_type(&self) -> Option<&Type> {
        match &self.kind {
            TypeKind::Reference { ty, .. } | TypeKind::Pointer { ty, .. } => Some(ty),
            _ => None,
        }
    }
}

impl PrimitiveType {
    /// Check if this is an integer type.
    pub fn is_integer(self) -> bool {
        matches!(
            self,
            PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32 | PrimitiveType::I64 |
            PrimitiveType::I128 | PrimitiveType::Isize | PrimitiveType::U8 | PrimitiveType::U16 |
            PrimitiveType::U32 | PrimitiveType::U64 | PrimitiveType::U128 | PrimitiveType::Usize
        )
    }

    /// Check if this is a signed integer type.
    pub fn is_signed_integer(self) -> bool {
        matches!(
            self,
            PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32 | PrimitiveType::I64 |
            PrimitiveType::I128 | PrimitiveType::Isize
        )
    }

    /// Check if this is an unsigned integer type.
    pub fn is_unsigned_integer(self) -> bool {
        matches!(
            self,
            PrimitiveType::U8 | PrimitiveType::U16 | PrimitiveType::U32 | PrimitiveType::U64 |
            PrimitiveType::U128 | PrimitiveType::Usize
        )
    }

    /// Check if this is a floating-point type.
    pub fn is_float(self) -> bool {
        matches!(self, PrimitiveType::F32 | PrimitiveType::F64)
    }

    /// Check if this is a numeric type (integer or float).
    pub fn is_numeric(self) -> bool {
        self.is_integer() || self.is_float()
    }

    /// Get the bit width of this type, if known.
    pub fn bit_width(self) -> Option<u32> {
        match self {
            PrimitiveType::Bool => Some(1),
            PrimitiveType::I8 | PrimitiveType::U8 => Some(8),
            PrimitiveType::I16 | PrimitiveType::U16 => Some(16),
            PrimitiveType::I32 | PrimitiveType::U32 | PrimitiveType::F32 | PrimitiveType::Char => Some(32),
            PrimitiveType::I64 | PrimitiveType::U64 | PrimitiveType::F64 => Some(64),
            PrimitiveType::I128 | PrimitiveType::U128 => Some(128),
            PrimitiveType::Isize | PrimitiveType::Usize => None, // Depends on target
            PrimitiveType::Str | PrimitiveType::Unit => None,
        }
    }

    /// Get the default value for this type as a string.
    pub fn default_value(self) -> &'static str {
        match self {
            PrimitiveType::Bool => "false",
            PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32 | PrimitiveType::I64 |
            PrimitiveType::I128 | PrimitiveType::Isize => "0",
            PrimitiveType::U8 | PrimitiveType::U16 | PrimitiveType::U32 | PrimitiveType::U64 |
            PrimitiveType::U128 | PrimitiveType::Usize => "0",
            PrimitiveType::F32 | PrimitiveType::F64 => "0.0",
            PrimitiveType::Char => "'\\0'",
            PrimitiveType::Str => "\"\"",
            PrimitiveType::Unit => "()",
        }
    }
}

impl TraitBound {
    pub fn new(path: Vec<String>, span: SourceSpan) -> Self {
        Self {
            path,
            generics: Vec::new(),
            span,
        }
    }

    pub fn simple(name: String, span: SourceSpan) -> Self {
        Self::new(vec![name], span)
    }
}