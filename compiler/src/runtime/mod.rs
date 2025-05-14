// compiler/src/runtime/mod.rs
pub mod value;
pub mod env;
pub mod eval;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod parser_eval_tests;
pub mod error;

// pub use error::RuntimeError;