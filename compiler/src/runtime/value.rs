// compiler/src/runtime/value.rs

use crate::runtime::error::RuntimeError;

/// Trait for callable values
pub trait Callable: Fn(Vec<Value>) -> Result<Value, RuntimeError> + Send + Sync {
    fn clone_box(&self) -> Box<dyn Callable>;
}

impl<T> Callable for T
where
    T: Fn(Vec<Value>) -> Result<Value, RuntimeError> + Clone + Send + Sync + 'static,
{
    fn clone_box(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Callable> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub enum Value {
    Number(f64),
    Bool(bool),
    String(String),
    Function(Vec<String>, Box<dyn Callable>),
}

// Manual Debug (excludes closure internals)
impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "Number({})", n),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::String(s) => write!(f, "String({:?})", s),
            Value::Function(_, _) => write!(f, "<function>"),
        }
    }
}

// Manual Clone (closure cloning via trait)
impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Number(n) => Value::Number(*n),
            Value::Bool(b) => Value::Bool(*b),
            Value::String(s) => Value::String(s.clone()),
            Value::Function(params, func) => Value::Function(params.clone(), func.clone()),
        }
    }
}

// Manual PartialEq (ignores function equality)
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Function(_, _), Value::Function(_, _)) => false, // or true if comparing structure
            _ => false,
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Function(_, _) => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::Bool(_) => "bool",
            Value::String(_) => "string",
            Value::Function(_, _) => "function",
        }
    }
}
