// File: shared/src/utils.rs
// -----------------------------------------------------------------------------

//! Utility functions for safe error handling without unwrap() calls.

use std::fmt::Display;

/// Safe result helper that never panics.
pub fn safe_unwrap_or<T, E: Display>(result: Result<T, E>, default: T, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Error in {}: {}", context, e);
            default
        }
    }
}

/// Safe option helper that never panics.
pub fn safe_unwrap_option_or<T>(option: Option<T>, default: T, context: &str) -> T {
    match option {
        Some(value) => value,
        None => {
            eprintln!("None value encountered in: {}", context);
            default
        }
    }
}

/// Safe vector access that never panics.
pub fn safe_get<'a, T>(vec: &'a [T], index: usize, context: &str) -> Option<&'a T> {
    match vec.get(index) {
        Some(item) => Some(item),
        None => {
            eprintln!("Index {} out of bounds in {}, length: {}", index, context, vec.len());
            None
        }
    }
}

/// Safe string extraction that never panics.
pub fn safe_to_string<T: Display>(value: &T) -> String {
    format!("{}", value)
}

/// Safe process spawning that never panics.
pub fn safe_spawn_process(command: &str, args: &[&str]) -> Result<std::process::Child, String> {
    std::process::Command::new(command)
        .args(args)
        .spawn()
        .map_err(|e| format!("Failed to spawn process {}: {}", command, e))
}

/// Safe file reading that never panics.
pub fn safe_read_file(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file {}: {}", path, e))
}

/// Safe JSON parsing that never panics.
pub fn safe_parse_json<T>(json_str: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned
{
    serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse JSON: {}", e))
}

/// Safe thread joining that never panics.
pub fn safe_join_thread<T>(handle: std::thread::JoinHandle<T>, context: &str) -> Option<T> {
    match handle.join() {
        Ok(result) => Some(result),
        Err(e) => {
            eprintln!("Thread join failed in {}: {:?}", context, e);
            None
        }
    }
}
