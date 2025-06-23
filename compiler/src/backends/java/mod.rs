// File: compiler/src/backends/java/mod.rs
//! Java codegen backend for T-Lang.
//! Reads our IR debug-text and emits a standalone Java class
//! that replays the instructions on two stacks and prints values.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct JavaBackend;

impl Backend<CompiledModule> for JavaBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // 1. Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // 2. Begin Java source
        let mut code = String::new();
        code.push_str("public class TLang {\n");
        code.push_str("    public static void main(String[] args) {\n");
        code.push_str("        java.util.List<Long> intStack = new java.util.ArrayList<>();\n");
        code.push_str("        java.util.List<String> strStack = new java.util.ArrayList<>();\n\n");

        // 3. Replay each IR instruction
        for line in ir.lines() {
            let instr = line.trim();
            if instr.is_empty() || instr == "Nop" {
                continue;
            }
            if let Some(n) = instr.strip_prefix("PushInt(").and_then(|s| s.strip_suffix(")")) {
                code.push_str(&format!("        intStack.add({}L);\n", n));
            } else if let Some(raw) = instr
                .strip_prefix("PushStr(\"")
                .and_then(|s| s.strip_suffix("\")"))
            {
                let esc = raw.replace('\\', "\\\\").replace('"', "\\\"");
                code.push_str(&format!("        strStack.add(\"{}\");\n", esc));
            } else if instr == "CallPrint" {
                code.push_str(
                    "        if (!strStack.isEmpty()) {\n\
                     \n            String s = strStack.remove(strStack.size()-1);\n\
                     \n            System.out.print(s);\n\
                     \n        } else if (!intStack.isEmpty()) {\n\
                     \n            Long v = intStack.remove(intStack.size()-1);\n\
                     \n            System.out.print(v);\n\
                     \n        }\n",
                );
            } else {
                return Err(BackendError::Generic(format!("Unknown IR instr: {}", instr)));
            }
        }

        // 4. Close method and class
        code.push_str("    }\n");
        code.push_str("}\n");

        Ok(Box::new(code.into_bytes()))
    }

    fn name(&self) -> &'static str {
        "java"
    }
}

// Register this backend at startup
static JAVA_REG: Lazy<()> = Lazy::new(|| {
    register_backend(JavaBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_JAVA_REG: fn() = {
    fn init() {
        Lazy::force(&JAVA_REG);
    }
    init
};
