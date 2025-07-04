// File: compiler/src/backends/ruby/mod.rs
//! Ruby codegen backend for T-Lang.
//! Reads our IR debug-text and emits a standalone Ruby script
//! that replays the instructions on two simple stacks and prints values.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct RubyBackend;

impl Backend<CompiledModule> for RubyBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // 1. Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // 2. Begin Ruby script
        let mut code = String::new();
        code.push_str("# Generated by T-Lang Ruby backend\n");
        code.push_str("int_stack = []\n");
        code.push_str("str_stack = []\n\n");

        // 3. Translate each IR instruction
        for line in ir.lines() {
            let instr = line.trim();
            if instr.is_empty() || instr == "Nop" {
                continue;
            }

            if let Some(n) = instr.strip_prefix("PushInt(").and_then(|s| s.strip_suffix(")")) {
                code.push_str(&format!("int_stack.push({})\n", n));
            } else if let Some(raw) = instr
                .strip_prefix("PushStr(\"")
                .and_then(|s| s.strip_suffix("\")"))
            {
                let esc = raw.replace('\\', "\\\\").replace('"', "\\\"");
                code.push_str(&format!("str_stack.push(\"{}\")\n", esc));
            } else if instr == "CallPrint" {
                code.push_str(
                    "if !str_stack.empty?\n\
                     \tprint str_stack.pop\n\
                     elsif !int_stack.empty?\n\
                     \tprint int_stack.pop\n\
                     end\n",
                );
            } else {
                return Err(BackendError::Generic(format!("Unknown IR instr: {}", instr)));
            }
        }

        // 4. End of script
        code.push_str("\n");
        Ok(Box::new(code.into_bytes()))
    }

    fn name(&self) -> &'static str {
        "ruby"
    }
}

// Register this backend at startup
static RUBY_REG: Lazy<()> = Lazy::new(|| {
    register_backend(RubyBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_RUBY_REG: fn() = {
    fn init() {
        Lazy::force(&RUBY_REG);
    }
    init
};
