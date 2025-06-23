/* ===================== example_backend/src/lib.rs ======================== */
use plugin_api::{register_backend, Backend, BackendError, CompiledModule};

/// A toy backend that simply echoes the module byte‑code back out.
pub struct EchoBackend;

impl Backend<CompiledModule> for EchoBackend {
    type ModuleIr = Box<dyn std::any::Any + Send + Sync>; // final artefact as trait object

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        Ok(Box::new(module.bytecode.into_boxed_slice()))
    }

    fn name(&self) -> &'static str {
        "echo"
    }
}

// Register on library load.
#[ctor::ctor]
fn init() {
    register_backend(EchoBackend);
}