//! Central AST definitions for T‑Lang: items, modules, types, patterns,
//! expressions, statements, metadata, and extensibility hooks.

use std::collections::HashMap;
use crate::token::Token;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use enumflags2::{bitflags, BitFlags};

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
    pub stmts: ()
}

impl AST {
    /// Construct a new AST. Items are now Vec<Item>.
    pub fn new(src: String, items: Vec<Item>, spans: Vec<Span>) -> Self {
        AST { src, items, spans, stmts: () }
    }
}

/// Alias so that `Statement` refers to top‐level `Item`.
pub type Statement = Item;

/// Plugin hook: any future extension can attach custom data here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Extension {
    Unknown,               // placeholder
    Custom(String, Vec<u8>), // name + raw payload
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetaItem { /* … */ }

/// Attributes and metadata (docs, lints, energy hints, deprecation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub path: Vec<String>,
    pub args: Vec<MetaItem>,
    pub span: Span,
}

#[bitflags]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Effects on expressions (pure, io, async, unsafe, etc.)
pub enum Effects {
    Pure   = 0b0000_0001,
    Io     = 0b0000_0010,
    Async  = 0b0000_0100,
    Unsafe = 0b0000_1000,
}

// alias so your code stays the same:
pub type EffectsFlags = BitFlags<Effects>;

// Use a custom wrapper function instead of implementing Default directly
pub fn default_effects_flags() -> EffectsFlags {
    BitFlags::empty()
}

impl Default for Effects {
    fn default() -> Self {
        Effects::Pure
    }
}

// Manual serde impls for transparent u8:
impl Serialize for Effects {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> Deserialize<'de> for Effects {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bits = u8::deserialize(deserializer)?;
        let flags = BitFlags::<Effects>::from_bits_truncate(bits);
        if flags.is_empty() {
            Ok(Effects::Pure)
        } else if flags.contains(Effects::Pure) {
            Ok(Effects::Pure)
        } else if flags.contains(Effects::Io) {
            Ok(Effects::Io)
        } else if flags.contains(Effects::Async) {
            Ok(Effects::Async)
        } else {
            Ok(Effects::Unsafe)
        }
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
        abi: String,
        linkage: Option<String>,
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
    Infer,
    Path(Vec<String>),
    Generic(String),
    Function(Vec<TypeRef>, Box<TypeRef>),
    Tuple(Vec<TypeRef>),
    Array(Box<TypeRef>, Option<usize>),
    Reference { is_mut: bool, inner: Box<TypeRef> },
    Pointer   { is_mut: bool, inner: Box<TypeRef> },
    Owned     (Box<TypeRef>),
    Borrowed  (Box<TypeRef>),
    BorrowedMut(Box<TypeRef>),
    ImplTrait(Vec<TypeRef>),
    DynTrait(Vec<TypeRef>),
    Never,
    Unit,
}

/// How closures/async capture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CaptureBy {
    Ref,     // `|&x|`
    Mut,     // `|&mut x|`
    Value,   // `|x|`
}

/// Pattern matching.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Wildcard,
    Literal(Literal),
    Identifier { name: String, capture: Option<CaptureBy>, span: Span },
    Reference { is_mut: bool, pat: Box<Pattern> },
    Struct     { path: Vec<String>, fields: HashMap<String, Pattern>, rest: bool },
    Tuple      (Vec<Pattern>),
    TupleStruct{ path: Vec<String>, elems: Vec<Pattern> },
    Slice      { front: Vec<Pattern>, rest: Option<Box<Pattern>>, back: Vec<Pattern> },
    Or         (Vec<Pattern>),
    Range      { start: Option<Literal>, end: Option<Literal>, inclusive: bool },
    Binding    { name: String, subpat: Option<Box<Pattern>> },
    Macro      (Vec<String>, TokenStream),
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
    Local { pat: Pattern, ty: Option<TypeRef>, init: Option<Expr>, attrs: Vec<Attribute>, span: Span },
    Expr  { expr: Expr, span: Span },
    Semi  { expr: Expr, span: Span },
    Item  (Item),
    Macro (Vec<String>, TokenStream, Span),
    Extension(Extension),
}

/// Expressions, each carries an `Effects` mask for purity/IO/async/etc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expr {
    pub kind: ExprKind,
    #[serde(default, skip_serializing, skip_deserializing)]
    pub effects: EffectsFlags,
}

/// Unary operators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,  // -
    Not,  // !
    Deref, // *
    Ref,   // &
    RefMut, // &mut
}

/// Binary operators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,      // +
    Sub,      // -
    Mul,      // *
    Div,      // /
    Rem,      // %
    And,      // &&
    Or,       // ||
    BitAnd,   // &
    BitOr,    // |
    BitXor,   // ^
    Shl,      // <<
    Shr,      // >>
    Eq,       // ==
    Ne,       // !=
    Lt,       // <
    Le,       // <=
    Gt,       // >
    Ge,       // >=
    Assign,   // =
}

/// The variant of expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExprKind {
    Literal     (Literal, Span),
    Variable    (Vec<String>, Span),
    Grouping    (Box<Expr>, Span),
    Unary       { op: UnaryOp, expr: Box<Expr>, span: Span },
    Binary      { left: Box<Expr>, op: BinaryOp, right: Box<Expr>, span: Span },
    Call        { func: Box<Expr>, args: Vec<Expr>, span: Span },
    MethodCall  { receiver: Box<Expr>, method: String, args: Vec<Expr>, span: Span },
    If          { cond: Box<Expr>, then_branch: Block, else_branch: Option<Box<Expr>>, span: Span },
    While       { cond: Box<Expr>, body: Block, is_loop: bool, span: Span },
    For         { pat: Pattern, iter: Box<Expr>, body: Block, span: Span },
    Loop        { body: Block, span: Span },
    Match       { expr: Box<Expr>, arms: Vec<MatchArm>, span: Span },
    Closure     { capture_by: CaptureBy, params: Vec<Param>, return_ty: Option<TypeRef>, body: Box<Expr>, is_async: bool, is_move: bool, span: Span },
    BlockExpr   (Block),
    Async       { capture_by: CaptureBy, block: Block, span: Span },
    Await       { expr: Box<Expr>, span: Span },
    Try         { expr: Box<Expr>, span: Span },
    Cast        { expr: Box<Expr>, ty: TypeRef, span: Span },
    TypeAscription { expr: Box<Expr>, ty: TypeRef, span: Span },
    Tuple       (Vec<Expr>, Span),
    Array       (Vec<Expr>, Span),
    Index       { expr: Box<Expr>, index: Box<Expr>, span: Span },
    Field       { expr: Box<Expr>, name: String, span: Span },
    Range       { start: Option<Box<Expr>>, end: Option<Box<Expr>>, inclusive: bool, span: Span },
    StructLit   { path: Vec<String>, fields: HashMap<String, Expr>, rest: Option<Box<Expr>>, span: Span },
    MacroCall   { path: Vec<String>, tokens: TokenStream, span: Span },
    Extension   (Extension),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pat: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub attrs: Vec<Attribute>,
    pub span: Span,
}
