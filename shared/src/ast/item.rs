// shared/src/ast/item.rs
//! Item definitions for T-Lang AST.

use crate::{Span, span::HasSpan};
use super::{Expr, Type, Pattern, MacroArg};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub span: Span,
}

impl HasSpan for Item {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ItemKind {
    Function {
        name: String,
        generics: Vec<GenericParam>,
        params: Vec<FnParam>,
        return_type: Option<Type>,
        body: Expr,
        safety: SafetyLevel,
    },
    Struct {
        name: String,
        generics: Vec<GenericParam>,
        fields: StructFields,
    },
    Enum {
        name: String,
        generics: Vec<GenericParam>,
        variants: Vec<EnumVariant>,
    },
    Const {
        name: String,
        ty: Type,
        init: Expr,  // Fixed: was 'value'
    },
    Static {
        name: String,
        ty: Type,
        init: Expr,  // Fixed: was 'value'
        mutable: bool,
    },
    TypeAlias {
        name: String,
        generics: Vec<GenericParam>,
        ty: Type,
    },
    Use {
        path: Vec<String>,
        alias: Option<String>,
        glob: bool,
    },
    Module {
        name: String,
        items: Vec<Item>,
        inline: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SafetyLevel {
    Safe,
    Unsafe,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FnParam {
    pub pattern: Pattern,
    pub ty: Type,
    pub default: Option<Expr>,
    pub attrs: Vec<Attribute>,
    pub span: Span,
}

// Alias for TIR compatibility
pub type FunctionParam = FnParam;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<Type>,
    pub default: Option<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub vis: Visibility,
    pub attrs: Vec<Attribute>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StructFields {
    Named(Vec<StructField>),
    Tuple(Vec<Type>),
    Unit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub data: EnumVariantData,
    pub attrs: Vec<Attribute>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnumVariantData {
    Unit,
    Tuple(Vec<Type>),
    Struct(Vec<StructField>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub path: Vec<String>,
    pub args: Vec<MacroArg>,
    pub span: Span,
}