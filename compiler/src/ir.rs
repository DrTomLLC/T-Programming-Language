// File: compiler/src/ir.rs
// IR lowering from AST `Program` into plugin_api::CompiledModule and Instruction.

use plugin_api::{CompiledModule, Instruction};
use crate::parser::Program;
use crate::ast::{Stmt, Expr, ExprKind, Literal};

/// Lower a full program into a CompiledModule (instructions-only here).
pub fn lower_program(prog: &Program) -> CompiledModule {
    let mut instrs = Vec::new();
    // Walk through each top-level statement
    for stmt in &prog.statements {
        lower_stmt(stmt, &mut instrs);
    }
    // bytecode Vec<u8> can be filled by a serializer later; empty for now
    CompiledModule::new(Vec::new(), instrs)
}

/// Lower a single statement into instructions.
fn lower_stmt(stmt: &Stmt, instrs: &mut Vec<Instruction>) {
    match stmt {
        Stmt::Expr { expr, .. } | Stmt::Semi { expr, .. } => {
            lower_expr(expr, instrs);
        }
        // Other Stmt variants (Local, Item, etc.) currently no-op
        _ => {}
    }
}

/// Lower an expression into instructions.
fn lower_expr(expr: &Expr, instrs: &mut Vec<Instruction>) {
    match &expr.kind {
        ExprKind::Literal(lit, _) => match lit {
            Literal::Integer(i) => {
                // safely convert i128 to i64, skipping out-of-range
                if let Ok(v) = i64::try_from(*i) {
                    instrs.push(Instruction::PushInt(v));
                }
            }
            Literal::String(s) => {
                instrs.push(Instruction::PushStr(s.clone()));
            }
            _ => {}
        },
        ExprKind::Call { func, args, .. } => {
            lower_expr(func, instrs);
            for arg in args {
                lower_expr(arg, instrs);
            }
            instrs.push(Instruction::CallFunction);
        }
        // Nested grouping, binary, etc.
        ExprKind::Binary { left, op, right, .. } => {
            lower_expr(left, instrs);
            lower_expr(right, instrs);
            // You can push other op-specific instructions here
        }
        _ => {}
    }
}
