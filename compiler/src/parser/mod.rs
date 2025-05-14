// compiler/src/parser/mod.rs
mod error;
pub mod expr;
//  mod stmt;
#[cfg(test)]
mod tests;

// Re‑export the main parser and its helper
pub use expr::Parser;
pub use expr::is_identifier;