
// shared/src/ast/stmt.rs
//! Statement AST nodes for T-Lang.
//! Comprehensive statement system for all programming paradigms.

use super::{
    expr::{Expr, Pattern},
    types::{Type, SafetyLevel},
};
use miette::SourceSpan;
use serde::{Deserialize, Serialize};

/// A statement in T-Lang.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: SourceSpan,
}

/// All possible statement kinds in T-Lang.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StmtKind {
    /// Expression statement: expr;
    Expr(Expr),

    /// Let binding: let pat: Type = expr;
    Let {
        pattern: Pattern,
        ty: Option<Type>,
        initializer: Option<Expr>,
        mutable: bool,
    },

    /// Item definition (function, struct, enum, etc.)
    Item(Item),

    /// Macro invocation: macro_name!(args);
    Macro {
        path: Vec<String>,
        args: Vec<MacroArg>,
    },
}

/// Top-level items (functions, types, modules, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub span: SourceSpan,
}

/// All possible item kinds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ItemKind {
    /// Function definition: fn name(params) -> RetType { body }
    Function {
        name: String,
        generics: Vec<GenericParam>,
        params: Vec<FnParam>,
        return_type: Option<Type>,
        body: Option<Expr>, // None for declarations
        safety: SafetyLevel,
        async_: bool,
        const_: bool,
    },

    /// Struct definition: struct Name { fields }
    Struct {
        name: String,
        generics: Vec<GenericParam>,
        fields: StructFields,
    },

    /// Enum definition: enum Name { variants }
    Enum {
        name: String,
        generics: Vec<GenericParam>,
        variants: Vec<EnumVariant>,
    },

    /// Union definition: union Name { fields }
    Union {
        name: String,
        generics: Vec<GenericParam>,
        fields: Vec<StructField>,
    },

    /// Trait definition: trait Name { items }
    Trait {
        name: String,
        generics: Vec<GenericParam>,
        supertraits: Vec<Type>,
        items: Vec<TraitItem>,
        safety: SafetyLevel,
    },

    /// Implementation: impl [Trait for] Type { items }
    Impl {
        generics: Vec<GenericParam>,
        trait_: Option<Type>,
        self_ty: Type,
        items: Vec<ImplItem>,
        safety: SafetyLevel,
    },

    /// Type alias: type Name = Type;
    TypeAlias {
        name: String,
        generics: Vec<GenericParam>,
        ty: Type,
    },

    /// Constant: const NAME: Type = expr;
    Const {
        name: String,
        ty: Type,
        value: Expr,
    },

    /// Static variable: static NAME: Type = expr;
    Static {
        name: String,
        ty: Type,
        value: Expr,
        mutable: bool,
    },

    /// Module: mod name { items }
    Module {
        name: String,
        items: Vec<Item>,
        inline: bool, // true for mod name { }, false for mod name;
    },

    /// Use declaration: use path::to::item;
    Use {
        path: Vec<String>,
        alias: Option<String>,
        glob: bool, // for use path::*;
    },

    /// External block: extern "C" { items }
    Extern {
        abi: Option<String>,
        items: Vec<ExternItem>,
    },

    /// Macro definition: macro_rules! name { rules }
    Macro {
        name: String,
        rules: Vec<MacroRule>,
    },
}

/// Struct field definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StructFields {
    Named(Vec<StructField>),     // struct S { x: i32, y: i32 }
    Unnamed(Vec<Type>),          // struct S(i32, i32);
    Unit,                        // struct S;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub vis: Visibility,
    pub attrs: Vec<Attribute>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub fields: StructFields,
    pub discriminant: Option<Expr>,
    pub attrs: Vec<Attribute>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FnParam {
    pub pattern: Pattern,
    pub ty: Type,
    pub default: Option<Expr>,
    pub attrs: Vec<Attribute>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<Type>,
    pub default: Option<Type>,
    pub span: SourceSpan,
}

/// Trait items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TraitItem {
    Function {
        name: String,
        generics: Vec<GenericParam>,
        params: Vec<FnParam>,
        return_type: Option<Type>,
        body: Option<Expr>,
        safety: SafetyLevel,
    },
    Type {
        name: String,
        bounds: Vec<Type>,
        default: Option<Type>,
    },
    Const {
        name: String,
        ty: Type,
        value: Option<Expr>,
    },
}

/// Implementation items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImplItem {
    Function {
        name: String,
        generics: Vec<GenericParam>,
        params: Vec<FnParam>,
        return_type: Option<Type>,
        body: Expr,
        safety: SafetyLevel,
        vis: Visibility,
    },
    Type {
        name: String,
        ty: Type,
        vis: Visibility,
    },
    Const {
        name: String,
        ty: Type,
        value: Expr,
        vis: Visibility,
    },
}

/// External items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExternItem {
    Function {
        name: String,
        params: Vec<FnParam>,
        return_type: Option<Type>,
        variadic: bool,
    },
    Static {
        name: String,
        ty: Type,
        mutable: bool,
    },
}

/// Visibility levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    Public,                      // pub
    PublicCrate,                 // pub(crate)
    PublicSuper,                 // pub(super)
    PublicIn(Vec<String>),       // pub(in path::to::module)
    Private,                     // (default)
}

/// Attributes like #[derive(Debug)], #[inline].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub path: Vec<String>,
    pub args: Vec<AttributeArg>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeArg {
    Literal(super::expr::Literal),
    Ident(String),
    List(Vec<AttributeArg>),
}

/// Macro-related types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacroArg {
    pub tokens: Vec<String>, // Simplified for now
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacroRule {
    pub pattern: Vec<String>, // Simplified for now
    pub body: Vec<String>,
    pub span: SourceSpan,
}

impl Stmt {
    pub fn new(kind: StmtKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    pub fn expr(expr: Expr) -> Self {
        let span = expr.span;
        Self::new(StmtKind::Expr(expr), span)
    }
}

impl Item {
    pub fn new(kind: ItemKind, span: SourceSpan) -> Self {
        Self {
            kind,
            attrs: Vec::new(),
            vis: Visibility::Private,
            span,
        }
    }

    pub fn with_visibility(mut self, vis: Visibility) -> Self {
        self.vis = vis;
        self
    }

    pub fn with_attrs(mut self, attrs: Vec<Attribute>) -> Self {
        self.attrs = attrs;
        self
    }
}