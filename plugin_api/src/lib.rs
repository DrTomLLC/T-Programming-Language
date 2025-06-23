//! Defines the `CompiledModule` IR and backend registration for T‑Lang.
//!
//! This crate exposes:
//! - `CompiledModule`: holds raw bytecode and structured instructions.
//! - `Instruction`: an enum of bytecode operations.
//! - `Backend` trait: for pluggable codegen backends.
//! - Registration functions to register and list backends.

use once_cell::sync::Lazy;
use std::{any::Any, sync::Mutex};
use thiserror::Error;

/// Instructions that the front‑end emits as IR (a.k.a. "bytecode").
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Push a UTF‑8 string literal.
    PushStr(String),
    /// Call the standard print function.
    CallPrint,
    /// Push a 64‑bit integer literal.
    PushInt(i64),
    // TODO: add other instructions as needed, e.g. CallAdd, etc.
}

/// A compiled module, pairing raw bytes with a sequence of high‑level instructions.
#[derive(Clone, Debug)]
pub struct CompiledModule {
    /// Raw bytecode representation.
    pub bytecode: Vec<u8>,
    /// High‑level decoded instructions.
    pub instructions: Vec<Instruction>,
}

impl CompiledModule {
    /// Create a new `CompiledModule` from raw bytes and decoded instructions.
    pub fn new(bytecode: Vec<u8>, instructions: Vec<Instruction>) -> Self {
        CompiledModule { bytecode, instructions }
    }

    /// Access the sequence of instructions.
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }
}

/// Errors returned by backends.
#[derive(Error, Debug)]
pub enum BackendError {
    #[error("generic backend error: {0}")]
    Generic(String),
}

/// The pluggable backend interface: transform a `CompiledModule` into some IR.
pub trait Backend<M>: Send + Sync {
    /// The backend‑specific IR type.
    type ModuleIr: Any + Send + Sync;
    /// Compile or transform the given module, returning backend IR or an error.
    fn compile(&self, module: M) -> Result<Self::ModuleIr, BackendError>;
    /// A human‑readable backend name.
    fn name(&self) -> &'static str;
}

/// Registry of all available backends.
static BACKENDS: Lazy<Mutex<Vec<&'static dyn Backend<CompiledModule, ModuleIr = Box<dyn Any + Send + Sync>>>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

/// Register a new backend at startup.
pub fn register_backend<B>(backend: B)
where
    B: Backend<CompiledModule, ModuleIr = Box<dyn Any + Send + Sync>> + 'static,
{
    // Leak the backend so it lives for the program duration.
    let boxed: Box<dyn Backend<CompiledModule, ModuleIr = Box<dyn Any + Send + Sync>>> = Box::new(backend);
    let static_ref: &'static dyn Backend<CompiledModule, ModuleIr = Box<dyn Any + Send + Sync>> = Box::leak(boxed);
    BACKENDS.lock().unwrap().push(static_ref);
}

/// List all registered backends.
pub fn list_backends(
) -> Vec<&'static dyn Backend<CompiledModule, ModuleIr = Box<dyn Any + Send + Sync>>> {
    BACKENDS.lock().unwrap().clone()
}
