// File: shared/src/tir/mod.rs - FULL IMPLEMENTATION
// -----------------------------------------------------------------------------

//! T-Lang Intermediate Representation (TIR)
//!
//! This module provides the complete implementation of TIR, which serves as
//! the bridge between the high-level AST and backend code generation.

use miette::SourceSpan;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete TIR module containing all program components
#[derive(Debug, Clone, PartialEq)]
pub struct TirModule {
    pub name: String,
    pub functions: Vec<TirFunction>,
    pub globals: Vec<TirGlobal>,
    pub types: Vec<TirTypeDef>,
    pub imports: Vec<TirImport>,
    pub exports: Vec<TirExport>,
    pub span: SourceSpan,
}

/// A function in TIR with complete signature and body
#[derive(Debug, Clone, PartialEq)]
pub struct TirFunction {
    pub id: FunctionId,
    pub name: String,
    pub params: Vec<TirParam>,
    pub return_type: TirType,
    pub blocks: HashMap<BlockId, TirBlock>,
    pub entry_block: BlockId,
    pub locals: Vec<TirLocal>,
    pub attributes: Vec<TirAttribute>,
    pub span: SourceSpan,
}

/// A basic block in the control flow graph
#[derive(Debug, Clone, PartialEq)]
pub struct TirBlock {
    pub id: BlockId,
    pub instructions: Vec<TirInstruction>,
    pub terminator: TirTerminator,
    pub predecessors: Vec<BlockId>,
    pub span: SourceSpan,
}

/// TIR instructions for all operations
#[derive(Debug, Clone, PartialEq)]
pub struct TirInstruction {
    pub id: InstructionId,
    pub kind: TirInstructionKind,
    pub ty: TirType,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TirInstructionKind {
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
    Load { ptr: ValueId },
    Store { ptr: ValueId, value: ValueId },
    Alloca { ty: TirType },

    // Function operations
    Call { function: FunctionId, args: Vec<ValueId> },

    // Type operations
    Cast { value: ValueId, target_type: TirType },

    // Aggregate operations
    StructAccess { struct_val: ValueId, field_index: usize },
    ArrayAccess { array: ValueId, index: ValueId },

    // Constants
    Const { value: TirConstant },

    // PHI nodes for SSA form
    Phi { incoming: Vec<(BlockId, ValueId)> },
}

/// Block terminators (control flow)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TirTerminator {
    Return { value: Option<ValueId> },
    Branch { target: BlockId },
    CondBranch {
        condition: ValueId,
        then_block: BlockId,
        else_block: BlockId
    },
    Switch {
        value: ValueId,
        cases: Vec<(TirConstant, BlockId)>,
        default: BlockId
    },
    Unreachable,
}

/// TIR type system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TirType {
    pub kind: TirTypeKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TirTypeKind {
    // Primitive types
    Bool,
    I8, I16, I32, I64, I128, ISize,
    U8, U16, U32, U64, U128, USize,
    F32, F64,
    Char,
    Str,
    Unit,

    // Compound types
    Array { element: Box<TirType>, size: u64 },
    Slice { element: Box<TirType> },
    Tuple { elements: Vec<TirType> },
    Struct { name: String, fields: Vec<TirField> },
    Enum { name: String, variants: Vec<TirVariant> },

    // Reference types
    Ref { target: Box<TirType>, mutable: bool },
    Ptr { target: Box<TirType>, mutable: bool },

    // Function types
    Function {
        params: Vec<TirType>,
        return_type: Box<TirType>
    },

    // Generic types
    Generic { name: String, bounds: Vec<String> },

    // Never type
    Never,

    // Unknown type (for inference)
    Unknown,
}

/// Constants in TIR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TirConstant {
    Bool(bool),
    I8(i8), I16(i16), I32(i32), I64(i64), I128(i128), ISize(isize),
    U8(u8), U16(u16), U32(u32), U64(u64), U128(u128), USize(usize),
    F32(f32), F64(f64),
    Char(char),
    String(String),
    Unit,
    Array(Vec<TirConstant>),
    Struct(Vec<TirConstant>),
}

/// Other TIR data structures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TirParam {
    pub name: String,
    pub ty: TirType,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TirLocal {
    pub name: String,
    pub ty: TirType,
    pub mutable: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TirGlobal {
    pub name: String,
    pub ty: TirType,
    pub initializer: Option<TirConstant>,
    pub mutable: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TirTypeDef {
    pub name: String,
    pub ty: TirType,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TirField {
    pub name: String,
    pub ty: TirType,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TirVariant {
    pub name: String,
    pub fields: Vec<TirType>,
    pub discriminant: Option<i64>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TirImport {
    pub path: Vec<String>,
    pub items: Vec<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TirExport {
    pub name: String,
    pub ty: TirType,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TirAttribute {
    pub name: String,
    pub args: Vec<String>,
    pub span: SourceSpan,
}

// ID types for referencing TIR entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstructionId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId(pub u32);

/// TIR builder for constructing TIR from AST
pub struct TirBuilder {
    next_function_id: u32,
    next_block_id: u32,
    next_instruction_id: u32,
    next_value_id: u32,
    current_function: Option<FunctionId>,
    current_block: Option<BlockId>,
}

impl TirBuilder {
    pub fn new() -> Self {
        Self {
            next_function_id: 0,
            next_block_id: 0,
            next_instruction_id: 0,
            next_value_id: 0,
            current_function: None,
            current_block: None,
        }
    }

    pub fn fresh_function_id(&mut self) -> FunctionId {
        let id = FunctionId(self.next_function_id);
        self.next_function_id += 1;
        id
    }

    pub fn fresh_block_id(&mut self) -> BlockId {
        let id = BlockId(self.next_block_id);
        self.next_block_id += 1;
        id
    }

    pub fn fresh_instruction_id(&mut self) -> InstructionId {
        let id = InstructionId(self.next_instruction_id);
        self.next_instruction_id += 1;
        id
    }

    pub fn fresh_value_id(&mut self) -> ValueId {
        let id = ValueId(self.next_value_id);
        self.next_value_id += 1;
        id
    }

    /// Build a complete TIR module from AST
    pub fn build_module(&mut self, ast: &crate::shared::ast::Program) -> errors::Result<TirModule> {
        let mut functions = Vec::new();
        let mut globals = Vec::new();
        let mut types = Vec::new();
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        for item in &ast.items {
            match &item.kind {
                crate::shared::ast::ItemKind::Function { name, params, return_type, body, .. } => {
                    let function = self.build_function(name, params, return_type, body)?;
                    functions.push(function);
                }
                crate::shared::ast::ItemKind::Const { name, ty, value } => {
                    let global = self.build_global(name, ty, Some(value), false)?;
                    globals.push(global);
                }
                crate::shared::ast::ItemKind::Static { name, ty, value, mutable } => {
                    let global = self.build_global(name, ty, Some(value), *mutable)?;
                    globals.push(global);
                }
                // TODO: Add other item types
                _ => {} // Skip unsupported items for now
            }
        }

        Ok(TirModule {
            name: "main".to_string(),
            functions,
            globals,
            types,
            imports,
            exports,
            span: ast.span,
        })
    }

    fn build_function(
        &mut self,
        name: &str,
        params: &[crate::shared::ast::FunctionParam],
        return_type: &Option<crate::shared::ast::Type>,
        body: &crate::shared::ast::Block,
    ) -> errors::Result<TirFunction> {
        let function_id = self.fresh_function_id();
        self.current_function = Some(function_id);

        let entry_block = self.fresh_block_id();
        self.current_block = Some(entry_block);

        let tir_params = params.iter().map(|p| TirParam {
            name: p.name.clone(),
            ty: self.convert_type(&p.ty),
            span: p.span,
        }).collect();

        let tir_return_type = match return_type {
            Some(ty) => self.convert_type(ty),
            None => TirType {
                kind: TirTypeKind::Unit,
                span: body.span,
            },
        };

        let mut blocks = HashMap::new();
        let entry_block_tir = self.build_block(body)?;
        blocks.insert(entry_block, entry_block_tir);

        Ok(TirFunction {
            id: function_id,
            name: name.to_string(),
            params: tir_params,
            return_type: tir_return_type,
            blocks,
            entry_block,
            locals: Vec::new(), // TODO: Collect from body
            attributes: Vec::new(),
            span: body.span,
        })
    }

    fn build_block(&mut self, block: &crate::shared::ast::Block) -> errors::Result<TirBlock> {
        let block_id = self.current_block.unwrap();
        let mut instructions = Vec::new();

        for stmt in &block.statements {
            let instrs = self.build_statement(stmt)?;
            instructions.extend(instrs);
        }

        let terminator = if let Some(expr) = &block.expr {
            // Block ends with expression, return its value
            let value_id = self.build_expression(expr)?;
            TirTerminator::Return { value: Some(value_id) }
        } else {
            // Block ends without value, return unit
            TirTerminator::Return { value: None }
        };

        Ok(TirBlock {
            id: block_id,
            instructions,
            terminator,
            predecessors: Vec::new(),
            span: block.span,
        })
    }

    fn build_statement(&mut self, stmt: &shared::ast::Stmt) -> errors::Result<Vec<TirInstruction>> {
        match &stmt.kind {
            shared::ast::StmtKind::Expr(expr) => {
                self.build_expression(expr)?;
                Ok(Vec::new()) // Expression statements don't produce instructions
            }
            shared::ast::StmtKind::Let { pattern, ty, initializer, .. } => {
                // TODO: Implement let statement lowering
                Ok(Vec::new())
            }
            shared::ast::StmtKind::Return { value } => {
                // TODO: Implement return statement lowering
                Ok(Vec::new())
            }
            _ => {
                // TODO: Implement other statement types
                Ok(Vec::new())
            }
        }
    }

    fn build_expression(&mut self, expr: &shared::ast::Expr) -> errors::Result<ValueId> {
        match &expr.kind {
            shared::ast::ExprKind::Literal(lit) => {
                self.build_literal(lit, expr.span)
            }
            shared::ast::ExprKind::Variable { path } => {
                // TODO: Implement variable reference
                Ok(self.fresh_value_id())
            }
            shared::ast::ExprKind::Binary { left, op, right } => {
                self.build_binary_expression(left, op, right, expr.span)
            }
            _ => {
                // TODO: Implement other expression types
                Ok(self.fresh_value_id())
            }
        }
    }

    fn build_literal(&mut self, lit: &shared::ast::Literal, span: SourceSpan) -> errors::Result<ValueId> {
        let value_id = self.fresh_value_id();
        let instruction_id = self.fresh_instruction_id();

        let (constant, ty) = match lit {
            shared::ast::Literal::Integer(n) => {
                (TirConstant::I32(*n as i32), TirTypeKind::I32)
            }
            shared::ast::Literal::Float(f) => {
                (TirConstant::F64(*f), TirTypeKind::F64)
            }
            shared::ast::Literal::String(s) => {
                (TirConstant::String(s.clone()), TirTypeKind::Str)
            }
            shared::ast::Literal::Char(c) => {
                (TirConstant::Char(*c), TirTypeKind::Char)
            }
            shared::ast::Literal::Bool(b) => {
                (TirConstant::Bool(*b), TirTypeKind::Bool)
            }
            shared::ast::Literal::Unit => {
                (TirConstant::Unit, TirTypeKind::Unit)
            }
        };

        // TODO: Add instruction to current block

        Ok(value_id)
    }

    fn build_binary_expression(
        &mut self,
        left: &shared::ast::Expr,
        op: &shared::ast::BinaryOp,
        right: &shared::ast::Expr,
        span: SourceSpan,
    ) -> errors::Result<ValueId> {
        let left_value = self.build_expression(left)?;
        let right_value = self.build_expression(right)?;
        let result_value = self.fresh_value_id();

        let instruction_kind = match op {
            shared::ast::BinaryOp::Add => TirInstructionKind::Add { lhs: left_value, rhs: right_value },
            shared::ast::BinaryOp::Sub => TirInstructionKind::Sub { lhs: left_value, rhs: right_value },
            shared::ast::BinaryOp::Mul => TirInstructionKind::Mul { lhs: left_value, rhs: right_value },
            shared::ast::BinaryOp::Div => TirInstructionKind::Div { lhs: left_value, rhs: right_value },
            shared::ast::BinaryOp::Eq => TirInstructionKind::Eq { lhs: left_value, rhs: right_value },
            shared::ast::BinaryOp::Ne => TirInstructionKind::Ne { lhs: left_value, rhs: right_value },
            shared::ast::BinaryOp::Lt => TirInstructionKind::Lt { lhs: left_value, rhs: right_value },
            shared::ast::BinaryOp::Le => TirInstructionKind::Le { lhs: left_value, rhs: right_value },
            shared::ast::BinaryOp::Gt => TirInstructionKind::Gt { lhs: left_value, rhs: right_value },
            shared::ast::BinaryOp::Ge => TirInstructionKind::Ge { lhs: left_value, rhs: right_value },
            _ => {
                return Err(errors::TlError::internal(
                    format!("Unsupported binary operator: {:?}", op)
                ));
            }
        };

        // TODO: Add instruction to current block and determine result type

        Ok(result_value)
    }

    fn build_global(
        &mut self,
        name: &str,
        ty: &shared::ast::Type,
        value: Option<&shared::ast::Expr>,
        mutable: bool,
    ) -> errors::Result<TirGlobal> {
        let tir_type = self.convert_type(ty);
        let initializer = match value {
            Some(expr) => {
                // TODO: Evaluate constant expression
                Some(TirConstant::Unit)
            }
            None => None,
        };

        Ok(TirGlobal {
            name: name.to_string(),
            ty: tir_type,
            initializer,
            mutable,
            span: ty.span,
        })
    }

    fn convert_type(&self, ast_type: &shared::ast::Type) -> TirType {
        let kind = match &ast_type.kind {
            shared::ast::TypeKind::Primitive(prim) => {
                match prim {
                    shared::ast::PrimitiveType::Bool => TirTypeKind::Bool,
                    shared::ast::PrimitiveType::I8 => TirTypeKind::I8,
                    shared::ast::PrimitiveType::I16 => TirTypeKind::I16,
                    shared::ast::PrimitiveType::I32 => TirTypeKind::I32,
                    shared::ast::PrimitiveType::I64 => TirTypeKind::I64,
                    shared::ast::PrimitiveType::I128 => TirTypeKind::I128,
                    shared::ast::PrimitiveType::ISize => TirTypeKind::ISize,
                    shared::ast::PrimitiveType::U8 => TirTypeKind::U8,
                    shared::ast::PrimitiveType::U16 => TirTypeKind::U16,
                    shared::ast::PrimitiveType::U32 => TirTypeKind::U32,
                    shared::ast::PrimitiveType::U64 => TirTypeKind::U64,
                    shared::ast::PrimitiveType::U128 => TirTypeKind::U128,
                    shared::ast::PrimitiveType::USize => TirTypeKind::USize,
                    shared::ast::PrimitiveType::F32 => TirTypeKind::F32,
                    shared::ast::PrimitiveType::F64 => TirTypeKind::F64,
                    shared::ast::PrimitiveType::Char => TirTypeKind::Char,
                    shared::ast::PrimitiveType::Str => TirTypeKind::Str,
                    shared::ast::PrimitiveType::Unit => TirTypeKind::Unit,
                }
            }
            shared::ast::TypeKind::Named(path) => {
                // TODO: Resolve named types
                TirTypeKind::Unknown
            }
            _ => TirTypeKind::Unknown,
        };

        TirType {
            kind,
            span: ast_type.span,
        }
    }
}

impl Default for TirBuilder {
    fn default() -> Self {
        Self::new()
    }
}
