// File: compiler/src/backends/html/mod.rs
//! HTML codegen backend for T-Lang.
//! Reads our IR debug-text and emits a standalone HTML document
//! embedding each instruction as an HTML comment and displaying them in a <pre> block.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct HtmlBackend;

impl Backend<CompiledModule> for HtmlBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // 1. Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // 2. Begin HTML document
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("  <meta charset=\"UTF-8\">\n");
        html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str("  <title>T-Lang IR</title>\n");
        html.push_str("</head>\n<body>\n");
        html.push_str("  <!-- T-Lang IR embedded as comments -->\n");

        // 3. Embed each IR instruction as an HTML comment
        for line in ir.lines() {
            let inst = line.trim();
            if inst.is_empty() || inst == "Nop" {
                continue;
            }
            html.push_str("  <!-- ");
            html.push_str(inst);
            html.push_str(" -->\n");
        }

        // 4. Display IR in a <pre> block for readability
        html.push_str("  <pre>\n");
        for line in ir.lines() {
            let inst = line.trim();
            if !inst.is_empty() {
                html.push_str("    ");
                html.push_str(inst);
                html.push('\n');
            }
        }
        html.push_str("  </pre>\n");

        html.push_str("</body>\n</html>\n");

        Ok(Box::new(html.into_bytes()))
    }

    fn name(&self) -> &'static str {
        "html"
    }
}

// Register this backend at startup
static HTML_REG: Lazy<()> = Lazy::new(|| {
    register_backend(HtmlBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_HTML_REG: fn() = {
    fn init() {
        Lazy::force(&HTML_REG);
    }
    init
};
