//! A “prelude” module re-exports the most-common types and functions
//! so that `use tstd::prelude::*;` brings in the basics.

pub use crate::io::{print, println};
pub use crate::math::{abs, pow};
pub use crate::collections::{Vec, HashMap};

// Add additional re-exports here as you build out tstd

pub mod io;
pub mod math;
pub mod collections;