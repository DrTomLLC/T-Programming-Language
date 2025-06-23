//! compiler/src/runtime/eval.rs
//! The T‑Lang interpreter: evaluate a Vec<Value> IR to produce results.

use std::collections::HashMap;
use errors::{TlError};
use miette::SourceSpan;
use crate::codegen::Value;
use errors::parse;
use errors::runtime;

/// A runtime value.
#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Number(f64),
    Bool(bool),
    String(String),
    List(Vec<RuntimeValue>),

    // A user‑defined function closure:
    Function {
        params: Vec<String>,
        body: Vec<Value>,
        env: Env,
    },
    Null,
}

/// An environment frame, mapping names to values.
#[derive(Debug, Clone)]
pub struct Env {
    vars: HashMap<String, RuntimeValue>,
}

impl Env {
    pub fn new() -> Self {
        Env { vars: HashMap::new() }
    }

    pub fn get(&self, name: &str, span: SourceSpan) -> Result<RuntimeValue, TlError> {
        self.vars.get(name)
            .cloned()
            .ok_or_else(|| parse::unexpected_token(                "<runtime>", "", span,
                &format!("variable `{}` not found", name),
                "",
            ))
    }

    pub fn set(&mut self, name: String, val: RuntimeValue) {
        self.vars.insert(name, val);
    }
}

/// The interpreter itself.
pub struct Evaluator {
    env: Env,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator { env: Env::new() }
    }

    /// Evaluate a top‑level statement IR and return its result.
    pub fn eval_stmt(&mut self, v: Value) -> Result<RuntimeValue, TlError> {
        match v {
            Value::Number(n, _) => Ok(RuntimeValue::Number(n)),
            Value::Bool(b, _)   => Ok(RuntimeValue::Bool(b)),
            Value::String(s, _) => Ok(RuntimeValue::String(s)),
            Value::List(vals, _) => {
                let mut out = Vec::new();
                for val in vals {
                    out.push(self.eval_stmt(val)?);
                }
                Ok(RuntimeValue::List(out))
            }

            Value::Add(a,b,span) => self.binary_op(a,*b, span, |x,y| x + y),
            Value::Sub(a,b,span) => self.binary_op(a,*b, span, |x,y| x - y),
            Value::Mul(a,b,span) => self.binary_op(a,*b, span, |x,y| x * y),
            Value::Div(a,b,span) => {
                let right = self.eval_stmt(*b.clone())?;
                if let RuntimeValue::Number(y) = &right {
                    if *y == 0.0 {
                        return Err(runtime::division_by_zero(span));
                    }
                }
                self.binary_op(a,*b, span, |x,y| x / y)
            }

            Value::Less(a,b,span)        => self.bool_cmp(a,*b, span, |x,y| x < y),
            Value::Greater(a,b,span)     => self.bool_cmp(a,*b, span, |x,y| x > y),
            Value::LessEqual(a,b,span)   => self.bool_cmp(a,*b, span, |x,y| x <= y),
            Value::GreaterEqual(a,b,span)=> self.bool_cmp(a,*b, span, |x,y| x >= y),
            Value::EqualEqual(a,b,span)  => self.bool_cmp(a,*b, span, |x,y| (x - y).abs() < f64::EPSILON),
            Value::NotEqual(a,b,span)    => self.bool_cmp(a,*b, span, |x,y| (x - y).abs() >= f64::EPSILON),
            Value::Or(a,b,span)  => {
                let x = self.eval_stmt(*a)?;
                if let RuntimeValue::Bool(true) = x { return Ok(RuntimeValue::Bool(true)); }
                let y = self.eval_stmt(*b)?;
                if let RuntimeValue::Bool(b) = y { return Ok(RuntimeValue::Bool(b)); }
                Err(runtime::generic(span, "`||` applied to non‐bool"))
            }
            Value::Not(a,span)   => {
                let x = self.eval_stmt(*a)?;
                if let RuntimeValue::Bool(b) = x { return Ok(RuntimeValue::Bool(!b)); }
                Err(runtime::generic(span, "`!` applied to non‐bool"))
            }

            Value::GetVar(name, span) => self.env.get(&name, span),
            Value::SetVar(name, boxed, _) => {
                let v = self.eval_stmt(*boxed)?;
                self.env.set(name.clone(), v.clone());
                Ok(v)
            }

            Value::Function { name, params, body, span: _span } => {
                // register the function in the environment as a closure:
                let func = RuntimeValue::Function {
                    params: params.clone(),
                    body: vec![*body.clone()],
                    env: self.env.clone(),
                };
                self.env.set(name.clone(), func.clone());
                Ok(func)
            }

            Value::Call(name, args, span) => {
                let callee = self.env.get(&name, span)?;
                if let RuntimeValue::Function { params, body, env } = callee {
                    if params.len() != args.len() {
                        return Err(runtime::generic(
                            span,
                            &format!("Expected {} args, got {}", params.len(), args.len()),
                        ));
                    }
                    // prepare a fresh frame
                    let mut frame = Env::new();
                    // capture outer locals
                    for (k,v) in env.vars.into_iter() {
                        frame.vars.insert(k, v);
                    }
                    // bind args
                    for (name, val_ir) in params.into_iter().zip(args.into_iter()) {
                        let val = self.eval_stmt(val_ir)?;
                        frame.set(name, val);
                    }
                    // execute body
                    let mut sub_eval = Evaluator { env: frame };
                    let mut result = RuntimeValue::Null;
                    for instr in body {
                        result = sub_eval.eval_stmt(instr)?;
                    }
                    Ok(result)
                } else {
                    Err(runtime::generic(span, &format!("`{}` is not a function", name)))
                }
            }

            Value::If { cond, then_branch, else_branch, span: _span } => {
                let c = self.eval_stmt(*cond)?;
                if let RuntimeValue::Bool(true) = c {
                    self.eval_stmt(*then_branch)
                } else if let Some(else_expr) = else_branch {
                    self.eval_stmt(*else_expr)
                } else {
                    Ok(RuntimeValue::Null)
                }
            }

            Value::Block(stmts, _) => self.eval_block(&stmts),

            Value::Return(boxed, _) => self.eval_stmt(*boxed),

            // catch-all
            other => Err(runtime::generic(
                other.span(),
                &format!("Unsupported IR in evaluator: {:?}", other),
            )),
        }
    }

    fn eval_block(&mut self, stmts: &[Value]) -> Result<RuntimeValue, TlError> {
        let mut last = RuntimeValue::Null;
        for instr in stmts {
            last = self.eval_stmt(instr.clone())?;
        }
        Ok(last)
    }

    fn binary_op<F>(&mut self, a: Box<Value>, b: Value, span: SourceSpan, f: F)
                    -> Result<RuntimeValue, TlError>
    where F: FnOnce(f64,f64)->f64
    {
        let l = self.eval_stmt(*a)?;
        let r = self.eval_stmt(b)?;
        if let (RuntimeValue::Number(x), RuntimeValue::Number(y)) = (l,r) {
            Ok(RuntimeValue::Number(f(x,y)))
        } else {
            Err(runtime::generic(span, "Arithmetic on non‐numbers"))
        }
    }

    fn bool_cmp<F>(&mut self, a: Box<Value>, b: Value, span: SourceSpan, f: F)
                   -> Result<RuntimeValue, TlError>
    where F: FnOnce(f64,f64)->bool
    {
        let l = self.eval_stmt(*a)?;
        let r = self.eval_stmt(b)?;
        if let (RuntimeValue::Number(x), RuntimeValue::Number(y)) = (l,r) {
            Ok(RuntimeValue::Bool(f(x,y)))
        } else {
            Err(runtime::generic(span, "Comparison on non‐numbers"))
        }
    }
}

// expose to outer API
// Remove or comment out this line as it's causing the duplicate definition error
// pub use Evaluator;

