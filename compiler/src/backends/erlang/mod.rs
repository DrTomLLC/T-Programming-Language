// File: compiler/src/backends/erlang/mod.rs
//! Erlang codegen backend: reads our IR debug‐text and emits
//! a standalone Erlang script that replays the instructions on
//! a simple list‐based stack and prints values.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct ErlangBackend;

impl Backend<CompiledModule> for ErlangBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // 1. Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // 2. Begin Erlang script
        let mut code = String::new();
        code.push_str("-module(tlang).\n");
        code.push_str("-export([main/0]).\n\n");
        code.push_str("main() ->\n");
        code.push_str("    S0 = [],\n");

        // 3. Replay each IR instruction, generating S1, S2, ...
        let mut idx = 0;
        for line in ir.lines() {
            let instr = line.trim();
            if instr.is_empty() || instr == "Nop" {
                continue;
            }
            let next = idx + 1;
            match instr {
                _ if instr.starts_with("PushInt(") && instr.ends_with(")") => {
                    let n = &instr["PushInt(".len()..instr.len() - 1];
                    code.push_str(&format!("    S{n} = [ {n} | S{idx}],\n", n = n, idx = idx));
                }
                _ if instr.starts_with("PushStr(\"") && instr.ends_with("\")") => {
                    let raw = &instr["PushStr(\"".len()..instr.len() - 2];
                    let esc = raw.replace('\\', "\\\\").replace('"', "\\\"");
                    code.push_str(&format!(
                        "    S{next} = [ \"{esc}\" | S{idx}],\n",
                        next = next,
                        idx = idx,
                        esc = esc
                    ));
                    idx = next;
                    continue; // skip increment below since handled
                }
                "CallPrint" => {
                    // Pop head and print with ~p
                    code.push_str(&format!(
                        "    [H|T] = S{idx}, io:format(\"~p\", [H]),\n    S{next} = T,\n",
                        idx = idx,
                        next = next
                    ));
                }
                other => {
                    return Err(BackendError::Generic(format!("Unknown IR instr: {}", other)));
                }
            }
            idx = next;
        }

        // 4. Finish function
        code.push_str("    ok.\n");

        Ok(Box::new(code.into_bytes()))
    }

    fn name(&self) -> &'static str {
        "erlang"
    }
}

// Register this backend at startup
static ERLANG_REG: Lazy<()> = Lazy::new(|| {
    register_backend(ErlangBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_ERLANG_REG: fn() = {
    fn init() {
        Lazy::force(&ERLANG_REG);
    }
    init
};
