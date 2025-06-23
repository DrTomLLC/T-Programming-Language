// tlang/src/cli.rs

use clap::{Parser, Subcommand};

/// Top-level CLI definition for T-Lang.
#[derive(Parser)]
#[command(name = "tlang", version)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Execute a .tl script then exit.
    Run {
        /// Path to the `.tl` script file
        script: String,
    },
    /// Launch the interactive REPL.
    Repl,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_run_command() {
        let args = Cli::parse_from(&["tlang", "run", "file.tl"]);
        match args.cmd {
            Command::Run { script } => assert_eq!(script, "file.tl"),
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn parse_repl_command() {
        let args = Cli::parse_from(&["tlang", "repl"]);
        match args.cmd {
            Command::Repl => (),
            _ => panic!("Expected Repl command"),
        }
    }
}
