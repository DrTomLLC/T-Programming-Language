// File: compiler/src/backends/cobol/mod.rs
//! COBOL codegen backend for T-Lang.
//! Reads our IR debug-text and emits a standalone COBOL program
//! that replays the instructions on two simple stacks and prints values.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct CobolBackend;

impl Backend<CompiledModule> for CobolBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // Begin COBOL program
        let mut code = String::new();
        code.push_str(r#"IDENTIFICATION DIVISION.
PROGRAM-ID. TLANG.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 INT-TOP    PIC 9(4) VALUE ZERO.
01 INT-STACK.
   05 INT-ELEM OCCURS 100 TIMES PIC 9(18).
01 STR-TOP    PIC 9(4) VALUE ZERO.
01 STR-STACK.
   05 STR-ELEM OCCURS 100 TIMES PIC X(256).
PROCEDURE DIVISION.
BEGIN.
"#);

        // Translate each IR instruction
        for line in ir.lines() {
            let instr = line.trim();
            if instr.is_empty() || instr == "Nop" {
                continue;
            }
            if let Some(n) = instr.strip_prefix("PushInt(").and_then(|s| s.strip_suffix(")")) {
                // increment INT-TOP and store
                code.push_str(&format!(
                    "    ADD 1 TO INT-TOP\n    MOVE {} TO INT-ELEM (INT-TOP)\n",
                    n
                ));
            } else if let Some(raw) = instr
                .strip_prefix("PushStr(\"")
                .and_then(|s| s.strip_suffix("\")"))
            {
                let esc = raw.replace('\'', "''");
                code.push_str(
                    "    ADD 1 TO STR-TOP\n"
                );
                code.push_str(&format!(
                    "    MOVE '{0}' TO STR-ELEM (STR-TOP)\n",
                    esc
                ));
            } else if instr == "CallPrint" {
                code.push_str(
                    r#"    IF STR-TOP > 0
        DISPLAY STR-ELEM (STR-TOP) NO ADVANCING
        SUBTRACT 1 FROM STR-TOP
    ELSE
        IF INT-TOP > 0
            DISPLAY INT-ELEM (INT-TOP) NO ADVANCING
            SUBTRACT 1 FROM INT-TOP
        END-IF
    END-IF
"#,
                );
            } else {
                return Err(BackendError::Generic(format!("Unknown IR instr: {}", instr)));
            }
        }

        // End program
        code.push_str("    STOP RUN.\n");

        Ok(Box::new(code.into_bytes()))
    }

    fn name(&self) -> &'static str {
        "cobol"
    }
}

// Register this backend at startup
static COBOL_REG: Lazy<()> = Lazy::new(|| {
    register_backend(CobolBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_COBOL_REG: fn() = {
    fn init() {
        Lazy::force(&COBOL_REG);
    }
    init
};
