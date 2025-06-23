// File: compiler/src/backends/julia/mod.rs
//! Julia codegen backend: reads our IR debug-text and emits a standalone Julia script
//! that replays the instructions on a simple stack.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct JuliaBackend;

impl Backend<CompiledModule> for JuliaBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // 1. Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // 2. Begin Julia script
        let mut code = String::new();
        code.push_str("function main()\n");
        code.push_str("    stack = Any[]\n");

        // 3. Translate each IR instruction
        for line in ir.lines() {
            let instr = line.trim();
            if instr.is_empty() || instr == "Nop" {
                continue;
            }
            if let Some(n) = instr.strip_prefix("PushInt(").and_then(|s| s.strip_suffix(")")) {
                code.push_str(&format!("    push!(stack, {})\n", n));
            } else if let Some(raw) = instr
                .strip_prefix("PushStr(\"")
                .and_then(|s| s.strip_suffix("\")"))
            {
                // escape backslashes and quotes
                let esc = raw.replace('\\', "\\\\").replace('"', "\\\"");
                code.push_str(&format!("    push!(stack, \"{}\")\n", esc));
            } else if instr == "CallPrint" {
                code.push_str("    if !isempty(stack)\n");
                code.push_str("        v = pop!(stack)\n");
                code.push_str("        print(v)\n");
                code.push_str("    end\n");
            } else {
                return Err(BackendError::Generic(format!("Unknown IR instr: {}", instr)));
            }
        }

        // 4. End and run
        code.push_str("end\n\n");
        code.push_str("main()\n");

        Ok(Box::new(code.into_bytes()))
    }

    fn name(&self) -> &'static str {
        "julia"
    }
}

// Register this backend at startup
static JULIA_REG: Lazy<()> = Lazy::new(|| {
    register_backend(JuliaBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_JULIA_REG: fn() = {
    fn init() {
        Lazy::force(&JULIA_REG);
    }
    init
};
