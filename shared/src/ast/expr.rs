// shared/src/ast/expr.rs
//! Expression AST nodes for T-Lang.

use miette::SourceSpan;
use serde::{Deserialize, Serialize};

/// An expression in T-Lang.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: SourceSpan,
}

/// All possible expression kinds in T-Lang.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExprKind {
    /// Literal values: 42, "hello", true, etc.
    Literal(Literal),

    /// Identifier: variable_name
    Identifier(String),

    /// Binary operation: lhs + rhs
    Binary {
        lhs: Box<Expr>,
        op: BinaryOp,
        rhs: Box<Expr>,
    },

    /// Unary operation: !expr, -expr
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },

    /// Function call: func(args)
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },

    /// Field access: expr.field
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },

    /// Method call: expr.method(args)
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },

    /// Index access: expr[index]
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },

    /// Slice: expr[start..end]
    Slice {
        object: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
    },

    /// Tuple: (expr1, expr2, ...)
    Tuple(Vec<Expr>),

    /// Array: [expr1, expr2, ...] or [expr; count]
    Array {
        elements: Vec<Expr>,
        repeat: Option<Box<Expr>>, // For [value; count] syntax
    },

    /// Struct literal: StructName { field1: expr1, .. }
    Struct {
        path: Vec<String>,
        fields: Vec<StructFieldExpr>,
        base: Option<Box<Expr>>, // For update syntax
    },

    /// Assignment: lvalue = rvalue
    Assignment {
        target: Box<Expr>,
        value: Box<Expr>,
        op: Option<BinaryOp>, // For compound assignment +=, -=, etc.
    },

    /// Block expression: { stmts; expr }
    Block {
        stmts: Vec<super::Stmt>,
        expr: Option<Box<Expr>>,
    },

    /// If expression: if cond { then } else { else }
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },

    /// While loop: while cond { body }
    While {
        condition: Box<Expr>,
        body: Box<Expr>,
    },

    /// For loop: for pattern in iterable { body }
    For {
        pattern: Pattern,
        iterable: Box<Expr>,
        body: Box<Expr>,
    },

    /// Loop: loop { body }
    Loop {
        body: Box<Expr>,
        label: Option<String>,
    },

    /// Break: break [label] [value]
    Break {
        label: Option<String>,
        value: Option<Box<Expr>>,
    },

    /// Continue: continue [label]
    Continue {
        label: Option<String>,
    },

    /// Return: return [expr]
    Return {
        value: Option<Box<Expr>>,
    },

    /// Match expression: match expr { arms }
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },

    /// Closure: |params| body
    Closure {
        params: Vec<ClosureParam>,
        body: Box<Expr>,
        is_async: bool,
        is_move: bool,
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

    /// Type cast: expr as Type
    Cast {
        expr: Box<Expr>,
        ty: super::Type,
    },

    /// Try operator: expr?
    Try {
        expr: Box<Expr>,
    },

    /// Await: expr.await
    Await {
        expr: Box<Expr>,
    },

    /// Async block: async { body }
    Async {
        body: Box<Expr>,
    },

    /// Unsafe block: unsafe { body }
    Unsafe {
        body: Box<Expr>,
    },

    /// Parenthesized expression: (expr)
    Grouping(Box<Expr>),

    /// Range: start..end, start..=end, ..end, start.., ..
    Range {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
    },

    /// Macro invocation: macro_name!(args)
    Macro {
        path: Vec<String>,
        args: Vec<super::MacroArg>,
    },
}

/// Literal values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Unit,
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    Mod,    // %

    // Bitwise
    And,    // &
    Or,     // |
    Xor,    // ^
    Shl,    // <<
    Shr,    // >>

    // Comparison
    Eq,     // ==
    Ne,     // !=
    Lt,     // <
    Le,     // <=
    Gt,     // >
    Ge,     // >=

    // Logical
    LogicalAnd, // &&
    LogicalOr,  // ||
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,    // !
    Neg,    // -
    Plus,   // +
}

/// Struct field in a struct literal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructFieldExpr {
    pub name: String,
    pub value: Expr,
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
    /// Wildcard pattern: _
    Wildcard,

    /// Identifier pattern: name
    Identifier {
        name: String,
        mutable: bool,
    },

    /// Literal pattern: 42, "hello", true
    Literal(Literal),

    /// Tuple pattern: (pat1, pat2, ...)
    Tuple(Vec<Pattern>),

    /// Struct pattern: StructName { field1: pat1, .. }
    Struct {
        path: Vec<String>,
        fields: Vec<(String, Pattern)>,
        rest: bool, // For .. in pattern
    },

    /// Reference pattern: &pat, &mut pat
    Ref(Box<Pattern>),

    /// Range pattern: 1..=10
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },

    /// Or pattern: pat1 | pat2
    Or(Vec<Pattern>),
}

/// Match arm in a match expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: SourceSpan,
}

/// Closure parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClosureParam {
    pub pattern: Pattern,
    pub ty: Option<super::Type>,
    pub span: SourceSpan,
}

/// Helper constructors for expressions.
impl Expr {
    pub fn literal(lit: Literal, span: SourceSpan) -> Self {
        Self {
            kind: ExprKind::Literal(lit),
            span,
        }
    }

    pub fn identifier(name: String, span: SourceSpan) -> Self {
        Self {
            kind: ExprKind::Identifier(name),
            span,
        }
    }

    pub fn binary(lhs: Expr, op: BinaryOp, rhs: Expr, span: SourceSpan) -> Self {
        Self {
            kind: ExprKind::Binary {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            },
            span,
        }
    }

    pub fn unary(op: UnaryOp, operand: Expr, span: SourceSpan) -> Self {
        Self {
            kind: ExprKind::Unary {
                op,
                operand: Box::new(operand),
            },
            span,
        }
    }

    pub fn call(func: Expr, args: Vec<Expr>, span: SourceSpan) -> Self {
        Self {
            kind: ExprKind::Call {
                func: Box::new(func),
                args,
            },
            span,
        }
    }

    pub fn field_access(object: Expr, field: String, span: SourceSpan) -> Self {
        Self {
            kind: ExprKind::FieldAccess {
                object: Box::new(object),
                field,
            },
            span,
        }
    }

    pub fn index(object: Expr, index: Expr, span: SourceSpan) -> Self {
        Self {
            kind: ExprKind::Index {
                object: Box::new(object),
                index: Box::new(index),
            },
            span,
        }
    }

    pub fn tuple(elements: Vec<Expr>, span: SourceSpan) -> Self {
        Self {
            kind: ExprKind::Tuple(elements),
            span,
        }
    }

    pub fn array(elements: Vec<Expr>, span: SourceSpan) -> Self {
        Self {
            kind: ExprKind::Array {
                elements,
                repeat: None,
            },
            span,
        }
    }

    pub fn block(stmts: Vec<super::Stmt>, expr: Option<Expr>, span: SourceSpan) -> Self {
        Self {
            kind: ExprKind::Block {
                stmts,
                expr: expr.map(Box::new),
            },
            span,
        }
    }

    pub fn if_expr(condition: Expr, then_branch: Expr, else_branch: Option<Expr>, span: SourceSpan) -> Self {
        Self {
            kind: ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: else_branch.map(Box::new),
            },
            span,
        }
    }

    pub fn while_loop(condition: Expr, body: Expr, span: SourceSpan) -> Self {
        Self {
            kind: ExprKind::While {
                condition: Box::new(condition),
                body: Box::new(body),
            },
            span,
        }
    }

    pub fn assignment(target: Expr, value: Expr, span: SourceSpan) -> Self {
        Self {
            kind: ExprKind::Assignment {
                target: Box::new(target),
                value: Box::new(value),
                op: None,
            },
            span,
        }
    }

    pub fn return_expr(value: Option<Expr>, span: SourceSpan) -> Self {
        Self {
            kind: ExprKind::Return {
                value: value.map(Box::new),
            },
            span,
        }
    }

    pub fn break_expr(label: Option<String>, value: Option<Expr>, span: SourceSpan) -> Self {
        Self {
            kind: ExprKind::Break {
                label,
                value: value.map(Box::new),
            },
            span,
        }
    }

    pub fn continue_expr(label: Option<String>, span: SourceSpan) -> Self {
        Self {
            kind: ExprKind::Continue { label },
            span,
        }
    }
}

impl Pattern {
    pub fn wildcard(span: SourceSpan) -> Self {
        Self {
            kind: PatternKind::Wildcard,
            span,
        }
    }

    pub fn identifier(name: String, span: SourceSpan) -> Self {
        Self {
            kind: PatternKind::Identifier {
                name,
                mutable: false,
            },
            span,
        }
    }

    pub fn literal(lit: Literal, span: SourceSpan) -> Self {
        Self {
            kind: PatternKind::Literal(lit),
            span,
        }
    }

    pub fn tuple(patterns: Vec<Pattern>, span: SourceSpan) -> Self {
        Self {
            kind: PatternKind::Tuple(patterns),
            span,
        }
    }
}

impl StructFieldExpr {
    pub fn new(name: String, value: Expr, span: SourceSpan) -> Self {
        Self { name, value, span }
    }
}

impl MatchArm {
    pub fn new(pattern: Pattern, body: Expr, span: SourceSpan) -> Self {
        Self {
            pattern,
            guard: None,
            body,
            span,
        }
    }

    pub fn with_guard(pattern: Pattern, guard: Expr, body: Expr, span: SourceSpan) -> Self {
        Self {
            pattern,
            guard: Some(guard),
            body,
            span,
        }
    }
}

impl ClosureParam {
    pub fn new(pattern: Pattern, span: SourceSpan) -> Self {
        Self {
            pattern,
            ty: None,
            span,
        }
    }
}

/// Operator precedence for binary operators.
impl BinaryOp {
    pub fn precedence(self) -> u8 {
        match self {
            BinaryOp::LogicalOr => 1,
            BinaryOp::LogicalAnd => 2,
            BinaryOp::Eq | BinaryOp::Ne => 3,
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => 4,
            BinaryOp::Or => 5,
            BinaryOp::Xor => 6,
            BinaryOp::And => 7,
            BinaryOp::Shl | BinaryOp::Shr => 8,
            BinaryOp::Add | BinaryOp::Sub => 9,
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => 10,
        }
    }

    pub fn is_right_associative(self) -> bool {
        false // All binary operators are left-associative in T-Lang
    }

    pub fn is_comparison(self) -> bool {
        matches!(self, BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge)
    }

    pub fn is_arithmetic(self) -> bool {
        matches!(self, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod)
    }

    pub fn is_logical(self) -> bool {
        matches!(self, BinaryOp::LogicalAnd | BinaryOp::LogicalOr)
    }

    pub fn is_bitwise(self) -> bool {
        matches!(self, BinaryOp::And | BinaryOp::Or | BinaryOp::Xor | BinaryOp::Shl | BinaryOp::Shr)
    }
}