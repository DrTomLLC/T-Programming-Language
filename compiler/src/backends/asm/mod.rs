// File: compiler/src/backends/asm/mod.rs
//! Assembly codegen backend: reads our IR debug‐text and emits
//! an x86-64 assembly listing with the IR instructions preserved as comments.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct AsmBackend;

impl Backend<CompiledModule> for AsmBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // 1. Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // 2. Begin assembly listing
        let mut asm = String::new();
        asm.push_str("    .section .text\n");
        asm.push_str("    .global _start\n");
        asm.push_str("_start:\n");

        // 3. Emit each IR instruction as a comment
        for line in ir.lines() {
            let inst = line.trim();
            if inst.is_empty() || inst == "Nop" {
                continue;
            }
            asm.push_str("    # ");
            asm.push_str(inst);
            asm.push('\n');
        }

        // 4. Exit via syscall
        asm.push_str("    mov rax, 60      # syscall: exit\n");
        asm.push_str("    xor rdi, rdi     # status 0\n");
        asm.push_str("    syscall\n");

        Ok(Box::new(asm.into_bytes()))
    }

    fn name(&self) -> &'static str {
        "asm"
    }
}

// Register this backend at startup
static ASM_REG: Lazy<()> = Lazy::new(|| {
    register_backend(AsmBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_ASM_REG: fn() = {
    fn init() {
        Lazy::force(&ASM_REG);
    }
    init
};
