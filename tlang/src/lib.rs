// tlang/src/lib.rs

//! T-Lang library: exposes runner and REPL functionality.

pub mod runner;
pub mod repl;

pub use runner::run_file;
pub use repl::start_repl;

/// This is the entry point for your evaluator.
/// Adjust the signature and body to call into your compiler/runtime.
pub fn evaluate(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    // TODO: hook up your real compiler/interpreter here.
    Ok(format!("(evaluated '{}')", input))
}