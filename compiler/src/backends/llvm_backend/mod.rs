// File: compiler/src/backends/llvm_backend/mod.rs
//! LLVM IR backend for T-Lang.
//! Reads our IR debug-text and emits a minimal LLVM IR module
//! with each T-Lang IR instruction preserved as an LLVM comment.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct LlvmBackend;

impl Backend<CompiledModule> for LlvmBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // 1. Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // 2. Begin LLVM IR module
        let mut code = String::new();
        code.push_str("; ModuleID = 'tlang'\n");
        code.push_str("declare i32 @printf(i8*, ...)\n");
        code.push_str("@.str = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\", align 1\n\n");
        code.push_str("define i32 @main() {\n");
        code.push_str("entry:\n");

        // 3. Embed each T-Lang IR instruction as an LLVM comment
        for line in ir.lines() {
            let inst = line.trim();
            if inst.is_empty() || inst == "Nop" {
                continue;
            }
            code.push_str("  ; ");
            code.push_str(inst);
            code.push('\n');
        }

        // 4. Return 0
        code.push_str("  ret i32 0\n");
        code.push_str("}\n");

        Ok(Box::new(code.into_bytes()))
    }

    fn name(&self) -> &'static str {
        "llvm"
    }
}

// Register this backend at startup
static LLVM_REG: Lazy<()> = Lazy::new(|| {
    register_backend(LlvmBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_LLVM_REG: fn() = {
    fn init() {
        Lazy::force(&LLVM_REG);
    }
    init
};
