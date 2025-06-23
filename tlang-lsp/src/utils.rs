// tlang-lsp/src/utils.rs
// Utility functions for mapping between LSP positions, offsets, and AST

//! File: compiler/src/ir.rs
//!
//! Intermediate representation (IR) instructions emitted by the compiler backend.

use shared::ast::{AST, Expr, ExprKind, Literal, Stmt};

/// A low‑level instruction in the compiler IR.
#[derive(Debug, Clone, PartialEq)]
pub enum Instr {
    /// Push a string literal onto the stack.
    PushStr(String),
    /// Push an integer literal onto the stack.
    PushInt(i64),
    /// Push a float literal onto the stack.
    PushFloat(f64),
    /// Push a boolean literal onto the stack.
    PushBool(bool),
    /// Call the built‑in print function.
    CallPrint,
    // TODO: extend with other instructions as needed
}

/// Lower an AST into IR instructions.
pub fn lower_program(ast: &AST) -> Vec<Instr> {
    let mut instrs = Vec::new();
    for stmt in &ast.items {
        match stmt {
            Stmt::Expr { expr, .. } | Stmt::Semi { expr, .. } => {
                lower_expr(expr, &mut instrs);
                // If this was a call to `print`, emit the print instruction
                if let ExprKind::Call { func, .. } = &expr.kind {
                    if let ExprKind::Variable(path, _) = &func.kind {
                        if path == &vec!["print".to_string()] {
                            instrs.push(Instr::CallPrint);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    instrs
}

fn lower_expr(expr: &Expr, instrs: &mut Vec<Instr>) {
    match &expr.kind {
        ExprKind::Literal(lit, _) => match lit {
            Literal::String(s) => instrs.push(Instr::PushStr(s.clone())),
            Literal::Integer(i) => instrs.push(Instr::PushInt(*i as i64)),
            Literal::Float(f) => instrs.push(Instr::PushFloat(*f)),
            Literal::Boolean(b) => instrs.push(Instr::PushBool(*b)),
            _ => {}
        },
        ExprKind::Grouping(inner, _) => lower_expr(inner, instrs),
        _ => {}
    }
}

/// A compiled module: its emitted instructions along with raw bytecode.
#[derive(Debug, Clone)]
pub struct CompiledModule {
    /// Sequence of IR instructions.
    pub instructions: Vec<Instr>,
    /// Raw bytecode for VM execution.
    pub bytecode: Vec<u8>,
}

impl CompiledModule {
    /// Create a new module from IR and bytecode.
    pub fn new(instructions: Vec<Instr>, bytecode: Vec<u8>) -> Self {
        Self { instructions, bytecode }
    }
}
