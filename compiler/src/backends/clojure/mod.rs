// File: compiler/src/backends/clojure/mod.rs
//! Clojure codegen backend: reads our IR debug‐text and emits
//! a standalone Clojure script that replays the instructions
//! on a simple stack.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct ClojureBackend;

impl Backend<CompiledModule> for ClojureBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // 1. Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // 2. Begin Clojure script
        let mut code = String::new();
        code.push_str(r#"(ns tlang.core)
(defn -main []
  (let [stack (atom [])]"#);
        code.push('\n');

        // 3. Translate each IR instruction
        for line in ir.lines() {
            let instr = line.trim();
            if instr.is_empty() || instr == "Nop" {
                continue;
            }
            if let Some(n) = instr.strip_prefix("PushInt(").and_then(|s| s.strip_suffix(")")) {
                code.push_str(&format!("    (swap! stack conj {})\n", n));
            } else if let Some(raw) = instr
                .strip_prefix("PushStr(\"")
                .and_then(|s| s.strip_suffix("\")"))
            {
                let esc = raw.replace('\\', "\\\\").replace('"', "\\\"");
                code.push_str(&format!("    (swap! stack conj \"{}\")\n", esc));
            } else if instr == "CallPrint" {
                code.push_str(
                    "    (let [s (peek @stack)]\n\
                     \t(swap! stack pop)\n\
                     \t(print s))\n",
                );
            } else {
                return Err(BackendError::Generic(format!(
                    "Unknown IR instr: {}",
                    instr
                )));
            }
        }

        // 4. End and invoke
        code.push_str("  )\n)\n\n(-main)\n");

        Ok(Box::new(code.into_bytes()))
    }

    fn name(&self) -> &'static str {
        "clojure"
    }
}

// Register this backend at startup
static CLOJURE_REG: Lazy<()> = Lazy::new(|| {
    register_backend(ClojureBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_CLOJURE_REG: fn() = {
    fn init() {
        Lazy::force(&CLOJURE_REG);
    }
    init
};
