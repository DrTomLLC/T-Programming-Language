// compiler/src/runtime/value.rs
//
// A self‑contained, flat module with no panics, no unwraps, and exhaustive error handling.

// compiler/src/runtime/value.rs
//! Evaluate our IR (`codegen::Value`) into runtime values,
//! with no panics, no unwraps, and exhaustive error handling.

use crate::codegen::Value as IrValue;
use crate::runtime::env::Env;
use errors::TlError;

/// The values our VM understands.
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Str(String),
    // … add more runtime types here as your language grows …
}

/// Top‑level entry: execute a flat sequence of IR instructions.
/// Returns the list of resulting values (one per instruction).
pub fn execute(ir: Vec<IrValue>) -> Result<Vec<Value>, TlError> {
    let mut env = Env::new();
    let mut results = Vec::with_capacity(ir.len());

    for instr in ir {
        let v = eval(instr, &mut env)?;
        results.push(v);
    }

    Ok(results)
}

/// Evaluate one IR instruction in the given environment.
fn eval(instr: IrValue, env: &mut Env) -> Result<Value, TlError> {
    match instr {
        IrValue::LiteralNumber(n, _span) => Ok(Value::Number(n)),
        IrValue::LiteralBool(b, _span)   => Ok(Value::Bool(b)),
        IrValue::LiteralString(s, _span) => Ok(Value::Str(s)),

        IrValue::GetVar(name, _span) => {
            // Lookup must succeed or return a runtime error
            env.get(&name)
        }

        IrValue::SetVar(name, boxed_val, _span) => {
            // Evaluate RHS, bind it, and return it
            let v = eval(*boxed_val, env)?;
            env.set(name.clone(), v.clone());
            Ok(v)
        }

        IrValue::Block(stmts, _span) => {
            // Evaluate each nested instruction in order,
            // return the last one's result (or a default)
            let mut last = Value::Number(0.0);
            for stmt in stmts {
                last = eval(stmt, env)?;
            }
            Ok(last)
        }

        // If you add more IR ops (Add, Sub, Call, etc.), handle them here…

        other => {
            // Exhaustive catch‑all for any unhandled IR variant
            Err(TlError::new(
                "runtime", // source name
                "", // source text (empty since this is a runtime error)
                0..0, // empty span since this is a runtime error
                errors::ErrorCode::RuntimeError, // assuming this error code exists
                format!("unhandled IR instruction in eval(): {:?}", other)
            ))
        }
    }
}
