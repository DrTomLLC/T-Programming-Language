// shared/src/ast/expr.rs
//! Expression AST nodes for T-Lang.
//! Comprehensive expression system supporting all programming paradigms.

use super::types::{Type, SafetyLevel};
use miette::SourceSpan;
use serde::{Deserialize, Serialize};

/// A complete expression with type information and source location.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Option<Type>,
    pub span: SourceSpan,
}

/// All possible expression kinds in T-Lang.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExprKind {
    /// Literals: 42, "hello", true, 3.14
    Literal(Literal),

    /// Variable reference: x, self, super
    Variable {
        path: Vec<String>,
    },

    /// Function call: f(a, b, c)
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        safety: SafetyLevel,
    },

    /// Method call: obj.method(args)
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },

    /// Field access: obj.field
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },

    /// Index access: arr[i]
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },

    /// Range: 0..10, 0..=10, ..10, 0.., ..
    Range {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
    },

    /// Binary operations: +, -, *, /, ==, !=, <, >, etc.
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },

    /// Unary operations: -, !, &, *, ~
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    /// Assignment: x = value, x += value
    Assign {
        target: Box<Expr>,
        op: Option<BinaryOp>, // None for =, Some for +=, -=, etc.
        value: Box<Expr>,
    },

    /// Conditional: if cond { then } else { else }
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },

    /// Match expression: match expr { patterns }
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },

    /// Block expression: { stmt1; stmt2; expr }
    Block(Block),

    /// Loop: loop { body }
    Loop {
        body: Box<Expr>,
        label: Option<String>,
    },

    /// While loop: while cond { body }
    While {
        condition: Box<Expr>,
        body: Box<Expr>,
        label: Option<String>,
    },

    /// For loop: for pat in iter { body }
    For {
        pattern: Pattern,
        iterable: Box<Expr>,
        body: Box<Expr>,
        label: Option<String>,
    },

    /// Break: break, break 'label, break expr
    Break {
        label: Option<String>,
        value: Option<Box<Expr>>,
    },

    /// Continue: continue, continue 'label
    Continue {
        label: Option<String>,
    },

    /// Return: return, return expr
    Return {
        value: Option<Box<Expr>>,
    },

    /// Tuple: (a, b, c)
    Tuple(Vec<Expr>),

    /// Array: [a, b, c] or [expr; count]
    Array {
        elements: Vec<Expr>,
        repeat: Option<Box<Expr>>, // for [expr; count]
    },

    /// Struct literal: Point { x: 1, y: 2 }
    Struct {
        path: Vec<String>,
        fields: Vec<FieldInit>,
        base: Option<Box<Expr>>, // for ..base
    },

    /// Closure: |a, b| a + b, move |x| x
    Closure {
        capture: CaptureMode,
        params: Vec<ClosureParam>,
        return_type: Option<Type>,
        body: Box<Expr>,
    },

    /// Async block: async { ... }, async move { ... }
    Async {
        capture: CaptureMode,
        body: Box<Expr>,
    },

    /// Await: expr.await
    Await {
        expr: Box<Expr>,
    },

    /// Try: expr?
    Try {
        expr: Box<Expr>,
    },

    /// Unsafe block: unsafe { ... }
    Unsafe {
        body: Box<Expr>,
    },

    /// Type cast: expr as Type
    Cast {
        expr: Box<Expr>,
        target_type: Type,
    },

    /// Reference: &expr, &mut expr
    Reference {
        expr: Box<Expr>,
        mutable: bool,
    },

    /// Dereference: *expr
    Dereference {
        expr: Box<Expr>,
    },
}

/// Literal values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Integer(i128),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Unit,
}

/// Binary operators.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod, Pow,

    // Comparison
    Eq, Ne, Lt, Le, Gt, Ge,

    // Logical
    And, Or,

    // Bitwise
    BitAnd, BitOr, BitXor, Shl, Shr,
}

/// Unary operators.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,    // -expr
    Not,    // !expr
    BitNot, // ~expr
}

/// A block of statements and expressions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<super::stmt::Stmt>,
    pub expr: Option<Box<Expr>>,
    pub span: SourceSpan,
}

/// Pattern matching constructs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternKind {
    Wild,                          // _
    Ident(String),                 // x
    Literal(Literal),              // 42, "hello"
    Tuple(Vec<Pattern>),           // (a, b, c)
    Struct { path: Vec<String>, fields: Vec<FieldPattern> }, // Point { x, y }
    Enum { path: Vec<String>, variant: String, fields: Vec<Pattern> }, // Some(x)
    Slice(Vec<Pattern>),           // [a, b, c]
    Range { start: Box<Expr>, end: Box<Expr>, inclusive: bool }, // 0..10
    Or(Vec<Pattern>),              // a | b | c
    Guard { pattern: Box<Pattern>, condition: Box<Expr> }, // x if x > 0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldPattern {
    pub name: String,
    pub pattern: Option<Pattern>, // None for shorthand
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldInit {
    pub name: String,
    pub value: Option<Expr>, // None for shorthand
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CaptureMode {
    Move,
    Ref,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClosureParam {
    pub pattern: Pattern,
    pub ty: Option<Type>,
    pub span: SourceSpan,
}

impl Expr {
    pub fn new(kind: ExprKind, span: SourceSpan) -> Self {
        Self { kind, ty: None, span }
    }

    pub fn with_type(mut self, ty: Type) -> Self {
        self.ty = Some(ty);
        self
    }
}