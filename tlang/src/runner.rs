// File: tlang/src/runner.rs

//! File runner for T-Lang source files.
//! Reads a source file, compiles it, and prints bytecode or errors.

use std::{error::Error, fs, path::Path};
use compiler::compile_source;

/// Run T-Lang on the specified file path.
///
/// # Errors
/// Returns an error if file I/O or compilation fails.
pub fn run_file(path: &Path) -> Result<(), Box<dyn Error>> {
    let src = fs::read_to_string(path)?;
    match compile_source(&src) {
        Ok(module) => {
            let out = String::from_utf8_lossy(&module.bytecode);
            println!("{}", out);
            Ok(())
        }
        Err(e) => {
            eprintln!("Compilation error: {:#?}", e);
            Err(Box::new(e))
        }
    }
}
