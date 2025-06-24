// File: tstd/src/io.rs - COMPLETE REWRITE
// -----------------------------------------------------------------------------

//! Complete I/O implementation for T-Lang standard library.

use std::io::{self, Write, BufRead, BufReader, Read};
use std::fs::File;
use std::path::Path;

/// Print a value without a trailing newline
pub fn print<T: std::fmt::Display>(value: T) -> Result<(), io::Error> {
    print!("{}", value);
    io::stdout().flush()
}

/// Print a value with a trailing newline
pub fn println<T: std::fmt::Display>(value: T) -> Result<(), io::Error> {
    println!("{}", value);
    Ok(())
}

/// Read a line from standard input
pub fn read_line() -> Result<String, io::Error> {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut buffer = String::new();

    io::stdout().flush()?;
    reader.read_line(&mut buffer)?;

    // Remove trailing newline
    if buffer.ends_with('\n') {
        buffer.pop();
        if buffer.ends_with('\r') {
            buffer.pop();
        }
    }

    Ok(buffer)
}

/// Read all input until EOF
pub fn read_to_string() -> Result<String, io::Error> {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut buffer = String::new();

    reader.read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Read an entire file to a string
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    std::fs::read_to_string(path)
}

/// Write a string to a file
pub fn write_file<P: AsRef<Path>>(path: P, contents: &str) -> Result<(), io::Error> {
    std::fs::write(path, contents)
}

/// Append a string to a file
pub fn append_file<P: AsRef<Path>>(path: P, contents: &str) -> Result<(), io::Error> {
    use std::fs::OpenOptions;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    file.write_all(contents.as_bytes())?;
    Ok(())
}

/// Print to standard error
pub fn eprint<T: std::fmt::Display>(value: T) -> Result<(), io::Error> {
    eprint!("{}", value);
    io::stderr().flush()
}

/// Print to standard error with newline
pub fn eprintln<T: std::fmt::Display>(value: T) -> Result<(), io::Error> {
    eprintln!("{}", value);
    Ok(())
}
