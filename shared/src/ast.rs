//! Central AST definitions for T‑Lang: items, modules, types, patterns,
//! expressions, statements, metadata, and extensibility hooks.

use std::collections::HashMap;
use bitflags::bitflags;
use crate::token::Token;
use serde::{Serialize, Deserialize};

/// A byte‑range within the source file.
#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub len: usize,
}

/// Top‑level AST root.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AST {
    pub src: String,
    pub items: Vec<Item>,
    pub spans: Vec<Span>,
}

impl AST {
    pub fn new(src: String, items: Vec<Item>, spans: Vec<Span>) -> Self {
        AST { src, items, spans }
    }
}

/// Plugin hook: any future extension can attach custom data here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Extension {
    Unknown,               // placeholder
    Custom(String, Vec<u8>), // name + raw payload
}

/// Attributes and metadata (docs, lints, energy hints, deprecation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub path: Vec<String>,
    pub args: Vec<MetaItem>,
    pub span: Span,
}

// … other imports …

/// Key/value metadata in attributes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetaItem { /* … */ }

bitflags! {
    /// bitflags for effects on expressions (pure, io, async, unsafe, etc.)
    pub struct Effects: u8 {
        const PURE   = 0b00000001;
        const IO     = 0b00000010;
        const ASYNC  = 0b00000100;
        const UNSAFE = 0b00001000;
    }
}

impl Default for Effects {
    fn default() -> Self {
        Effects::empty()
    }
}


/// Top‑level items in a module or file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Module {
        name: String,
        attrs: Vec<Attribute>,
        items: Vec<Item>,
        span: Span,
    },
    Use {
        path: Vec<String>,
        alias: Option<String>,
        attrs: Vec<Attribute>,
        span: Span,
    },
    Function {
        attrs: Vec<Attribute>,
        signature: FnSignature,
        body: Block,
        span: Span,
    },
    Struct {
        attrs: Vec<Attribute>,
        name: String,
        generics: Generics,
        fields: Vec<StructField>,
        span: Span,
    },
    Enum {
        attrs: Vec<Attribute>,
        name: String,
        generics: Generics,
        variants: Vec<EnumVariant>,
        span: Span,
    },
    Trait {
        attrs: Vec<Attribute>,
        name: String,
        generics: Generics,
        items: Vec<Item>,
        span: Span,
    },
    Impl {
        attrs: Vec<Attribute>,
        generics: Generics,
        trait_ref: Option<TypeRef>,
        for_type: TypeRef,
        items: Vec<Item>,
        span: Span,
    },
    ExternBlock {
        attrs: Vec<Attribute>,
        abi: String,                // e.g. "C", "Swift", "Mojo"
        linkage: Option<String>,    // optional DLL/shared flags
        items: Vec<Item>,
        span: Span,
    },
    Const {
        attrs: Vec<Attribute>,
        name: String,
        ty: TypeRef,
        expr: Expr,
        span: Span,
    },
    Static {
        attrs: Vec<Attribute>,
        name: String,
        mutability: bool,
        ty: TypeRef,
        expr: Expr,
        span: Span,
    },
    TypeAlias {
        attrs: Vec<Attribute>,
        name: String,
        generics: Generics,
        ty: TypeRef,
        span: Span,
    },
    MacroDef {
        attrs: Vec<Attribute>,
        name: String,
        args: Vec<String>,
        body: TokenStream,
        span: Span,
    },
    Extension(Extension),
}

/// Function signature (name, params, return, generics, async/unsafe).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FnSignature {
    pub name: String,
    pub generics: Generics,
    pub params: Vec<Param>,
    pub return_ty: Option<TypeRef>,
    pub is_async: bool,
    pub is_unsafe: bool,
    pub span: Span,
}

/// A function parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub pat: Pattern,
    pub ty: TypeRef,
    pub span: Span,
}

/// Generic parameters (lifetimes, types, consts).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Generics {
    pub lifetimes: Vec<String>,
    pub type_params: Vec<GenericParam>,
    pub const_params: Vec<String>,
}

/// Single generic type parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TypeRef>,
}

/// A struct field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub attrs: Vec<Attribute>,
    pub name: Option<String>,
    pub ty: TypeRef,
    pub span: Span,
}

/// An enum variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub attrs: Vec<Attribute>,
    pub name: String,
    pub fields: EnumFields,
    pub span: Span,
}

/// Variant data shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnumFields {
    Unit,
    Tuple(Vec<TypeRef>),
    Named(Vec<StructField>),
}

/// Token tree for macros.
pub type TokenStream = Vec<Token>;

/// Literal kinds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Integer(i128),
    Float(f64),
    Boolean(bool),
    String(String),
    Char(char),
    Byte(u8),
    ByteString(Vec<u8>),
    RawString(String),
}

/// Type references, with explicit ownership/borrowing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeRef {
    Infer,                                     // `_`
    Path(Vec<String>),                         // `std::io::Result`
    Generic(String),                           // `T`
    Function(Vec<TypeRef>, Box<TypeRef>),      // `(A,B)->C`
    Tuple(Vec<TypeRef>),                       // `(A,B,C)`
    Array(Box<TypeRef>, Option<usize>),        // `[T; N]` or `[T]`
    Reference {                                // `&T` or `&mut T`
        is_mut: bool,
        inner: Box<TypeRef>,
    },
    Pointer {                                  // `*const T` or `*mut T`
        is_mut: bool,
        inner: Box<TypeRef>,
    },
    Owned(Box<TypeRef>),                       // T by value
    Borrowed(Box<TypeRef>),                    // &T
    BorrowedMut(Box<TypeRef>),                 // &mut T
    ImplTrait(Vec<TypeRef>),                   // `impl Trait+...`
    DynTrait(Vec<TypeRef>),                    // `dyn Trait+...`
    Never,                                     // `!`
    Unit,                                      // `()`
}

/// Pattern matching.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Wildcard,                                  // `_`
    Literal(Literal),
    Identifier {
        name: String,
        capture: Option<CaptureBy>,           // Owned/Ref/Mut
        span: Span,
    },
    Reference {
        is_mut: bool,
        pat: Box<Pattern>,
    },
    Struct {
        path: Vec<String>,
        fields: HashMap<String, Pattern>,
        rest: bool,
    },
    Tuple(Vec<Pattern>),
    TupleStruct {
        path: Vec<String>,
        elems: Vec<Pattern>,
    },
    Slice {
        front: Vec<Pattern>,
        rest: Option<Box<Pattern>>,
        back: Vec<Pattern>,
    },
    Or(Vec<Pattern>),
    Range {
        start: Option<Literal>,
        end: Option<Literal>,
        inclusive: bool,
    },
    Binding {
        name: String,
        subpat: Option<Box<Pattern>>,
    },
    Macro(Vec<String>, TokenStream),
}

/// A block of statements, possibly ending in an expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub expr: Option<Box<Expr>>,
    pub span: Span,
}

/// Statements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    Local {
        pat: Pattern,
        ty: Option<TypeRef>,
        init: Option<Expr>,
        attrs: Vec<Attribute>,
        span: Span,
    },
    Expr {
        expr: Expr,
        span: Span,
    },
    Semi {
        expr: Expr,
        span: Span,
    },
    Item(Item),
    Macro(Vec<String>, TokenStream, Span),
    Extension(Extension),
}

/// Expressions, each carries an `Effects` mask for purity/IO/async/etc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expr {
    pub kind: ExprKind,
    pub effects: Effects,
}

/// The variant of expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExprKind {
    Literal(Literal, Span),
    Variable(Vec<String>, Span),
    Grouping(Box<Expr>, Span),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        span: Span,
    },
    If {
        cond: Box<Expr>,
        then_branch: Block,
        else_branch: Option<Box<Expr>>,
        span: Span,
    },
    While {
        cond: Box<Expr>,
        body: Block,
        is_loop: bool,
        span: Span,
    },
    For {
        pat: Pattern,
        iter: Box<Expr>,
        body: Block,
        span: Span,
    },
    Loop {
        body: Block,
        span: Span,
    },
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    Closure {
        capture_by: CaptureBy,
        params: Vec<Param>,
        return_ty: Option<TypeRef>,
        body: Box<Expr>,
        is_async: bool,
        is_move: bool,
        span: Span,
    },
    BlockExpr(Block),
    Async {
        capture_by: CaptureBy,
        block: Block,
        span: Span,
    },
    Await {
        expr: Box<Expr>,
        span: Span,
    },
    Try {
        expr: Box<Expr>,
        span: Span,
    },
    Cast {
        expr: Box<Expr>,
        ty: TypeRef,
        span: Span,
    },
    TypeAscription {
        expr: Box<Expr>,
        ty: TypeRef,
        span: Span,
    },
    Tuple(Vec<Expr>, Span),
    Array(Vec<Expr>, Span),
    Index {
        expr: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    Field {
        expr: Box<Expr>,
        name: String,
        span: Span,
    },
    Range {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
        span: Span,
    },
    StructLit {
        path: Vec<String>,
        fields: HashMap<String, Expr>,
        rest: Option<Box<Expr>>,
        span: Span,
    },
    MacroCall {
        path: Vec<String>,
        tokens: TokenStream,
        span: Span,
    },
    Extension(Extension),
}

/// Binary operators.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    And, Or,
    BitAnd, BitOr, BitXor, Shl, Shr,
    Eq, Ne, Lt, Le, Gt, Ge,
}

/// Unary operators.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg, Not, Deref,
}

/// How closures/async capture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CaptureBy {
    Ref,     // `|&x|`
    Mut,     // `|&mut x|`
    Value,   // `|x|`
}

/// One arm of a `match`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pat: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub attrs: Vec<Attribute>,
    pub span: Span,
}
