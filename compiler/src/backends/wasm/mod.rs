// File: compiler/src/backends/wasm/mod.rs
//! WASM codegen backend for T-Lang.
//! Reads our IR debug-text and emits a WebAssembly Text (WAT) module
//! with each IR instruction preserved as a comment.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct WasmBackend;

impl Backend<CompiledModule> for WasmBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // 1. Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // 2. Begin WAT module
        let mut wat = String::new();
        wat.push_str("(module\n");
        wat.push_str("  ;; T-Lang IR embedded as comments\n");

        // 3. Embed each IR instruction as a comment inside the main func
        wat.push_str("  (func $main (result i32)\n");
        for line in ir.lines() {
            let inst = line.trim();
            if inst.is_empty() || inst == "Nop" {
                continue;
            }
            wat.push_str("    ;; ");
            wat.push_str(inst);
            wat.push('\n');
        }
        // 4. Provide a dummy return value so the function is valid
        wat.push_str("    i32.const 0\n");
        wat.push_str("    return\n");
        wat.push_str("  )\n");

        // 5. Export the main function
        wat.push_str("  (export \"main\" (func $main))\n");
        wat.push_str(")\n");

        Ok(Box::new(wat.into_bytes()))
    }

    fn name(&self) -> &'static str {
        "wasm"
    }
}

// Register this backend at startup
static WASM_REG: Lazy<()> = Lazy::new(|| {
    register_backend(WasmBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_WASM_REG: fn() = {
    fn init() {
        Lazy::force(&WASM_REG);
    }
    init
};
