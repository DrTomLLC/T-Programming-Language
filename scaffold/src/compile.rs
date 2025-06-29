//! scaffold/src/compile.rs - Calls rustc to build executable
//!
//! This module handles compilation of generated Rust code to executable.

use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub enum CompileError {
    WriteError(std::io::Error),
    CompileError(String),
    RustcNotFound,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::WriteError(e) => write!(f, "Failed to write Rust file: {e}"),
            CompileError::CompileError(msg) => write!(f, "Compilation failed: {msg}"),
            CompileError::RustcNotFound => write!(f, "rustc not found - please install Rust"),
        }
    }
}

impl std::error::Error for CompileError {}

pub struct Compiler {
    temp_dir: String,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            temp_dir: "target/scaffold_temp".to_string(),
        }
    }

    /// Compile Rust source code to executable
    pub fn compile_rust_code(&self, rust_code: &str, output_name: &str) -> Result<String, CompileError> {
        // Create temp directory if it doesn't exist
        std::fs::create_dir_all(&self.temp_dir)
            .map_err(CompileError::WriteError)?;

        // Write Rust code to temporary file
        let rust_file = format!("{}/main.rs", self.temp_dir);
        fs::write(&rust_file, rust_code)
            .map_err(CompileError::WriteError)?;

        // Determine output executable name (with .exe on Windows)
        let exe_name = if cfg!(windows) {
            format!("{}.exe", output_name)
        } else {
            output_name.to_string()
        };

        let output_path = format!("{}/{}", self.temp_dir, exe_name);

        // Call rustc to compile
        let output = Command::new("rustc")
            .arg(&rust_file)
            .arg("-o")
            .arg(&output_path)
            .arg("--edition")
            .arg("2021")
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    // Compilation successful
                    Ok(output_path)
                } else {
                    // Compilation failed
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    Err(CompileError::CompileError(stderr.to_string()))
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Err(CompileError::RustcNotFound)
                } else {
                    Err(CompileError::CompileError(format!("Failed to run rustc: {}", e)))
                }
            }
        }
    }

    /// Run the compiled executable and return its output
    pub fn run_executable(&self, exe_path: &str) -> Result<String, CompileError> {
        let output = Command::new(exe_path)
            .output()
            .map_err(|e| CompileError::CompileError(format!("Failed to run executable: {}", e)))?;

        // For T-Lang programs, we expect them to exit with the return value
        // So we don't treat non-zero exit codes as errors - just capture the output
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Full compilation pipeline: T-Lang source -> executable -> run
    pub fn compile_and_run(&self, rust_code: &str, output_name: &str) -> Result<(String, String, i32), CompileError> {
        // Compile the Rust code
        let exe_path = self.compile_rust_code(rust_code, output_name)?;

        // Run the executable and capture both output and exit code
        let output_result = Command::new(&exe_path)
            .output()
            .map_err(|e| CompileError::CompileError(format!("Failed to run executable: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output_result.stdout).to_string();
        let exit_code = output_result.status.code().unwrap_or(-1);

        Ok((exe_path, stdout, exit_code))
    }

    /// Clean up temporary files
    pub fn cleanup(&self) -> Result<(), CompileError> {
        if Path::new(&self.temp_dir).exists() {
            std::fs::remove_dir_all(&self.temp_dir)
                .map_err(CompileError::WriteError)?;
        }
        Ok(())
    }
}