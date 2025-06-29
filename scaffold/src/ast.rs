//! scaffold/src/ast.rs - Minimal AST for parsing `fn main() -> i32 { return 42; }`
//!
//! FROZEN: Once this works, never change the structure - only add fields.

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone)]
pub struct Type {
    pub name: String,  // Just store type names as strings for now
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Return(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Integer(i64),
}

impl Program {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
        }
    }
}

impl Function {
    pub fn new(name: String) -> Self {
        Self {
            name,
            params: Vec::new(),
            return_type: None,
            body: Block { statements: Vec::new() },
        }
    }
}