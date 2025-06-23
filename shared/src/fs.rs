//! File: shared/src/fs.rs
//! Simple file‐I/O helper.

use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn read_to_string(path: &Path) -> Result<String> {
    let s = fs::read_to_string(path)?;
    Ok(s)
}
