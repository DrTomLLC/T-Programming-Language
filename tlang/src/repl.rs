// --- src/repl.rs ---
use std::io::{self, Write};

/// Starts the REPL loop using only std::io.
pub fn start_repl() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        // Print prompt
        write!(stdout, "tlang> ")?;
        stdout.flush()?;

        // Read one line of user input
        let mut line = String::new();
        let bytes_read = stdin.read_line(&mut line)?;
        if bytes_read == 0 {
            // EOF (Ctrl-D)
            println!();
            break;
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            break;
        }

        // Dispatch to your evaluator/compiler
        match crate::evaluate(input) {
            Ok(output) => println!("{}", output),
            Err(err)    => eprintln!("Error: {}", err),
        }
    }

    Ok(())
}
