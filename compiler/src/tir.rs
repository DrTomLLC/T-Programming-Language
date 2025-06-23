// VERSION 1.0.0 of the plugin interface.
// 🚨 Any breaking changes here must bump the API version.

pub mod plugin_api {
    use shared::ast::Item::Module;
    use errors::TlError as CompileError;
    use serde::{Serialize, Deserialize};

    // A plugin must export one of these:
    //
    // ```rust
    // use compiler::plugin_api;
    //
    // #[no_mangle]
    // pub extern "C" fn register_plugin() -> plugin_api::PluginInfo { /* ... */ }
    // ```
    #[derive(Serialize, Deserialize)]
    pub struct PluginInfo {
        pub name: String,
        pub version: String,
    }

    /// Called by the compiler to transform a module.
    pub type PluginFn = fn(input: shared::ast::Item) -> Result<shared::ast::Item, CompileError>;
}
