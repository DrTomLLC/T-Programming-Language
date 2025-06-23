// examples/plugin/src/lib.rs

// Bring in the plugin API types:
use compiler::plugin_api::{PluginInfo, PluginFn};
use shared::ast::AST;
use errors::TlError;

/// Called once to register your plugin.
#[no_mangle]
pub extern "C" fn register_plugin() -> PluginInfo {
    PluginInfo {
        name: "example-plugin".to_string(),
        version: "1.0.0".to_string(),
    }
}

/// Entrypoint called by the compiler: receives the full AST, returns a (possibly) transformed AST.
#[no_mangle]
pub extern "C" fn transform(input: AST) -> Result<AST, TlError> {
    // No‑op: return the AST unchanged
    Ok(input)
}

/// Entrypoint called by the compiler: receives the full AST, returns a (possibly) transformed AST.