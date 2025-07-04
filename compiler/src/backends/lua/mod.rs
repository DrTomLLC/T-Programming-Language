// File: compiler/src/backends/lua/mod.rs
//! Lua codegen backend for T-Lang.
//! Reads our IR debug-text and emits a standalone Lua script
//! that replays the instructions on two simple stacks and prints values.

use plugin_api::{register_backend, Backend, BackendError, CompiledModule};
use std::{any::Any, format, str, sync::OnceLock};

#[derive(Debug)]
pub struct LuaBackend;

impl Backend<CompiledModule> for LuaBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // 1. Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // 2. Begin Lua script
        let mut code = String::new();
        code.push_str("-- Generated by T-Lang Lua backend\n");
        code.push_str("local intStack = {}\n");
        code.push_str("local strStack = {}\n\n");

        // 3. Translate each IR instruction
        for line in ir.lines() {
            let instr = line.trim();
            if instr.is_empty() || instr == "Nop" { continue; }
            if let Some(n) = instr.strip_prefix("PushInt(").and_then(|s| s.strip_suffix(")")) {
                code.push_str(&format!("table.insert(intStack, {})\n", n));
            } else if let Some(raw) = instr
                .strip_prefix("PushStr(\"")
                .and_then(|s| s.strip_suffix("\")"))
            {
                let esc = raw.replace('\\', "\\\\").replace('"', "\\\"");
                code.push_str(&format!("table.insert(strStack, \"{}\")\n", esc));
            } else if instr == "CallPrint" {
                code.push_str(
                    "if #strStack > 0 then\n\
                     \tio.write(table.remove(strStack))\n\
                     elseif #intStack > 0 then\n\
                     \tio.write(tostring(table.remove(intStack)))\n\
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
        "lua"
    }
}

// Register this backend at startup
static LUA_REG: OnceLock<()> = OnceLock::new();

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_LUA_REG: fn() = {
    fn init() {
        LUA_REG.get_or_init(|| register_backend(LuaBackend));
    }
    init
};
