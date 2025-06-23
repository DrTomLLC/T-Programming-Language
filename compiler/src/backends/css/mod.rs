// File: compiler/src/backends/css/mod.rs
//! CSS codegen backend: reads our IR debug-text and emits
//! a standalone CSS file with each instruction preserved as a comment.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct CssBackend;

impl Backend<CompiledModule> for CssBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // 1. Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // 2. Begin CSS output
        let mut css = String::new();
        css.push_str("/* T-Lang CSS IR (debug) */\n");

        // 3. Embed each instruction as a CSS comment
        for line in ir.lines() {
            let inst = line.trim();
            if inst.is_empty() || inst == "Nop" {
                continue;
            }
            css.push_str("/* ");
            css.push_str(inst);
            css.push_str(" */\n");
        }

        // 4. Add a no-op valid rule to ensure valid CSS
        css.push_str("\nbody { /* T-Lang IR embedded above */ }\n");

        Ok(Box::new(css.into_bytes()))
    }

    fn name(&self) -> &'static str {
        "css"
    }
}

// Register this backend at startup
static CSS_REG: Lazy<()> = Lazy::new(|| {
    register_backend(CssBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_CSS_REG: fn() = {
    fn init() {
        Lazy::force(&CSS_REG);
    }
    init
};
