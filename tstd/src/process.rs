//! Process utilities for T-Lang

use std::process::{Command, ExitStatus, Stdio};
use std::io::Result;

/// Exit the current process with status code
pub fn exit(code: i32) -> ! {
    std::process::exit(code);
}

/// Get current process ID
pub fn id() -> u32 {
    std::process::id()
}

/// Execute a command and return exit status
pub fn execute(program: &str, args: &[&str]) -> Result<ExitStatus> {
    Command::new(program)
        .args(args)
        .status()
}

/// Execute a command and capture output
pub fn execute_with_output(program: &str, args: &[&str]) -> Result<CommandOutput> {
    let output = Command::new(program)
        .args(args)
        .output()?;

    Ok(CommandOutput {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

/// Result of executing a command with output capture
pub struct CommandOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

/// Spawn a background process
pub fn spawn(program: &str, args: &[&str]) -> Result<std::process::Child> {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}

/// Get environment variable
pub fn env_var(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

/// Set environment variable
pub fn set_env_var(key: &str, value: &str) {
    std::env::set_var(key, value);
}

/// Get all environment variables
pub fn env_vars() -> Vec<(String, String)> {
    std::env::vars().collect()
}

/// Get current working directory
pub fn current_dir() -> Result<String> {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
}

/// Change current working directory
pub fn set_current_dir(path: &str) -> Result<()> {
    std::env::set_current_dir(path)
}