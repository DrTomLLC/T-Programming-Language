// compiler/src/runtime/eval.rs

use std::collections::HashMap;
use Expr::{Block, Call, Grouping, If, ListLiteral, Variable};
use crate::runtime::env::Environment;
use crate::runtime::error::RuntimeError;
use crate::runtime::value::Value;
use shared::ast::{Expr, Stmt, UnaryOp, BinaryOp};

pub struct Interpreter {
    env: HashMap<String, Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self { env: HashMap::new() }
    }

    pub fn eval_expr(&mut self, expr: Expr, depth: usize) -> Result<Value, RuntimeError> {
        eprintln!("[DEBUG] eval_expr at depth {}: {:?}", depth, expr);
        match expr {
            Expr::LiteralNumber(n) => Ok(Value::Number(n)),
            Expr::LiteralBool(b) => Ok(Value::Bool(b)),
            Expr::LiteralString(s) => Ok(Value::String(s)),
            Expr::Unary { op, expr } => {
                let val = self.eval_expr(*expr, depth + 1)?;
                eprintln!("[DEBUG] Unary {:?} on {:?}", op, val);
                match op {
                    UnaryOp::Negate => match val {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err(RuntimeError::TypeError("Expected number".into())),
                    },
                    UnaryOp::Not => match val {
                        Value::Bool(b) => Ok(Value::Bool(!b)),
                        _ => Err(RuntimeError::TypeError("Expected bool".into())),
                    },
                }
            },
            Expr::Binary { left, op, right } => {
                let l = self.eval_expr(*left, depth + 1)?;
                let r = self.eval_expr(*right, depth + 1)?;
                eprintln!("[DEBUG] Binary {:?} on {:?} and {:?}", op, l, r);

                match (l, r) {
                    (Value::Number(a), Value::Number(b)) => match op {
                        BinaryOp::Add => Ok(Value::Number(a + b)),
                        BinaryOp::Sub => Ok(Value::Number(a - b)),
                        BinaryOp::Mul => Ok(Value::Number(a * b)),
                        BinaryOp::Div => Ok(Value::Number(a / b)),
                        BinaryOp::EqualEqual => Ok(Value::Bool(a == b)),
                        BinaryOp::NotEqual => Ok(Value::Bool(a != b)),
                        _ => Err(RuntimeError::TypeError("Unsupported binary op for numbers".into())),
                    },
                    (Value::Bool(a), Value::Bool(b)) => match op {
                        BinaryOp::EqualEqual => Ok(Value::Bool(a == b)),
                        BinaryOp::NotEqual => Ok(Value::Bool(a != b)),
                        _ => Err(RuntimeError::TypeError("Unsupported binary op for bools".into())),
                    },
                    (Value::String(a), Value::String(b)) => match op {
                        BinaryOp::Add => Ok(Value::String(a + &b)),
                        BinaryOp::EqualEqual => Ok(Value::Bool(a == b)),
                        BinaryOp::NotEqual => Ok(Value::Bool(a != b)),
                        _ => Err(RuntimeError::TypeError("Unsupported binary op for strings".into())),
                    },
                    _ => Err(RuntimeError::TypeError("Mismatched operand types".into())),
                }
            },
            Expr::Variable(name) => {
                eprintln!("[DEBUG] Variable lookup: {}", name);
                self.env.get(&name).cloned().ok_or_else(|| RuntimeError::UndefinedVariable(name))
            },
            Expr::Grouping(expr) => {
                eprintln!("[DEBUG] Grouping expression");
                self.eval_expr(*expr, depth + 1)
            },
            _ => {
                eprintln!("[DEBUG] Unsupported expression type: {:?}", expr);
                Err(RuntimeError::TypeError("Unsupported expression".into()))
            },
        }
    }

    pub fn eval_stmt(&mut self, stmt: Stmt, depth: usize) -> Result<Value, RuntimeError> {
        eprintln!("[DEBUG] eval_stmt at depth {}: {:?}", depth, stmt);
        match stmt {
            Stmt::Expr(expr) => self.eval_expr(expr, depth + 1),
            _ => Err(RuntimeError::TypeError("Unsupported statement type".into())),
        }
    }

    pub fn env(&self) -> &HashMap<String, Value> {
        &self.env
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Bool(b) => *b,
        Value::Number(n) => *n != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::Function(_, _) => true,
    }
}
