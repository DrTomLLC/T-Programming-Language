// compiler/src/runtime/mod.rs

pub mod eval;
pub mod env;
pub mod value;

/// Execute a sequence of runtime instructions (IR) represented as a vector of Values.
/// This is the primary runtime entry point after code generation.
pub use value::execute;
pub use eval::Evaluator;
