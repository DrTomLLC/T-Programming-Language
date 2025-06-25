//! Basic I/O functions for T-Lang.

use std::io::{self, Write};

/// Print without a trailing newline.
pub fn print(s: &str) {
    // Hook into T-Lang's runtime printing.
    // For now, use Rust's stdout for development.
    print!("{}", s);
}

/// Print with a trailing newline.
pub fn println(s: &str) {
    print(&format!("{}\n", s));
}

/// Read a line of input from stdin (blocking).
pub fn read_line() -> String {
    let mut buffer = String::new();
    // Flush stdout so prompt appears immediately.
    io::stdout().flush().ok();
    io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read line from stdin");
    buffer.trim_end().to_string()
}

/// Read all text from stdin until EOF
pub fn read_all() -> String {
    use std::io::Read;
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("Failed to read from stdin");
    buffer
}

/// Print an error message to stderr
pub fn eprint(s: &str) {
    eprint!("{}", s);
}

/// Print an error message to stderr with newline
pub fn eprintln(s: &str) {
    eprintln!("{}", s);
}