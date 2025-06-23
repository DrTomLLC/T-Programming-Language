/// compiler/src/runtime/env.rs
///
/// A runtime environment mapping names to values.
use std::collections::HashMap;
use errors::TlError;
#[derive(Debug, Default)]
pub struct Env {
    vars: HashMap<String, Value>,
}

impl Env {
    pub fn new() -> Self {
        Env { vars: HashMap::new() }
    }

    pub fn get(&self, name: &str) -> Result<Value, TlError> {
        self.vars
            .get(name)
            .cloned()
            .ok_or_else(|| TlError::new("", "", 0, errors::ErrorCode::UndefinedVariable, format!("Undefined variable: {}", name)))
    }

    pub fn set(&mut self, name: String, val: Value) {
        self.vars.insert(name, val);
    }
}

// Re‑export runtime Value so users can do `use compiler::runtime::{Env, Value};`
pub use crate::runtime::value::Value;
