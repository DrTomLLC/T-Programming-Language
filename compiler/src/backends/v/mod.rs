// File: compiler/src/backends/v/mod.rs
//! V codegen backend for T-Lang.
//! Reads our IR debug-text and emits a standalone V program
//! that replays the instructions on two stacks and prints values.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct VBackend;

impl Backend<CompiledModule> for VBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // 1. Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // 2. Begin V source
        let mut code = String::new();
        code.push_str("module main\n\n");
        code.push_str("fn main() {\n");
        code.push_str("\tmut int_stack := []int{}\n");
        code.push_str("\tmut str_stack := []string{}\n\n");

        // 3. Translate each IR instruction
        for line in ir.lines() {
            let instr = line.trim();
            if instr.is_empty() || instr == "Nop" {
                continue;
            }
            if let Some(n) = instr.strip_prefix("PushInt(").and_then(|s| s.strip_suffix(")")) {
                code.push_str(&format!("\tint_stack << {}\n", n));
            } else if let Some(raw) = instr
                .strip_prefix("PushStr(\"")
                .and_then(|s| s.strip_suffix("\")"))
            {
                let esc = raw.replace('\\', "\\\\").replace('"', "\\\"");
                code.push_str(&format!("\tstr_stack << \"{}\"\n", esc));
            } else if instr == "CallPrint" {
                code.push_str(
                    r#"    if str_stack.len > 0 {
        print(str_stack.pop())
    } else if int_stack.len > 0 {
        print(int_stack.pop().str())
    }
"#,
                );
            } else {
                return Err(BackendError::Generic(format!("Unknown IR instr: {}", instr)));
            }
        }

        // 4. End main
        code.push_str("}\n");

        Ok(Box::new(code.into_bytes()))
    }

    fn name(&self) -> &'static str {
        "v"
    }
}

// Register this backend at startup
static V_REG: Lazy<()> = Lazy::new(|| {
    register_backend(VBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_V_REG: fn() = {
    fn init() {
        Lazy::force(&V_REG);
    }
    init
};
