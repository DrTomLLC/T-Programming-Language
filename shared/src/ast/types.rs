// shared/src/ast/types.rs
use crate::{Span, span::HasSpan};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

impl HasSpan for Type {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeKind {
    Primitive(PrimitiveType),
    Path(Vec<String>),
    Tuple(Vec<Type>),
    Array(Box<Type>, Option<usize>),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    Generic {
        name: String,
        constraints: Vec<Type>,
    },
    Never,
    Infer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveType {
    Bool,
    I8, I16, I32, I64, I128, ISize,
    U8, U16, U32, U64, U128, USize,
    F32, F64,
    Char, Str, String, Unit,
}