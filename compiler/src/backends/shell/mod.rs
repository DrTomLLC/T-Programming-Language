// File: compiler/src/backends/shell/mod.rs
//! Shell (Bash) codegen backend for T-Lang.
//! Reads our IR debug-text and emits a standalone Bash script
//! that replays the instructions on two arrays and prints values.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct ShellBackend;

impl Backend<CompiledModule> for ShellBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // Begin Bash script
        let mut code = String::new();
        code.push_str("#!/usr/bin/env bash\n");
        code.push_str("set -euo pipefail\n\n");
        code.push_str("int_stack=()\n");
        code.push_str("str_stack=()\n\n");

        // Replay each IR instruction
        code.push_str("for instr in \\\n");
        for line in ir.lines() {
            let inst = line.trim();
            if inst.is_empty() || inst == "Nop" {
                continue;
            }
            // escape backslashes and quotes in comment
            let esc = inst.replace('\\', "\\\\").replace('"', "\\\"");
            code.push_str(&format!("    \"{}\" \\\n", esc));
        }
        code.push_str("; do\n");
        code.push_str("  if [[ \"$instr\" == PushInt(*\") ]]; then\n");
        code.push_str("    n=${instr#PushInt(}; n=${n%)}\n");
        code.push_str("    int_stack+=(\"$n\")\n");
        code.push_str("  elif [[ \"$instr\" == PushStr(\\\"*\\\") ]]; then\n");
        code.push_str("    raw=${instr#PushStr(\\\"}\n");
        code.push_str("    raw=${raw%\\\")}\n");
        code.push_str("    # escape for bash literal\n");
        code.push_str("    esc=${raw//\"/\\\\\\\"}\n");
        code.push_str("    str_stack+=(\"$esc\")\n");
        code.push_str("  elif [[ \"$instr\" == CallPrint ]]; then\n");
        code.push_str("    if (( ${#str_stack[@]} )); then\n");
        code.push_str("      s=${str_stack[-1]}\n");
        code.push_str("      echo -n \"$s\"\n");
        code.push_str("      unset 'str_stack[-1]'\n");
        code.push_str("    elif (( ${#int_stack[@]} )); then\n");
        code.push_str("      v=${int_stack[-1]}\n");
        code.push_str("      echo -n \"$v\"\n");
        code.push_str("      unset 'int_stack[-1]'\n");
        code.push_str("    fi\n");
        code.push_str("  fi\n");
        code.push_str("done\n");

        Ok(Box::new(code.into_bytes()))
    }

    fn name(&self) -> &'static str {
        "shell"
    }
}

// Register this backend at startup
static SHELL_REG: Lazy<()> = Lazy::new(|| {
    register_backend(ShellBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_SHELL_REG: fn() = {
    fn init() {
        Lazy::force(&SHELL_REG);
    }
    init
};
