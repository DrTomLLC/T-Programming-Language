// File: compiler/src/backends/zig/mod.rs
//! Zig codegen backend for T-Lang.
//! Reads our IR debug-text and emits a standalone Zig program
//! that replays the instructions on two arrays and prints values.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct ZigBackend;

impl Backend<CompiledModule> for ZigBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // 1. Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // 2. Begin Zig source
        let mut code = String::new();
        code.push_str("const std = @import(\"std\");\n\n");
        code.push_str("pub fn main() !void {\n");
        code.push_str("    var stdout = std.io.getStdOut().writer();\n");
        code.push_str("    var intStack = std.ArrayList(i64).init(std.heap.page_allocator);\n");
        code.push_str("    defer intStack.deinit();\n");
        code.push_str("    var strStack = std.ArrayList([]const u8).init(std.heap.page_allocator);\n");
        code.push_str("    defer strStack.deinit();\n\n");

        // 3. Translate each IR instruction
        for line in ir.lines() {
            let instr = line.trim();
            if instr.is_empty() || instr == "Nop" {
                continue;
            }
            if let Some(n) = instr.strip_prefix("PushInt(").and_then(|s| s.strip_suffix(")")) {
                code.push_str(&format!("    try intStack.append({});\n", n));
            } else if let Some(raw) = instr
                .strip_prefix("PushStr(\"")
                .and_then(|s| s.strip_suffix("\")"))
            {
                let esc = raw.replace('\\', "\\\\").replace('"', "\\\"");
                code.push_str(&format!(
                    "    try strStack.append(&\"{}\".{});\n",
                    esc,
                    ".*"
                ));
            } else if instr == "CallPrint" {
                code.push_str(
                    "    if (strStack.len > 0) {\n\
                     \n        const s = strStack.pop();\n\
                     \n        _ = try stdout.print(\"{s}\", .{{}});\n\
                     \n    } else if (intStack.len > 0) {\n\
                     \n        const i = intStack.pop();\n\
                     \n        _ = try stdout.print(\"{d}\", .{.d = i});\n\
                     \n    }\n",
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
        "zig"
    }
}

// Register this backend at startup
static ZIG_REG: Lazy<()> = Lazy::new(|| {
    register_backend(ZigBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_ZIG_REG: fn() = {
    fn init() {
        Lazy::force(&ZIG_REG);
    }
    init
};
