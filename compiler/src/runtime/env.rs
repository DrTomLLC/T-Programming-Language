/// compiler/src/runtime/env.rs
///
/// Environment for T‑Lang evaluator: holds variable bindings and an optional parent for nested scopes.

use std::collections::HashMap;
use crate::runtime::error::RuntimeError;
use crate::runtime::value::Value;

/// A mapping from variable names to values, with optional parent for nested scopes.
#[derive(Clone)]
pub struct Environment {
    variables: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    /// Create a brand‑new (global) environment.
    pub fn new() -> Self {
        Environment {
            variables: HashMap::new(),
            parent: None,
        }
    }

    /// Create a new child environment with this one as its parent.
    pub fn with_parent(parent: &Environment) -> Self {
        Environment {
            variables: HashMap::new(),
            parent: Some(Box::new(parent.clone())),
        }
    }

    /// Define a new variable in the _current_ scope.
    pub fn define(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// Assign to an existing variable; if not found in this scope, walk up to the parent.
    pub fn assign(&mut self, name: String, value: Value) -> Result<(), RuntimeError> {
        // If the variable exists in this scope, overwrite it
        if let std::collections::hash_map::Entry::Occupied(mut e) =
            self.variables.entry(name.clone())
        {
            e.insert(value);
            return Ok(());
        }
        // Otherwise, recurse into the parent if there is one
        if let Some(parent) = &mut self.parent {
            return parent.assign(name, value);
        }
        // Not found anywhere
        Err(RuntimeError::UndefinedVariable(name))
    }

    /// Look up a variable; if not found here, walk up to the parent.
    pub fn get(&self, name: &str) -> Result<Value, RuntimeError> {
        if let Some(val) = self.variables.get(name) {
            Ok(val.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            Err(RuntimeError::UndefinedVariable(name.to_string()))
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
