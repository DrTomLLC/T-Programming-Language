// File: compiler/src/backends/python/mod.rs
//! Python codegen backend for T-Lang.
//! Reads our IR debug-text and emits a standalone Python script
//! that replays the instructions on two lists and prints values.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct PythonBackend;

impl Backend<CompiledModule> for PythonBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // 1. Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // 2. Begin Python script
        let mut code = String::new();
        code.push_str("#!/usr/bin/env python3\n");
        code.push_str("import sys\n\n");
        code.push_str("def main():\n");
        code.push_str("    int_stack = []\n");
        code.push_str("    str_stack = []\n\n");

        // 3. Translate each IR instruction
        for line in ir.lines() {
            let inst = line.trim();
            if inst.is_empty() || inst == "Nop" {
                continue;
            }
            if let Some(n) = inst.strip_prefix("PushInt(").and_then(|s| s.strip_suffix(")")) {
                code.push_str(&format!("    int_stack.append({})\n", n));
            } else if let Some(raw) = inst
                .strip_prefix("PushStr(\"")
                .and_then(|s| s.strip_suffix("\")"))
            {
                let esc = raw.replace('\\', "\\\\").replace('"', "\\\"");
                code.push_str(&format!("    str_stack.append(\"{}\")\n", esc));
            } else if inst == "CallPrint" {
                code.push_str(
                    "    if str_stack:\n\
                     \n        sys.stdout.write(str_stack.pop())\n\
                     \n    elif int_stack:\n\
                     \n        sys.stdout.write(str(int_stack.pop()))\n",
                );
            } else {
                return Err(BackendError::Generic(format!("Unknown IR instr: {}", inst)));
            }
        }

        // 4. Invoke main
        code.push_str("\nif __name__ == \"__main__\":\n");
        code.push_str("    main()\n");

        Ok(Box::new(code.into_bytes()))
    }

    fn name(&self) -> &'static str {
        "python"
    }
}

// Register this backend at startup
static PYTHON_REG: Lazy<()> = Lazy::new(|| {
    register_backend(PythonBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_PYTHON_REG: fn() = {
    fn init() {
        Lazy::force(&PYTHON_REG);
    }
    init
};
