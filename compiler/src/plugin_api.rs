// VERSION 1.0.0 of the plugin interface.
// 🚨 Any breaking changes here must bump this.
pub const API_VERSION: &str = "1.0.0";

pub mod plugin_api {
    use shared::ast::AST;
    use errors::TlError as CompileError;
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize)]
    pub struct PluginInfo {
        pub name: String,
        pub version: String,
    }

    /// Entrypoint: receive the full AST, return a (possibly transformed) AST.
    pub type PluginFn = fn(input: AST) -> Result<AST, CompileError>;
}
