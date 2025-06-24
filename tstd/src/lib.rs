// File: tstd/src/lib.rs - COMPLETE REWRITE
// -----------------------------------------------------------------------------

//! Complete T-Lang standard library implementation.

pub mod io;
pub mod math;
pub mod string;
pub mod collections;
pub mod time;
pub mod fs;
pub mod process;

// Re-export commonly used items
pub use io::{print, println, read_line};
pub use math::{abs, sqrt, sin, cos, tan, ln, exp, pow};

/// The prelude module contains the most commonly used items
/// that are automatically imported into every T-Lang program.
pub mod prelude {
    pub use crate::io::{print, println};
    pub use crate::math::{abs, sqrt};
    pub use super::{Result, Option, Some, None, Ok, Err};
}

/// Result type for operations that can fail
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Option type for values that may be absent
pub use std::option::Option::{self, Some, None};

/// Result variants
pub use std::result::Result::{Ok, Err};