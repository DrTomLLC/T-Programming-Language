// File: compiler/src/backends/elixir/mod.rs
//! Elixir codegen backend: reads our IR debug-text and emits a standalone Elixir script
//! that replays the instructions on a simple list‐based stack.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct ElixirBackend;

impl Backend<CompiledModule> for ElixirBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // 1. Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // 2. Begin Elixir module
        let mut code = String::new();
        code.push_str("defmodule Tlang do\n");
        code.push_str("  def main do\n");
        code.push_str("    stack = []\n\n");

        // 3. Replay each IR instruction
        for line in ir.lines() {
            let instr = line.trim();
            if instr.is_empty() || instr == "Nop" {
                continue;
            }

            if let Some(n) = instr.strip_prefix("PushInt(").and_then(|s| s.strip_suffix(")")) {
                code.push_str(&format!("    stack = [ {} | stack ]\n", n));
            } else if let Some(raw) = instr
                .strip_prefix("PushStr(\"")
                .and_then(|s| s.strip_suffix("\")"))
            {
                let esc = raw.replace('\\', "\\\\").replace('"', "\\\"");
                code.push_str(&format!(
                    "    stack = [ \"{}\" | stack ]\n",
                    esc
                ));
            } else if instr == "CallPrint" {
                code.push_str(
                    "    stack = case stack do\n\
                     \n      [h | t] -> IO.write(to_string(h)); t\n\
                     \n      []      -> stack\n\
                     end\n\n",
                );
            } else {
                return Err(BackendError::Generic(format!("Unknown IR instr: {}", instr)));
            }
        }

        // 4. End module and invoke
        code.push_str("  end\nend\n\nTlang.main()\n");

        Ok(Box::new(code.into_bytes()))
    }

    fn name(&self) -> &'static str {
        "elixir"
    }
}

// Register this backend at startup
static ELIXIR_REG: Lazy<()> = Lazy::new(|| {
    register_backend(ElixirBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_ELIXIR_REG: fn() = {
    fn init() {
        Lazy::force(&ELIXIR_REG);
    }
    init
};
