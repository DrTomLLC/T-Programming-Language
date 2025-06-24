//! shared/src/lib.rs
//! Core shared types for the T-Lang compiler pipeline.
//! These types are used across all compiler phases.

mod tokenizer;

use miette::SourceSpan;
use std::collections::HashMap;

// Re-export for convenience
pub use errors::TlError;
pub type Result<T> = std::result::Result<T, TlError>;

/// Top-level program representation
#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
    pub source_map: SourceMap,
}

/// Top-level language items
#[derive(Debug, Clone)]
pub enum Item {
    Function(FunctionDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Use(UseDecl),
    Const(ConstDecl),
}

/// Function declaration
#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub span: SourceSpan,
    pub safety_level: SafetyLevel,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
    pub span: SourceSpan,
}

/// Struct declaration
#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub fields: Vec<FieldDecl>,
    pub span: SourceSpan,
}

/// Struct field
#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub name: String,
    pub ty: Type,
    pub span: SourceSpan,
}

/// Enum declaration
#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub span: SourceSpan,
}

/// Enum variant
#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub types: Vec<Type>,
    pub span: SourceSpan,
}

/// Use/import declaration
#[derive(Debug, Clone)]
pub struct UseDecl {
    pub path: Vec<String>,
    pub span: SourceSpan,
}

/// Constant declaration
#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub name: String,
    pub ty: Type,
    pub value: Expression,
    pub span: SourceSpan,
}

/// Type system
#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub kind: TypeKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Primitive(PrimitiveType),
    Reference(Box<Type>, Mutability),
    Array(Box<Type>, Option<usize>),
    Slice(Box<Type>),
    Tuple(Vec<Type>),
    Struct(String),
    Enum(String),
    Function(Vec<Type>, Box<Type>),
    Generic(String),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimitiveType {
    Bool,
    I8, I16, I32, I64, I128,
    U8, U16, U32, U64, U128,
    F32, F64,
    Char,
    Str,
    Unit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

impl Type {
    pub fn new(kind: TypeKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    pub fn primitive(prim: PrimitiveType, span: SourceSpan) -> Self {
        Self::new(TypeKind::Primitive(prim), span)
    }

    pub fn unit(span: SourceSpan) -> Self {
        Self::primitive(PrimitiveType::Unit, span)
    }

    pub fn bool(span: SourceSpan) -> Self {
        Self::primitive(PrimitiveType::Bool, span)
    }

    pub fn i32(span: SourceSpan) -> Self {
        Self::primitive(PrimitiveType::I32, span)
    }

    pub fn str(span: SourceSpan) -> Self {
        Self::primitive(PrimitiveType::Str, span)
    }
}

/// Code block
#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: SourceSpan,
}

/// Statements
#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    Let(LetStatement),
    If(IfStatement),
    While(WhileStatement),
    Return(ReturnStatement),
    Block(Block),
}

#[derive(Debug, Clone)]
pub struct LetStatement {
    pub name: String,
    pub ty: Option<Type>,
    pub initializer: Option<Expression>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: Block,
    pub else_branch: Option<Block>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Block,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub value: Option<Expression>,
    pub span: SourceSpan,
}

/// Expressions
#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Call(CallExpression),
    FieldAccess(FieldAccessExpression),
    Index(IndexExpression),
    Grouping(Box<Expression>),
}

#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub operand: Box<Expression>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct CallExpression {
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct FieldAccessExpression {
    pub object: Box<Expression>,
    pub field: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct IndexExpression {
    pub object: Box<Expression>,
    pub index: Box<Expression>,
    pub span: SourceSpan,
}

/// Operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOperator {
    Add, Sub, Mul, Div, Mod,
    Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    And, Or,
    Assign,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOperator {
    Minus, Not, Reference, Dereference,
}

/// Literals
#[derive(Debug, Clone)]
pub enum Literal {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Character(char),
    Unit,
}

/// Safety levels for operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SafetyLevel {
    Safe,
    Unsafe,
}

/// Source code mapping for error reporting
#[derive(Debug, Clone)]
pub struct SourceMap {
    pub source: String,
    pub file_path: Option<String>,
    pub line_starts: Vec<usize>,
}

impl SourceMap {
    pub fn new(source: String, file_path: Option<String>) -> Self {
        let mut line_starts = vec![0];
        for (i, ch) in source.char_indices() {
            if ch == '\n' {
                line_starts.push(i + 1);
            }
        }
        Self { source, file_path, line_starts }
    }

    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let line = self.line_starts.binary_search(&offset)
            .unwrap_or_else(|i| i.saturating_sub(1));
        let col = offset - self.line_starts[line];
        (line + 1, col + 1)
    }
}

/// Implementation helpers for common operations
impl Program {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            source_map: SourceMap::new(String::new(), None),
        }
    }

    pub fn with_source(source: String, file_path: Option<String>) -> Self {
        Self {
            items: Vec::new(),
            source_map: SourceMap::new(source, file_path),
        }
    }

    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }
}

impl Block {
    pub fn new(span: SourceSpan) -> Self {
        Self {
            statements: Vec::new(),
            span,
        }
    }

    pub fn add_statement(&mut self, stmt: Statement) {
        self.statements.push(stmt);
    }
}

/// Helper for creating expressions
impl Expression {
    pub fn literal(lit: Literal) -> Self {
        Expression::Literal(lit)
    }

    pub fn identifier(name: String) -> Self {
        Expression::Identifier(name)
    }

    pub fn binary(left: Expression, op: BinaryOperator, right: Expression, span: SourceSpan) -> Self {
        Expression::Binary(BinaryExpression {
            left: Box::new(left),
            operator: op,
            right: Box::new(right),
            span,
        })
    }

    pub fn call(callee: Expression, args: Vec<Expression>, span: SourceSpan) -> Self {
        Expression::Call(CallExpression {
            callee: Box::new(callee),
            arguments: args,
            span,
        })
    }
}

/// File system utilities for testing
pub mod fs {
    use std::path::Path;
    use crate::Result;

    pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
        std::fs::read_to_string(path)
            .map_err(|e| crate::TlError::Io {
                message: format!("Failed to read file: {}", e),
                source: Some(e),
            })
    }
}