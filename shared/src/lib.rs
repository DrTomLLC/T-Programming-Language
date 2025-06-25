// shared/src/lib.rs
//! Shared types and utilities for the T-Lang compiler.

use miette::SourceSpan;
use serde::{Deserialize, Serialize};

// Re-export error types
pub use errors::{TlError, ErrorCode, Result, ErrorCollector};

// Declare modules
pub mod ast;
pub mod token;
pub mod tokenizer;
pub mod tir;

// Re-export all AST types
pub use ast::*;
pub use token::*;
pub use tokenizer::*;

// Create a Span type that wraps SourceSpan with Default
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub len: usize,
}

impl Span {
    pub fn new(start: usize, len: usize) -> Self {
        Self { start, len }
    }

    pub fn end(&self) -> usize {
        self.start + self.len
    }
}

impl Default for Span {
    fn default() -> Self {
        Self { start: 0, len: 0 }
    }
}

impl From<Span> for SourceSpan {
    fn from(span: Span) -> Self {
        SourceSpan::from((span.start, span.len))
    }
}

impl From<SourceSpan> for Span {
    fn from(span: SourceSpan) -> Self {
        Self {
            start: span.offset(),
            len: span.len(),
        }
    }
}

impl From<(usize, usize)> for Span {
    fn from((start, len): (usize, usize)) -> Self {
        Self { start, len }
    }
}

// Define primitive types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveType {
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    ISize,
    U8,
    U16,
    U32,
    U64,
    U128,
    USize,
    F32,
    F64,
    Char,
    Str,
    String,
    Unit,
}

// Define type system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeKind {
    Primitive(PrimitiveType),
    Tuple(Vec<Type>),
    Array(Box<Type>, Option<usize>),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    Generic {
        name: String,
        args: Vec<Type>,
    },
    Reference {
        inner: Box<Type>,
        mutable: bool,
    },
    Path(Vec<String>),
    Inferred,
    Error,
}

// Define binary and unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    And,
    Or,

    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,

    // Assignment
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,
    Neg,
    Deref,
    Ref,
    RefMut,
}

// Define literals
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Char(char),
    String(String),
    Unit,
}

// Define expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExprKind {
    Literal(Literal),
    Identifier(String),
    Binary {
        lhs: Box<Expr>,
        op: BinaryOp,
        rhs: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Tuple(Vec<Expr>),
    Array(Vec<Expr>),
    Block(Block),
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Expr>,
    },
    For {
        pattern: Pattern,
        iterator: Box<Expr>,
        body: Box<Expr>,
    },
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    Return(Option<Box<Expr>>),
    Break(Option<Box<Expr>>),
    Continue,
    Closure {
        params: Vec<ClosureParam>,
        body: Box<Expr>,
    },
    Path(Vec<String>),
}

// Define patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternKind {
    Wildcard,
    Identifier(String),
    Literal(Literal),
    Tuple(Vec<Pattern>),
    Array(Vec<Pattern>),
    Struct {
        name: String,
        fields: Vec<(String, Pattern)>,
    },
    Enum {
        name: String,
        variant: String,
        data: Option<Box<Pattern>>,
    },
    Reference(Box<Pattern>),
    Range {
        start: Option<Box<Pattern>>,
        end: Option<Box<Pattern>>,
        inclusive: bool,
    },
}

// Define statements and blocks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StmtKind {
    Local {
        pattern: Pattern,
        ty: Option<Type>,
        init: Option<Expr>,
    },
    Item(Item),
    Expr(Expr),
    Semi(Expr),
}

// Define items
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub span: Span,
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
    Use {
        path: Vec<String>,
        alias: Option<String>,
        glob: bool,
    },
    Const {
        name: String,
        ty: Type,
        init: Expr,
    },
    Static {
        name: String,
        ty: Type,
        init: Expr,
        mutable: bool,
    },
    Module {
        name: String,
        items: Vec<Item>,
    },
}

// Supporting types for items
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<String>,
    pub default: Option<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FnParam {
    pub pattern: Pattern,
    pub ty: Type,
    pub default: Option<Expr>,
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
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub vis: Visibility,
    pub attrs: Vec<Attribute>,
    pub span: Span,
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
    pub args: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Crate,
    Module(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyLevel {
    Safe,
    Unsafe,
}

// Additional supporting types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClosureParam {
    pub pattern: Pattern,
    pub ty: Option<Type>,
    pub span: Span,
}

// Define the main Program type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: Span,
}

// Implement constructors for convenience
impl Type {
    pub fn primitive(prim: PrimitiveType, span: Span) -> Self {
        Self {
            kind: TypeKind::Primitive(prim),
            span,
        }
    }

    pub fn path(path: Vec<String>, span: Span) -> Self {
        Self {
            kind: TypeKind::Path(path),
            span,
        }
    }
}

impl Pattern {
    pub fn identifier(name: String, span: Span) -> Self {
        Self {
            kind: PatternKind::Identifier(name),
            span,
        }
    }

    pub fn wildcard(span: Span) -> Self {
        Self {
            kind: PatternKind::Wildcard,
            span,
        }
    }
}

impl Expr {
    pub fn literal(lit: Literal, span: Span) -> Self {
        Self {
            kind: ExprKind::Literal(lit),
            span,
        }
    }

    pub fn identifier(name: String, span: Span) -> Self {
        Self {
            kind: ExprKind::Identifier(name),
            span,
        }
    }
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Private
    }
}

impl Default for SafetyLevel {
    fn default() -> Self {
        Self::Safe
    }
}