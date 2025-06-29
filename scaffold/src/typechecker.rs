//! scaffold/src/typechecker.rs - Trivial type checker for `fn main() -> i32 { return 42; }`
//!
//! This is intentionally hardcoded - it only validates the exact program structure we support.

use crate::ast::*;

#[derive(Debug)]
pub enum TypeError {
    WrongFunctionName(String),
    WrongReturnType(String),
    WrongStatementType,
    WrongExpressionType,
    WrongLiteralType,
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeError::WrongFunctionName(name) => {
                write!(f, "Expected function 'main', found '{name}'")
            }
            TypeError::WrongReturnType(type_name) => {
                write!(f, "Expected return type 'i32', found '{type_name}'")
            }
            TypeError::WrongStatementType => {
                write!(f, "Expected return statement")
            }
            TypeError::WrongExpressionType => {
                write!(f, "Expected literal expression")
            }
            TypeError::WrongLiteralType => {
                write!(f, "Expected integer literal")
            }
        }
    }
}

impl std::error::Error for TypeError {}

pub struct TypeChecker;

impl TypeChecker {
    pub fn new() -> Self {
        Self
    }

    /// Type check a program - validates exactly our expected structure
    pub fn check_program(&self, program: &Program) -> Result<(), TypeError> {
        // Must have exactly one function
        if program.functions.len() != 1 {
            return Err(TypeError::WrongFunctionName("expected exactly one function".to_string()));
        }

        let function = &program.functions[0];
        self.check_function(function)
    }

    /// Type check a function - must be main() -> i32
    fn check_function(&self, function: &Function) -> Result<(), TypeError> {
        // Must be named "main"
        if function.name != "main" {
            return Err(TypeError::WrongFunctionName(function.name.clone()));
        }

        // Must have no parameters (already guaranteed by parser structure)
        if !function.params.is_empty() {
            return Err(TypeError::WrongFunctionName("main function cannot have parameters".to_string()));
        }

        // Must return i32
        match &function.return_type {
            Some(return_type) => {
                if return_type.name != "i32" {
                    return Err(TypeError::WrongReturnType(return_type.name.clone()));
                }
            }
            None => {
                return Err(TypeError::WrongReturnType("missing return type".to_string()));
            }
        }

        // Check the function body
        self.check_block(&function.body)
    }

    /// Type check a block - must contain exactly one return statement
    fn check_block(&self, block: &Block) -> Result<(), TypeError> {
        // Must have exactly one statement
        if block.statements.len() != 1 {
            return Err(TypeError::WrongStatementType);
        }

        let statement = &block.statements[0];
        self.check_statement(statement)
    }

    /// Type check a statement - must be return with integer literal
    fn check_statement(&self, statement: &Statement) -> Result<(), TypeError> {
        match statement {
            Statement::Return(expr) => self.check_expression_is_i32(expr),
        }
    }

    /// Type check an expression - must be integer literal
    fn check_expression_is_i32(&self, expr: &Expression) -> Result<(), TypeError> {
        match expr {
            Expression::Literal(literal) => self.check_literal_is_i32(literal),
        }
    }

    /// Type check a literal - must be integer
    fn check_literal_is_i32(&self, literal: &Literal) -> Result<(), TypeError> {
        match literal {
            Literal::Integer(_) => Ok(()), // Any integer is valid i32 for now
        }
    }
}