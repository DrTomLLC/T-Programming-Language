// plugin_api/src/lib.rs
//! Plugin API for T-Lang compiler backends and transformations.
//! Provides a stable interface for extending the compiler with custom code generators.

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use errors::TlError;
use serde::{Deserialize, Serialize};

/// Compiled module representation passed between compiler phases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledModule {
    /// Bytecode instructions for the module
    pub bytecode: Vec<u8>,
    /// Module metadata
    pub metadata: ModuleMetadata,
    /// Debug information
    pub debug_info: Option<DebugInfo>,
}

/// Metadata about a compiled module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetadata {
    /// Module name
    pub name: String,
    /// Source file path
    pub source_path: Option<String>,
    /// Compilation timestamp
    pub timestamp: u64,
    /// Compiler version used
    pub compiler_version: String,
    /// Target platform
    pub target: String,
    /// Optimization level
    pub optimization_level: u8,
}

/// Debug information for a compiled module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    /// Source line mappings
    pub line_info: Vec<LineInfo>,
    /// Variable information
    pub variables: Vec<VariableInfo>,
    /// Function information
    pub functions: Vec<FunctionInfo>,
}

/// Source line mapping information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineInfo {
    /// Bytecode offset
    pub bytecode_offset: usize,
    /// Source line number
    pub line: usize,
    /// Source column number
    pub column: usize,
}

/// Variable debug information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    /// Variable name
    pub name: String,
    /// Type information
    pub type_name: String,
    /// Scope start offset
    pub scope_start: usize,
    /// Scope end offset
    pub scope_end: usize,
}

/// Function debug information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    /// Function name
    pub name: String,
    /// Start offset in bytecode
    pub start_offset: usize,
    /// End offset in bytecode
    pub end_offset: usize,
    /// Parameter information
    pub parameters: Vec<ParameterInfo>,
    /// Return type
    pub return_type: String,
}

/// Parameter debug information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub type_name: String,
}

/// Error types for backend operations.
#[derive(Debug, Clone)]
pub enum BackendError {
    /// Generic error with message
    Generic(String),
    /// Unsupported feature error
    UnsupportedFeature(String),
    /// Code generation error
    CodegenError(String),
    /// Target-specific error
    TargetError(String),
    /// Configuration error
    ConfigError(String),
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendError::Generic(msg) => write!(f, "Backend error: {}", msg),
            BackendError::UnsupportedFeature(feature) => write!(f, "Unsupported feature: {}", feature),
            BackendError::CodegenError(msg) => write!(f, "Code generation error: {}", msg),
            BackendError::TargetError(msg) => write!(f, "Target error: {}", msg),
            BackendError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for BackendError {}

impl From<BackendError> for TlError {
    fn from(error: BackendError) -> Self {
        TlError::plugin("backend", error.to_string())
    }
}

/// Main trait for backend plugins.
pub trait Backend<M = CompiledModule> {
    /// Type of intermediate representation this backend generates
    type ModuleIr: Send + Sync;

    /// Compile a module to the backend's intermediate representation
    fn compile(&self, module: M) -> Result<Self::ModuleIr, BackendError>;

    /// Get the backend's unique name
    fn name(&self) -> &'static str;

    /// Get the backend's version
    fn version(&self) -> &'static str {
        "1.0.0"
    }

    /// Get supported target platforms
    fn supported_targets(&self) -> &'static [&'static str] {
        &["default"]
    }

    /// Check if a target is supported
    fn supports_target(&self, target: &str) -> bool {
        self.supported_targets().contains(&target)
    }

    /// Get backend capabilities
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::default()
    }

    /// Initialize the backend with configuration
    fn initialize(&mut self, _config: &BackendConfig) -> Result<(), BackendError> {
        Ok(())
    }

    /// Finalize compilation (optional cleanup)
    fn finalize(&mut self) -> Result<(), BackendError> {
        Ok(())
    }
}

/// Backend capabilities and features.
#[derive(Debug, Clone, Default)]
pub struct BackendCapabilities {
    /// Supports just-in-time compilation
    pub supports_jit: bool,
    /// Supports ahead-of-time compilation
    pub supports_aot: bool,
    /// Supports debugging information
    pub supports_debug_info: bool,
    /// Supports optimization
    pub supports_optimization: bool,
    /// Supports cross-compilation
    pub supports_cross_compilation: bool,
    /// Maximum optimization level supported
    pub max_optimization_level: u8,
    /// Supported output formats
    pub output_formats: Vec<String>,
}

/// Configuration for backends.
#[derive(Debug, Clone)]
pub struct BackendConfig {
    /// Target platform
    pub target: String,
    /// Optimization level
    pub optimization_level: u8,
    /// Enable debug information
    pub debug_info: bool,
    /// Output directory
    pub output_dir: String,
    /// Additional configuration options
    pub options: HashMap<String, String>,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            target: "default".to_string(),
            optimization_level: 1,
            debug_info: false,
            output_dir: "output".to_string(),
            options: HashMap::new(),
        }
    }
}

/// Registry for managing backend plugins.
#[derive(Debug)]
pub struct BackendRegistry {
    backends: HashMap<String, Box<dyn Backend<CompiledModule, ModuleIr = Box<dyn Any + Send + Sync>> + Send + Sync>>,
}

impl BackendRegistry {
    /// Create a new backend registry.
    pub fn new() -> Self {
        Self {
            backends: HashMap::new(),
        }
    }

    /// Register a backend plugin.
    pub fn register<B>(&mut self, backend: B)
    where
        B: Backend<CompiledModule, ModuleIr = Box<dyn Any + Send + Sync>> + Send + Sync + 'static,
    {
        let name = backend.name().to_string();
        self.backends.insert(name, Box::new(backend));
    }

    /// Get a backend by name.
    pub fn get(&self, name: &str) -> Option<&dyn Backend<CompiledModule, ModuleIr = Box<dyn Any + Send + Sync>>> {
        self.backends.get(name).map(|b| b.as_ref())
    }

    /// Get a mutable reference to a backend by name.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut dyn Backend<CompiledModule, ModuleIr = Box<dyn Any + Send + Sync>>> {
        self.backends.get_mut(name).map(|b| b.as_mut())
    }

    /// List all registered backend names.
    pub fn list_backends(&self) -> Vec<&str> {
        self.backends.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a backend is registered.
    pub fn has_backend(&self, name: &str) -> bool {
        self.backends.contains_key(name)
    }

    /// Remove a backend from the registry.
    pub fn remove(&mut self, name: &str) -> bool {
        self.backends.remove(name).is_some()
    }

    /// Clear all backends from the registry.
    pub fn clear(&mut self) {
        self.backends.clear();
    }
}

impl Default for BackendRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global backend registry instance.
static BACKEND_REGISTRY: Mutex<BackendRegistry> = Mutex::new(BackendRegistry {
    backends: HashMap::new(),
});

/// Register a backend with the global registry.
pub fn register_backend<B>(backend: B)
where
    B: Backend<CompiledModule, ModuleIr = Box<dyn Any + Send + Sync>> + Send + Sync + 'static,
{
    if let Ok(mut registry) = BACKEND_REGISTRY.lock() {
        registry.register(backend);
    }
}

/// Get a list of all registered backends.
pub fn list_backends() -> Vec<String> {
    BACKEND_REGISTRY
        .lock()
        .map(|registry| registry.list_backends().into_iter().map(|s| s.to_string()).collect())
        .unwrap_or_default()
}

/// Compile a module using a specific backend.
pub fn compile_with_backend(
    backend_name: &str,
    module: CompiledModule,
) -> Result<Box<dyn Any + Send + Sync>, BackendError> {
    let registry = BACKEND_REGISTRY.lock().map_err(|_| {
        BackendError::Generic("Failed to acquire backend registry lock".to_string())
    })?;

    let backend = registry.get(backend_name).ok_or_else(|| {
        BackendError::Generic(format!("Backend '{}' not found", backend_name))
    })?;

    backend.compile(module)
}

/// Information about a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: Option<String>,
    /// Plugin author
    pub author: Option<String>,
    /// Supported compiler version
    pub compiler_version: String,
}

/// Plugin lifecycle hooks.
pub trait Plugin {
    /// Get plugin information.
    fn info(&self) -> PluginInfo;

    /// Initialize the plugin.
    fn initialize(&mut self) -> Result<(), TlError> {
        Ok(())
    }

    /// Finalize the plugin.
    fn finalize(&mut self) -> Result<(), TlError> {
        Ok(())
    }
}

/// Trait for AST transformation plugins.
pub trait AstTransform {
    /// Transform an AST.
    fn transform(&self, ast: shared::ast::Program) -> Result<shared::ast::Program, TlError>;
}

/// Trait for optimization plugins.
pub trait Optimizer {
    /// Optimize a compiled module.
    fn optimize(&self, module: CompiledModule) -> Result<CompiledModule, TlError>;
}

impl CompiledModule {
    /// Create a new compiled module.
    pub fn new(name: String, bytecode: Vec<u8>) -> Self {
        Self {
            bytecode,
            metadata: ModuleMetadata {
                name,
                source_path: None,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                compiler_version: env!("CARGO_PKG_VERSION").to_string(),
                target: "default".to_string(),
                optimization_level: 1,
            },
            debug_info: None,
        }
    }

    /// Get the module name.
    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    /// Get the bytecode.
    pub fn bytecode(&self) -> &[u8] {
        &self.bytecode
    }

    /// Get instructions from bytecode (placeholder implementation).
    pub fn instructions(&self) -> Vec<String> {
        // This is a simplified implementation for demo purposes
        self.bytecode
            .chunks(4)
            .enumerate()
            .map(|(i, chunk)| {
                if chunk.len() == 4 {
                    let opcode = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    format!("Instruction_{}: {}", i, opcode)
                } else {
                    format!("Partial_{}: {:?}", i, chunk)
                }
            })
            .collect()
    }

    /// Add debug information.
    pub fn with_debug_info(mut self, debug_info: DebugInfo) -> Self {
        self.debug_info = Some(debug_info);
        self
    }

    /// Set target platform.
    pub fn with_target(mut self, target: String) -> Self {
        self.metadata.target = target;
        self
    }

    /// Set optimization level.
    pub fn with_optimization_level(mut self, level: u8) -> Self {
        self.metadata.optimization_level = level;
        self
    }
}

impl BackendConfig {
    /// Create a new backend configuration.
    pub fn new(target: String) -> Self {
        Self {
            target,
            ..Default::default()
        }
    }

    /// Set optimization level.
    pub fn with_optimization_level(mut self, level: u8) -> Self {
        self.optimization_level = level;
        self
    }

    /// Enable debug information.
    pub fn with_debug_info(mut self, enable: bool) -> Self {
        self.debug_info = enable;
        self
    }

    /// Set output directory.
    pub fn with_output_dir(mut self, dir: String) -> Self {
        self.output_dir = dir;
        self
    }

    /// Add a configuration option.
    pub fn with_option(mut self, key: String, value: String) -> Self {
        self.options.insert(key, value);
        self
    }

    /// Get a configuration option.
    pub fn get_option(&self, key: &str) -> Option<&str> {
        self.options.get(key).map(|s| s.as_str())
    }
}

/// Helper macros for defining backends.
#[macro_export]
macro_rules! define_backend {
    ($name:ident, $backend_name:expr) => {
        pub struct $name;

        impl Backend<CompiledModule> for $name {
            type ModuleIr = Box<dyn Any + Send + Sync>;

            fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
                // Default implementation
                Ok(Box::new(module.bytecode))
            }

            fn name(&self) -> &'static str {
                $backend_name
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    define_backend!(TestBackend, "test");

    #[test]
    fn test_backend_registration() {
        let mut registry = BackendRegistry::new();
        let backend = TestBackend;

        registry.register(backend);
        assert!(registry.has_backend("test"));
        assert_eq!(registry.list_backends(), vec!["test"]);
    }

    #[test]
    fn test_compiled_module() {
        let module = CompiledModule::new("test".to_string(), vec![1, 2, 3, 4]);
        assert_eq!(module.name(), "test");
        assert_eq!(module.bytecode(), &[1, 2, 3, 4]);
    }

    #[test]
    fn test_backend_config() {
        let config = BackendConfig::new("x86_64".to_string())
            .with_optimization_level(2)
            .with_debug_info(true)
            .with_option("feature".to_string(), "enabled".to_string());

        assert_eq!(config.target, "x86_64");
        assert_eq!(config.optimization_level, 2);
        assert!(config.debug_info);
        assert_eq!(config.get_option("feature"), Some("enabled"));
    }
}