// File: compiler/src/backends/cranelift/mod.rs
//! Cranelift IR “backend” for T-Lang.
//! Reads our IR debug-text and emits a minimal Cranelift function
//! that embeds each instruction as a comment.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct CraneliftBackend;

impl Backend<CompiledModule> for CraneliftBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // 1. Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // 2. Begin Cranelift IR
        let mut code = String::new();
        code.push_str(";; T-Lang Cranelift IR (debug)\n");
        code.push_str("function %main() -> i64 {\n");
        code.push_str("ebb0:\n");

        // 3. Emit each IR instruction as a comment
        for line in ir.lines() {
            let inst = line.trim();
            if inst.is_empty() || inst == "Nop" {
                continue;
            }
            code.push_str("    ;; ");
            code.push_str(inst);
            code.push('\n');
        }

        // 4. A dummy return value to make the function valid
        code.push_str("    v0 = iconst.i64 0\n");
        code.push_str("    return\n");
        code.push_str("}\n");

        Ok(Box::new(code.into_bytes()))
    }

    fn name(&self) -> &'static str {
        "cranelift"
    }
}

// Register this backend at startup
static CRANELIFT_REG: Lazy<()> = Lazy::new(|| {
    register_backend(CraneliftBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_CRANELIFT_REG: fn() = {
    fn init() {
        Lazy::force(&CRANELIFT_REG);
    }
    init
};
