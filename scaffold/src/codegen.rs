//! scaffold/src/codegen.rs - Code generator that outputs valid Rust code
//!
//! This generates Rust code from our validated AST.

use crate::ast::*;

#[derive(Debug)]
pub enum CodegenError {
    UnsupportedConstruct(String),
    InvalidLiteral(String),
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::UnsupportedConstruct(msg) => {
                write!(f, "Unsupported construct: {msg}")
            }
            CodegenError::InvalidLiteral(msg) => {
                write!(f, "Invalid literal: {msg}")
            }
        }
    }
}

impl std::error::Error for CodegenError {}

pub struct CodeGenerator {
    output: String,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    /// Generate Rust code from a program
    pub fn generate_program(&mut self, program: &Program) -> Result<String, CodegenError> {
        self.output.clear();

        // Generate all functions
        for function in &program.functions {
            self.generate_function(function)?;
        }

        Ok(self.output.clone())
    }

    /// Generate Rust code for a function
    fn generate_function(&mut self, function: &Function) -> Result<(), CodegenError> {
        // Special handling for main function to make it compatible with Rust
        if function.name == "main" {
            // Generate a wrapper main function
            self.output.push_str("fn main() {\n");
            self.output.push_str("    let exit_code = tlang_main();\n");
            self.output.push_str("    std::process::exit(exit_code);\n");
            self.output.push_str("}\n\n");

            // Generate the actual T-Lang main function with a different name
            self.output.push_str("fn tlang_main()");

            // Generate return type
            if let Some(return_type) = &function.return_type {
                self.output.push_str(" -> ");
                self.generate_type(return_type)?;
            }

            self.output.push(' ');

            // Generate function body
            self.generate_block(&function.body)?;
        } else {
            // Generate function signature normally for non-main functions
            self.output.push_str("fn ");
            self.output.push_str(&function.name);
            self.output.push('(');

            // Generate parameters
            for (i, param) in function.params.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                self.output.push_str(&param.name);
                self.output.push_str(": ");
                self.generate_type(&param.param_type)?;
            }

            self.output.push(')');

            // Generate return type
            if let Some(return_type) = &function.return_type {
                self.output.push_str(" -> ");
                self.generate_type(return_type)?;
            }

            self.output.push(' ');

            // Generate function body
            self.generate_block(&function.body)?;
        }

        self.output.push('\n');

        Ok(())
    }

    /// Generate Rust code for a type
    fn generate_type(&mut self, type_ref: &Type) -> Result<(), CodegenError> {
        // For now, just output the type name directly
        // This works for primitive types like i32
        self.output.push_str(&type_ref.name);
        Ok(())
    }

    /// Generate Rust code for a block
    fn generate_block(&mut self, block: &Block) -> Result<(), CodegenError> {
        self.output.push('{');

        for statement in &block.statements {
            self.output.push('\n');
            self.output.push_str("    "); // Indent
            self.generate_statement(statement)?;
        }

        self.output.push('\n');
        self.output.push('}');

        Ok(())
    }

    /// Generate Rust code for a statement
    fn generate_statement(&mut self, statement: &Statement) -> Result<(), CodegenError> {
        match statement {
            Statement::Return(expr) => {
                self.output.push_str("return ");
                self.generate_expression(expr)?;
                self.output.push(';');
            }
        }
        Ok(())
    }

    /// Generate Rust code for an expression
    fn generate_expression(&mut self, expr: &Expression) -> Result<(), CodegenError> {
        match expr {
            Expression::Literal(literal) => {
                self.generate_literal(literal)?;
            }
        }
        Ok(())
    }

    /// Generate Rust code for a literal
    fn generate_literal(&mut self, literal: &Literal) -> Result<(), CodegenError> {
        match literal {
            Literal::Integer(value) => {
                self.output.push_str(&value.to_string());
            }
        }
        Ok(())
    }
}