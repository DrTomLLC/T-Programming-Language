// File: compiler/src/runtime/eval.rs - COMPLETE REWRITE
// -----------------------------------------------------------------------------

//! Complete runtime evaluation system for T-Lang.

use std::collections::HashMap;
use errors::{TlError, Result};
use miette::SourceSpan;
use shared::ast::{Expr, ExprKind, Stmt, StmtKind, Literal, BinaryOp, UnaryOp, Block};

/// Runtime value representation
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Unit,
    Array(Vec<RuntimeValue>),
    Function {
        params: Vec<String>,
        body: Box<Expr>,
        closure: Environment,
    },
}

/// Runtime environment for variable bindings
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    vars: HashMap<String, RuntimeValue>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Environment) -> Self {
        Self {
            vars: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn define(&mut self, name: String, value: RuntimeValue) {
        self.vars.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<RuntimeValue> {
        self.vars.get(name).cloned().or_else(|| {
            self.parent.as_ref().and_then(|p| p.get(name))
        })
    }

    pub fn set(&mut self, name: &str, value: RuntimeValue) -> Result<()> {
        if self.vars.contains_key(name) {
            self.vars.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            parent.set(name, value)
        } else {
            Err(TlError::runtime(
                "<runtime>".to_string(),
                SourceSpan::new(0.into(), 0),
                format!("Undefined variable: {}", name),
            ))
        }
    }
}

/// Complete runtime evaluator
pub struct Evaluator {
    env: Environment,
}

impl Evaluator {
    pub fn new() -> Self {
        let mut env = Environment::new();

        // Add built-in functions
        env.define("print".to_string(), RuntimeValue::Function {
            params: vec!["value".to_string()],
            body: Box::new(Expr::new(ExprKind::Literal(Literal::Unit), SourceSpan::new(0.into(), 0))),
            closure: Environment::new(),
        });

        Self { env }
    }

    /// Evaluate a program (list of statements)
    pub fn eval_program(&mut self, statements: &[Stmt]) -> Result<RuntimeValue> {
        let mut result = RuntimeValue::Unit;

        for stmt in statements {
            result = self.eval_statement(stmt)?;
        }

        Ok(result)
    }

    /// Evaluate a single statement
    pub fn eval_statement(&mut self, stmt: &Stmt) -> Result<RuntimeValue> {
        match &stmt.kind {
            StmtKind::Expr(expr) => self.eval_expression(expr),

            StmtKind::Let { pattern, initializer, .. } => {
                let value = if let Some(init) = initializer {
                    self.eval_expression(init)?
                } else {
                    RuntimeValue::Unit
                };

                // TODO: Handle complex patterns
                if let shared::ast::PatternKind::Ident(name) = &pattern.kind {
                    self.env.define(name.clone(), value);
                }

                Ok(RuntimeValue::Unit)
            }

            StmtKind::Return { value } => {
                if let Some(expr) = value {
                    self.eval_expression(expr)
                } else {
                    Ok(RuntimeValue::Unit)
                }
            }

            StmtKind::If { condition, then_branch, else_branch } => {
                let cond_value = self.eval_expression(condition)?;

                if self.is_truthy(&cond_value) {
                    self.eval_block(then_branch)
                } else if let Some(else_stmt) = else_branch {
                    self.eval_statement(else_stmt)
                } else {
                    Ok(RuntimeValue::Unit)
                }
            }

            StmtKind::While { condition, body } => {
                let mut result = RuntimeValue::Unit;

                while self.is_truthy(&self.eval_expression(condition)?) {
                    result = self.eval_block(body)?;
                }

                Ok(result)
            }

            _ => {
                Err(TlError::runtime(
                    "<runtime>".to_string(),
                    stmt.span,
                    format!("Unsupported statement: {:?}", stmt.kind),
                ))
            }
        }
    }

    /// Evaluate an expression
    pub fn eval_expression(&mut self, expr: &Expr) -> Result<RuntimeValue> {
        match &expr.kind {
            ExprKind::Literal(lit) => self.eval_literal(lit),

            ExprKind::Variable { path } => {
                if path.len() == 1 {
                    let name = &path[0];
                    self.env.get(name).ok_or_else(|| {
                        TlError::runtime(
                            "<runtime>".to_string(),
                            expr.span,
                            format!("Undefined variable: {}", name),
                        )
                    })
                } else {
                    Err(TlError::runtime(
                        "<runtime>".to_string(),
                        expr.span,
                        "Module paths not yet supported in runtime".to_string(),
                    ))
                }
            }

            ExprKind::Binary { left, op, right } => {
                let left_val = self.eval_expression(left)?;
                let right_val = self.eval_expression(right)?;
                self.eval_binary_op(&left_val, op, &right_val, expr.span)
            }

            ExprKind::Unary { op, expr: inner } => {
                let value = self.eval_expression(inner)?;
                self.eval_unary_op(op, &value, expr.span)
            }

            ExprKind::Call { callee, args, .. } => {
                let func = self.eval_expression(callee)?;
                let arg_values: Result<Vec<_>> = args.iter()
                    .map(|arg| self.eval_expression(arg))
                    .collect();
                let arg_values = arg_values?;

                self.call_function(func, arg_values, expr.span)
            }

            ExprKind::If { condition, then_branch, else_branch } => {
                let cond_value = self.eval_expression(condition)?;

                if self.is_truthy(&cond_value) {
                    self.eval_block(then_branch)
                } else if let Some(else_expr) = else_branch {
                    self.eval_expression(else_expr)
                } else {
                    Ok(RuntimeValue::Unit)
                }
            }

            ExprKind::Block(block) => {
                self.eval_block(block)
            }

            _ => {
                Err(TlError::runtime(
                    "<runtime>".to_string(),
                    expr.span,
                    format!("Unsupported expression: {:?}", expr.kind),
                ))
            }
        }
    }

    /// Evaluate a block of statements
    pub fn eval_block(&mut self, block: &Block) -> Result<RuntimeValue> {
        // Create new scope
        let old_env = self.env.clone();
        self.env = Environment::with_parent(self.env.clone());

        let mut result = RuntimeValue::Unit;

        // Execute statements
        for stmt in &block.statements {
            result = self.eval_statement(stmt)?;
        }

        // Evaluate final expression if present
        if let Some(expr) = &block.expr {
            result = self.eval_expression(expr)?;
        }

        // Restore environment
        self.env = old_env;

        Ok(result)
    }

    /// Evaluate literals
    fn eval_literal(&self, lit: &Literal) -> Result<RuntimeValue> {
        match lit {
            Literal::Integer(n) => Ok(RuntimeValue::Integer(*n)),
            Literal::Float(f) => Ok(RuntimeValue::Float(*f)),
            Literal::String(s) => Ok(RuntimeValue::String(s.clone())),
            Literal::Char(c) => Ok(RuntimeValue::Char(*c)),
            Literal::Bool(b) => Ok(RuntimeValue::Bool(*b)),
            Literal::Unit => Ok(RuntimeValue::Unit),
        }
    }

    /// Evaluate binary operations
    fn eval_binary_op(
        &self,
        left: &RuntimeValue,
        op: &BinaryOp,
        right: &RuntimeValue,
        span: SourceSpan,
    ) -> Result<RuntimeValue> {
        match (left, right) {
            (RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                match op {
                    BinaryOp::Add => Ok(RuntimeValue::Integer(l + r)),
                    BinaryOp::Sub => Ok(RuntimeValue::Integer(l - r)),
                    BinaryOp::Mul => Ok(RuntimeValue::Integer(l * r)),
                    BinaryOp::Div => {
                        if *r == 0 {
                            Err(TlError::runtime(
                                "<runtime>".to_string(),
                                span,
                                "Division by zero".to_string(),
                            ))
                        } else {
                            Ok(RuntimeValue::Integer(l / r))
                        }
                    }
                    BinaryOp::Eq => Ok(RuntimeValue::Bool(l == r)),
                    BinaryOp::Ne => Ok(RuntimeValue::Bool(l != r)),
                    BinaryOp::Lt => Ok(RuntimeValue::Bool(l < r)),
                    BinaryOp::Le => Ok(RuntimeValue::Bool(l <= r)),
                    BinaryOp::Gt => Ok(RuntimeValue::Bool(l > r)),
                    BinaryOp::Ge => Ok(RuntimeValue::Bool(l >= r)),
                    _ => Err(TlError::runtime(
                        "<runtime>".to_string(),
                        span,
                        format!("Unsupported integer operation: {:?}", op),
                    )),
                }
            }

            (RuntimeValue::Float(l), RuntimeValue::Float(r)) => {
                match op {
                    BinaryOp::Add => Ok(RuntimeValue::Float(l + r)),
                    BinaryOp::Sub => Ok(RuntimeValue::Float(l - r)),
                    BinaryOp::Mul => Ok(RuntimeValue::Float(l * r)),
                    BinaryOp::Div => Ok(RuntimeValue::Float(l / r)),
                    BinaryOp::Eq => Ok(RuntimeValue::Bool((l - r).abs() < f64::EPSILON)),
                    BinaryOp::Ne => Ok(RuntimeValue::Bool((l - r).abs() >= f64::EPSILON)),
                    BinaryOp::Lt => Ok(RuntimeValue::Bool(l < r)),
                    BinaryOp::Le => Ok(RuntimeValue::Bool(l <= r)),
                    BinaryOp::Gt => Ok(RuntimeValue::Bool(l > r)),
                    BinaryOp::Ge => Ok(RuntimeValue::Bool(l >= r)),
                    _ => Err(TlError::runtime(
                        "<runtime>".to_string(),
                        span,
                        format!("Unsupported float operation: {:?}", op),
                    )),
                }
            }

            (RuntimeValue::String(l), RuntimeValue::String(r)) => {
                match op {
                    BinaryOp::Add => Ok(RuntimeValue::String(format!("{}{}", l, r))),
                    BinaryOp::Eq => Ok(RuntimeValue::Bool(l == r)),
                    BinaryOp::Ne => Ok(RuntimeValue::Bool(l != r)),
                    _ => Err(TlError::runtime(
                        "<runtime>".to_string(),
                        span,
                        format!("Unsupported string operation: {:?}", op),
                    )),
                }
            }

            (RuntimeValue::Bool(l), RuntimeValue::Bool(r)) => {
                match op {
                    BinaryOp::And => Ok(RuntimeValue::Bool(*l && *r)),
                    BinaryOp::Or => Ok(RuntimeValue::Bool(*l || *r)),
                    BinaryOp::Eq => Ok(RuntimeValue::Bool(l == r)),
                    BinaryOp::Ne => Ok(RuntimeValue::Bool(l != r)),
                    _ => Err(TlError::runtime(
                        "<runtime>".to_string(),
                        span,
                        format!("Unsupported boolean operation: {:?}", op),
                    )),
                }
            }

            _ => Err(TlError::runtime(
                "<runtime>".to_string(),
                span,
                format!("Type mismatch in binary operation: {:?} {:?} {:?}", left, op, right),
            )),
        }
    }

    /// Evaluate unary operations
    fn eval_unary_op(
        &self,
        op: &UnaryOp,
        operand: &RuntimeValue,
        span: SourceSpan,
    ) -> Result<RuntimeValue> {
        match (op, operand) {
            (UnaryOp::Neg, RuntimeValue::Integer(n)) => Ok(RuntimeValue::Integer(-n)),
            (UnaryOp::Neg, RuntimeValue::Float(f)) => Ok(RuntimeValue::Float(-f)),
            (UnaryOp::Not, RuntimeValue::Bool(b)) => Ok(RuntimeValue::Bool(!b)),
            (UnaryOp::BitNot, RuntimeValue::Integer(n)) => Ok(RuntimeValue::Integer(!n)),
            _ => Err(TlError::runtime(
                "<runtime>".to_string(),
                span,
                format!("Unsupported unary operation: {:?} {:?}", op, operand),
            )),
        }
    }

    /// Call a function
    fn call_function(
        &mut self,
        func: RuntimeValue,
        args: Vec<RuntimeValue>,
        span: SourceSpan,
    ) -> Result<RuntimeValue> {
        match func {
            RuntimeValue::Function { params, body, closure } => {
                if params.len() != args.len() {
                    return Err(TlError::runtime(
                        "<runtime>".to_string(),
                        span,
                        format!("Expected {} arguments, got {}", params.len(), args.len()),
                    ));
                }

                // Create new environment with closure and parameter bindings
                let old_env = self.env.clone();
                self.env = Environment::with_parent(closure);

                for (param, arg) in params.iter().zip(args.iter()) {
                    self.env.define(param.clone(), arg.clone());
                }

                // Execute function body
                let result = self.eval_expression(&body);

                // Restore environment
                self.env = old_env;

                result
            }
            _ => {
                // Handle built-in functions
                if let RuntimeValue::Function { .. } = func {
                    // This should be handled above, but let's check for built-ins
                    self.call_builtin("unknown", args, span)
                } else {
                    Err(TlError::runtime(
                        "<runtime>".to_string(),
                        span,
                        format!("Cannot call non-function value: {:?}", func),
                    ))
                }
            }
        }
    }

    /// Call built-in functions
    fn call_builtin(
        &self,
        name: &str,
        args: Vec<RuntimeValue>,
        span: SourceSpan,
    ) -> Result<RuntimeValue> {
        match name {
            "print" => {
                if args.len() != 1 {
                    return Err(TlError::runtime(
                        "<runtime>".to_string(),
                        span,
                        "print expects exactly 1 argument".to_string(),
                    ));
                }

                let output = self.value_to_string(&args[0]);
                print!("{}", output);
                Ok(RuntimeValue::Unit)
            }
            "println" => {
                if args.len() != 1 {
                    return Err(TlError::runtime(
                        "<runtime>".to_string(),
                        span,
                        "println expects exactly 1 argument".to_string(),
                    ));
                }

                let output = self.value_to_string(&args[0]);
                println!("{}", output);
                Ok(RuntimeValue::Unit)
            }
            _ => Err(TlError::runtime(
                "<runtime>".to_string(),
                span,
                format!("Unknown built-in function: {}", name),
            )),
        }
    }

    /// Check if a value is truthy
    fn is_truthy(&self, value: &RuntimeValue) -> bool {
        match value {
            RuntimeValue::Bool(b) => *b,
            RuntimeValue::Unit => false,
            RuntimeValue::Integer(0) => false,
            RuntimeValue::Float(f) => *f != 0.0,
            RuntimeValue::String(s) => !s.is_empty(),
            _ => true,
        }
    }

    /// Convert a runtime value to string for display
    fn value_to_string(&self, value: &RuntimeValue) -> String {
        match value {
            RuntimeValue::Integer(n) => n.to_string(),
            RuntimeValue::Float(f) => f.to_string(),
            RuntimeValue::String(s) => s.clone(),
            RuntimeValue::Char(c) => c.to_string(),
            RuntimeValue::Bool(b) => b.to_string(),
            RuntimeValue::Unit => "()".to_string(),
            RuntimeValue::Array(arr) => {
                let elements: Vec<String> = arr.iter()
                    .map(|v| self.value_to_string(v))
                    .collect();
                format!("[{}]", elements.join(", "))
            }
            RuntimeValue::Function { .. } => "<function>".to_string(),
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
