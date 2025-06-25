// shared/src/tir/mod.rs
//! T-Lang Intermediate Representation (TIR)
//!
//! This module provides a lower-level IR that sits between the AST and target code generation.
//! It performs type checking, safety analysis, and optimization while maintaining source location information.

use crate::ast::{self, *};
use errors::{TlError, *};
use miette::SourceSpan;
use std::collections::HashMap;

/// A complete TIR module representing a compilation unit
#[derive(Debug, Clone, PartialEq)]
pub struct TirModule {
    pub name: String,
    pub functions: Vec<TirFunction>,
    pub globals: Vec<TirGlobal>,
    pub types: Vec<TirTypeDefinition>,
    pub span: SourceSpan,
}

/// A function in TIR form with full type information
#[derive(Debug, Clone, PartialEq)]
pub struct TirFunction {
    pub name: String,
    pub parameters: Vec<TirParameter>,
    pub return_type: TirType,
    pub body: TirBlock,
    pub is_public: bool,
    pub is_unsafe: bool,
    pub span: SourceSpan,
}

/// A function parameter with type information
#[derive(Debug, Clone, PartialEq)]
pub struct TirParameter {
    pub name: String,
    pub ty: TirType,
    pub is_mutable: bool,
    pub span: SourceSpan,
}

/// A global variable or constant in TIR
#[derive(Debug, Clone, PartialEq)]
pub struct TirGlobal {
    pub name: String,
    pub ty: TirType,
    pub initializer: Option<ValueId>,
    pub is_mutable: bool,
    pub is_public: bool,
    pub span: SourceSpan,
}

/// A type definition in TIR
#[derive(Debug, Clone, PartialEq)]
pub struct TirTypeDefinition {
    pub name: String,
    pub kind: TirTypeDefKind,
    pub is_public: bool,
    pub span: SourceSpan,
}

/// Different kinds of type definitions
#[derive(Debug, Clone, PartialEq)]
pub enum TirTypeDefKind {
    Struct { fields: Vec<TirField> },
    Enum { variants: Vec<TirEnumVariant> },
    Alias { target: TirType },
}

/// A struct field
#[derive(Debug, Clone, PartialEq)]
pub struct TirField {
    pub name: String,
    pub ty: TirType,
    pub is_public: bool,
    pub span: SourceSpan,
}

/// An enum variant
#[derive(Debug, Clone, PartialEq)]
pub struct TirEnumVariant {
    pub name: String,
    pub fields: Option<Vec<TirType>>,
    pub span: SourceSpan,
}

/// A block of instructions
#[derive(Debug, Clone, PartialEq)]
pub struct TirBlock {
    pub instructions: Vec<TirInstruction>,
    pub terminator: Option<TirTerminator>,
    pub span: SourceSpan,
}

/// A single instruction in the TIR
#[derive(Debug, Clone, PartialEq)]
pub struct TirInstruction {
    pub kind: TirInstructionKind,
    pub result: Option<ValueId>,
    pub span: SourceSpan,
}

/// Different kinds of TIR instructions
#[derive(Debug, Clone, PartialEq)]
pub enum TirInstructionKind {
    // Literals
    ConstantInt { value: i64 },
    ConstantFloat { value: f64 },
    ConstantString { value: String },
    ConstantBool { value: bool },
    ConstantUnit,

    // Arithmetic operations
    Add { lhs: ValueId, rhs: ValueId },
    Sub { lhs: ValueId, rhs: ValueId },
    Mul { lhs: ValueId, rhs: ValueId },
    Div { lhs: ValueId, rhs: ValueId },
    Rem { lhs: ValueId, rhs: ValueId },

    // Comparison operations
    Eq { lhs: ValueId, rhs: ValueId },
    Ne { lhs: ValueId, rhs: ValueId },
    Lt { lhs: ValueId, rhs: ValueId },
    Le { lhs: ValueId, rhs: ValueId },
    Gt { lhs: ValueId, rhs: ValueId },
    Ge { lhs: ValueId, rhs: ValueId },

    // Logical operations
    And { lhs: ValueId, rhs: ValueId },
    Or { lhs: ValueId, rhs: ValueId },
    Not { operand: ValueId },

    // Memory operations
    Load { address: ValueId },
    Store { address: ValueId, value: ValueId },
    Alloca { ty: TirType },

    // Control flow
    Call { function: String, args: Vec<ValueId> },

    // Variable operations
    GetLocal { name: String },
    SetLocal { name: String, value: ValueId },
    GetGlobal { name: String },
    SetGlobal { name: String, value: ValueId },
}

/// Block terminators (instructions that end a block)
#[derive(Debug, Clone, PartialEq)]
pub enum TirTerminator {
    Return { value: Option<ValueId> },
    Branch { target: BlockId },
    ConditionalBranch { condition: ValueId, true_target: BlockId, false_target: BlockId },
    Unreachable,
}

/// Type representation in TIR
#[derive(Debug, Clone, PartialEq)]
pub struct TirType {
    pub kind: TirTypeKind,
    pub span: SourceSpan,
}

/// Different kinds of types in TIR
#[derive(Debug, Clone, PartialEq)]
pub enum TirTypeKind {
    // Primitive types
    Bool,
    I8, I16, I32, I64, I128, ISize,
    U8, U16, U32, U64, U128, USize,
    F32, F64,
    Char,
    Str,
    Unit,

    // Composite types
    Pointer { target: Box<TirType>, is_mutable: bool },
    Array { element: Box<TirType>, size: usize },
    Slice { element: Box<TirType> },
    Function { parameters: Vec<TirType>, return_type: Box<TirType> },

    // User-defined types
    Struct { name: String },
    Enum { name: String },

    // Type variables (for generics)
    Generic { name: String },
}

/// Unique identifier for values in TIR
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId(pub u32);

/// Unique identifier for basic blocks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

impl ValueId {
    pub fn new(id: u32) -> Self {
        ValueId(id)
    }
}

impl BlockId {
    pub fn new(id: u32) -> Self {
        BlockId(id)
    }
}

/// Builder for constructing TIR from AST
pub struct TirBuilder {
    next_value_id: u32,
    next_block_id: u32,
    locals: HashMap<String, ValueId>,
    globals: HashMap<String, ValueId>,
}

impl TirBuilder {
    pub fn new() -> Self {
        Self {
            next_value_id: 0,
            next_block_id: 0,
            locals: HashMap::new(),
            globals: HashMap::new(),
        }
    }

    pub fn next_value_id(&mut self) -> ValueId {
        let id = ValueId(self.next_value_id);
        self.next_value_id += 1;
        id
    }

    pub fn next_block_id(&mut self) -> ValueId {
        let id = ValueId(self.next_block_id);
        self.next_block_id += 1;
        id
    }

    /// Build a TIR module from an AST program
    pub fn build_module(&mut self, ast: &ast::Program) -> Result<TirModule> {
        let mut functions = Vec::new();
        let mut globals = Vec::new();
        let mut types = Vec::new();

        for item in &ast.items {
            match &item.kind {
                ast::ItemKind::Function { name, params, return_type, body, .. } => {
                    let tir_function = self.build_function(name, params, return_type, body)?;
                    functions.push(tir_function);
                }
                ast::ItemKind::Const { name, ty, value } => {
                    let tir_global = self.build_global(name, ty, Some(value), false)?;
                    globals.push(tir_global);
                }
                ast::ItemKind::Static { name, ty, value, mutable } => {
                    let tir_global = self.build_global(name, ty, value.as_ref(), *mutable)?;
                    globals.push(tir_global);
                }
                _ => {
                    // Handle other item types as needed
                    continue;
                }
            }
        }

        Ok(TirModule {
            name: "main".to_string(),
            functions,
            globals,
            types,
            span: ast.span,
        })
    }

    fn build_function(
        &mut self,
        name: &str,
        params: &[ast::FunctionParam],
        return_type: &Option<ast::Type>,
        body: &ast::Block,
    ) -> Result<TirFunction, E> {
        let mut parameters = Vec::new();

        for param in params {
            let tir_param = TirParameter {
                name: param.name.clone(),
                ty: self.convert_type(&param.ty)?,
                is_mutable: param.is_mutable,
                span: param.span,
            };
            parameters.push(tir_param);
        }

        let return_ty = if let Some(ty) = return_type {
            self.convert_type(ty)?
        } else {
            TirType {
                kind: TirTypeKind::Unit,
                span: SourceSpan::new(0.into(), 0),
            }
        };

        let tir_body = self.build_block(body)?;

        Ok(TirFunction {
            name: name.to_string(),
            parameters,
            return_type: return_ty,
            body: tir_body,
            is_public: true, // TODO: Get from visibility
            is_unsafe: false, // TODO: Get from attributes
            span: body.span,
        })
    }

    fn build_block(&mut self, block: &ast::Block) -> Result<TirBlock, E> {
        let mut instructions = Vec::new();

        for stmt in &block.stmts {
            match &stmt.kind {
                ast::StmtKind::Expr(expr) => {
                    self.build_expression(expr)?;
                }
                ast::StmtKind::Let { pattern, ty: _, initializer, .. } => {
                    if let Some(init_expr) = initializer {
                        let value_id = self.build_expression(init_expr)?;
                        if let ast::PatternKind::Identifier { name, .. } = &pattern.kind {
                            self.locals.insert(name.clone(), value_id);
                        }
                    }
                }
                ast::StmtKind::Return { value } => {
                    if let Some(expr) = value {
                        self.build_expression(expr)?;
                    }
                }
                _ => {
                    // Handle other statement types
                }
            }
        }

        Ok(TirBlock {
            instructions,
            terminator: None,
            span: block.span,
        })
    }

    fn build_expression(&mut self, expr: &ast::Expr) -> Result<ValueId> {
        let value_id = self.next_value_id();

        match &expr.kind {
            ast::ExprKind::Literal(lit) => {
                self.build_literal(lit, expr.span)?;
            }
            ast::ExprKind::Variable { path } => {
                // Handle variable lookup
                if path.len() == 1 {
                    let name = &path[0];
                    if let Some(&local_id) = self.locals.get(name) {
                        return Ok(local_id);
                    }
                }
            }
            ast::ExprKind::Binary { left, lhs, op, right, rhs } => {
                self.build_binary_op(left, op, right, expr.span)?;
            }
            _ => {
                // Handle other expression types
            }
        }

        Ok(value_id)
    }

    fn build_literal(&mut self, lit: &ast::Literal, span: SourceSpan) -> Result<ValueId, E> {
        let value_id = self.next_value_id();

        let instruction_kind = match lit {
            ast::Literal::Integer(n) => {
                TirInstructionKind::ConstantInt { value: *n }
            }
            ast::Literal::Float(f) => {
                TirInstructionKind::ConstantFloat { value: *f }
            }
            ast::Literal::String(s) => {
                TirInstructionKind::ConstantString { value: s.clone() }
            }
            ast::Literal::Char(_c) => {
                TirInstructionKind::ConstantInt { value: 0 } // TODO: Convert char to int
            }
            ast::Literal::Bool(b) => {
                TirInstructionKind::ConstantBool { value: *b }
            }
            ast::Literal::Unit => {
                TirInstructionKind::ConstantUnit
            }
        };

        // Note: In a real implementation, you'd add this instruction to the current block
        // For now, we just return the value ID
        Ok(value_id)
    }

    fn build_binary_op(
        &mut self,
        left: &ast::Expr,
        op: &ast::BinaryOp,
        right: &ast::Expr,
        _span: SourceSpan,
    ) -> Result<ValueId> {
        let left_value = self.build_expression(left)?;
        let right_value = self.build_expression(right)?;
        let result_id = self.next_value_id();

        let instruction_kind = match op {
            ast::BinaryOp::Add => TirInstructionKind::Add { lhs: left_value, rhs: right_value },
            ast::BinaryOp::Sub => TirInstructionKind::Sub { lhs: left_value, rhs: right_value },
            ast::BinaryOp::Mul => TirInstructionKind::Mul { lhs: left_value, rhs: right_value },
            ast::BinaryOp::Div => TirInstructionKind::Div { lhs: left_value, rhs: right_value },
            ast::BinaryOp::Eq => TirInstructionKind::Eq { lhs: left_value, rhs: right_value },
            ast::BinaryOp::Ne => TirInstructionKind::Ne { lhs: left_value, rhs: right_value },
            ast::BinaryOp::Lt => TirInstructionKind::Lt { lhs: left_value, rhs: right_value },
            ast::BinaryOp::Le => TirInstructionKind::Le { lhs: left_value, rhs: right_value },
            ast::BinaryOp::Gt => TirInstructionKind::Gt { lhs: left_value, rhs: right_value },
            ast::BinaryOp::Ge => TirInstructionKind::Ge { lhs: left_value, rhs: right_value },
            _ => {
                return Err(TlError::internal(
                    format!("Unsupported binary operator: {:?}", op)
                ));
            }
        };

        // Note: In a real implementation, you'd add this instruction to the current block
        Ok(result_id)
    }

    fn build_global(
        &mut self,
        name: &str,
        ty: &ast::Type,
        value: Option<&ast::Expr>,
        mutable: bool,
    ) -> Result<TirGlobal> {
        let tir_type = self.convert_type(ty)?;
        let initializer = if let Some(expr) = value {
            Some(self.build_expression(expr)?)
        } else {
            None
        };

        Ok(TirGlobal {
            name: name.to_string(),
            ty: tir_type,
            initializer,
            is_mutable: mutable,
            is_public: true, // TODO: Get from visibility
            span: ty.span,
        })
    }

    fn convert_type(&self, ast_type: &ast::Type) -> Result<TirType> {
        let kind = match &ast_type.kind {
            ast::TypeKind::Primitive(prim) => {
                match prim {
                    ast::PrimitiveType::Bool => TirTypeKind::Bool,
                    ast::PrimitiveType::I8 => TirTypeKind::I8,
                    ast::PrimitiveType::I16 => TirTypeKind::I16,
                    ast::PrimitiveType::I32 => TirTypeKind::I32,
                    ast::PrimitiveType::I64 => TirTypeKind::I64,
                    ast::PrimitiveType::I128 => TirTypeKind::I128,
                    ast::PrimitiveType::ISize => TirTypeKind::ISize,
                    ast::PrimitiveType::U8 => TirTypeKind::U8,
                    ast::PrimitiveType::U16 => TirTypeKind::U16,
                    ast::PrimitiveType::U32 => TirTypeKind::U32,
                    ast::PrimitiveType::U64 => TirTypeKind::U64,
                    ast::PrimitiveType::U128 => TirTypeKind::U128,
                    ast::PrimitiveType::USize => TirTypeKind::USize,
                    ast::PrimitiveType::F32 => TirTypeKind::F32,
                    ast::PrimitiveType::F64 => TirTypeKind::F64,
                    ast::PrimitiveType::Char => TirTypeKind::Char,
                    ast::PrimitiveType::Str => TirTypeKind::Str,
                    ast::PrimitiveType::Unit => TirTypeKind::Unit,
                }
            }
            ast::TypeKind::Named(path) => {
                // For now, assume it's a struct. In a real implementation,
                // you'd look up the type in a symbol table
                TirTypeKind::Struct { name: path.join("::") }
            }
            _ => {
                return Err(TlError::internal(
                    format!("Unsupported type kind: {:?}", ast_type.kind)
                ));
            }
        };

        Ok(TirType {
            kind,
            span: ast_type.span,
        })
    }
}

impl Default for TirBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert an AST program to TIR
pub fn ast_to_tir(program: &ast::Program) -> Result<TirModule> {
    let mut builder = TirBuilder::new();
    builder.build_module(program)
}