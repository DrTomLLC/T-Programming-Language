//! Basic I/O functions for T-Lang.
//! You can replace the underlying implementation once the runtime is ready.

/// Print without a trailing newline.
pub fn print(s: &str) {
    // TODO: Hook into T-Lang’s runtime printing.
    // For now, compile into Rust’s stdout for development.
    print!("{}", s);
}

/// Print with a trailing newline.
pub fn println(s: &str) {
    print(&format!("{}\n", s));
}

/// Read a line of input from stdin (blocking).
pub fn read_line() -> String {
    use std::io::{self, Write};
    let mut buffer = String::new();
    // Flush stdout so prompt appears immediately.
    io::stdout().flush().ok();
    io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read line from stdin");
    buffer.trim_end().to_string()
}
