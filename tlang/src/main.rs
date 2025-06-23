// tlang/src/main.rs 

use std::io::{self, Write};
use std::process;

/// Reads a line of input from stdin, displaying a prompt.
/// Returns `None` on EOF or error.
fn read_line(prompt: &str) -> Option<String> {
    // Print prompt
    print!("{}", prompt);
    // Flush to ensure prompt is shown immediately
    if io::stdout().flush().is_err() {
        return None;
    }

    let mut buffer = String::new();
    match io::stdin().read_line(&mut buffer) {
        Ok(0) => None, // EOF reached
        Ok(_) => Some(buffer.trim_end().to_owned()),
        Err(_) => None,
    }
}

fn process_input(line: &str) -> Result<String, String> {
    // TODO: Plug in your evaluation or compilation logic here.
    // For now, just echo the input.
    Ok(line.to_string())
}

fn main() {
    println!("Welcome to T-Lang REPL (std::io edition)");

    while let Some(line) = read_line("» ") {
        if line.trim().is_empty() {
            continue;
        }

        match process_input(&line) {
            Ok(output) => println!("{}", output),
            Err(err) => eprintln!("Error: {}", err),
        }
    }

    println!("Goodbye!");
    process::exit(0);
}
