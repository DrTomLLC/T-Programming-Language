// tlang/src/main.rs

use std::{
    fs,
    io::{self, Write},
    process,
};
use clap::{Parser, Subcommand};
use compiler::parser::Parser as TParser;
use compiler::runtime::env::Environment;
use compiler::runtime::eval::Evaluator;
use shared::ast::Stmt;

/// T‑Lang CLI (BOM‑aware, debug)
#[derive(Parser, Debug)]
#[command(name = "tlang", version, about = "T‑Lang Interpreter")]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a T‑Lang script file and exit
    Run {
        /// Path to the `.t` script file
        script: String,
    },
}

fn main() {
    eprintln!("DEBUG> Program start");
    let cli = Cli::parse();
    eprintln!("DEBUG> Parsed CLI args: {:?}", cli);

    match cli.cmd {
        Some(Commands::Run { script }) => {
            eprintln!("DEBUG> Entered Run branch, script={}", script);
            run_file(&script);
            process::exit(0);
        }
        None => {
            eprintln!("DEBUG> No subcommand, entering REPL");
            repl();
        }
    }
}

fn run_file(path: &str) {
    // 1) Read raw bytes
    eprintln!("DEBUG> Reading file {}", path);
    let bytes = fs::read(path).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", path, e);
        process::exit(1);
    });
    eprintln!("DEBUG> Read {} bytes", bytes.len());

    // 2) Decode BOM‐aware
    let source = decode_source(bytes);
    eprintln!("DEBUG> Decoded source ({} chars)", source.chars().count());

    // 3) Lex & parse all statements
    eprintln!("DEBUG> Lexing & parsing");
    let mut parser = TParser::from_source(&source).unwrap_or_else(|e| {
        eprintln!("Lex error: {}", e);
        process::exit(1);
    });
    let stmts = parser.parse().unwrap_or_else(|e| {
        eprintln!("Parse error: {}", e);
        process::exit(1);
    });
    eprintln!("DEBUG> Parsed {} statements", stmts.len());
    if let Some(stmt) = stmts.first() {
        eprintln!("DEBUG> First stmt: {}", stmt_name(stmt));
    }
    eprintln!("DEBUG> Proceeding to eval loop...");

    // 4) Evaluate
    let mut env = Environment::new();
    let mut eval = Evaluator::new(&mut env);
    for (i, stmt) in stmts.into_iter().enumerate() {
        eprintln!("DEBUG> Evaluating stmt[{}]: {:?}", i, stmt);
        match eval.eval_stmt(stmt) {
            Ok(val) => println!("{}", val),
            Err(err) => {
                eprintln!("Runtime error: {}", err);
                process::exit(1);
            }
        }
    }
    eprintln!("DEBUG> run_file complete");
}

fn decode_source(bytes: Vec<u8>) -> String {
    if bytes.starts_with(&[0xFF, 0xFE]) {
        eprintln!("DEBUG> Detected UTF-16 LE BOM");
        let mut u16_buf = Vec::with_capacity(bytes.len() / 2);
        for chunk in bytes[2..].chunks(2) {
            u16_buf.push(u16::from_le_bytes([chunk[0], chunk[1]]));
        }
        String::from_utf16(&u16_buf).unwrap_or_else(|e| {
            eprintln!("UTF-16LE decode error: {}", e);
            process::exit(1);
        })
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        eprintln!("DEBUG> Detected UTF-16 BE BOM");
        let mut u16_buf = Vec::with_capacity(bytes.len() / 2);
        for chunk in bytes[2..].chunks(2) {
            u16_buf.push(u16::from_be_bytes([chunk[0], chunk[1]]));
        }
        String::from_utf16(&u16_buf).unwrap_or_else(|e| {
            eprintln!("UTF-16BE decode error: {}", e);
            process::exit(1);
        })
    } else if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        eprintln!("DEBUG> Detected UTF-8 BOM");
        String::from_utf8(bytes[3..].to_vec()).unwrap_or_else(|e| {
            eprintln!("UTF-8 decode error: {}", e);
            process::exit(1);
        })
    } else {
        eprintln!("DEBUG> Assuming UTF-8 (no BOM)");
        String::from_utf8(bytes).unwrap_or_else(|e| {
            eprintln!("UTF-8 decode error: {}", e);
            process::exit(1);
        })
    }
}

fn repl() {
    println!("T‑Lang REPL — Type 'exit;' to quit.");
    let mut env = Environment::new();
    let mut eval = Evaluator::new(&mut env);

    loop {
        eprintln!("DEBUG> REPL prompt");
        print!("> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            eprintln!("DEBUG> Input error");
            continue;
        }
        eprintln!("DEBUG> Input: {:?}", line);

        let trimmed = line.trim();
        eprintln!("DEBUG> Trimmed: {:?}", trimmed);
        if trimmed == "exit;" {
            eprintln!("DEBUG> Exiting REPL");
            break;
        }

        match TParser::from_source(&line) {
            Ok(mut p) => match p.parse_statement() {
                Ok(stmt) => match eval.eval_stmt(stmt) {
                    Ok(v)   => println!("= {}", v),
                    Err(e)  => eprintln!("Runtime error: {}", e),
                },
                Err(e) => eprintln!("Parse error: {}", e),
            },
            Err(e) => eprintln!("Lex error: {}", e),
        }
    }
}

fn stmt_name(s: &Stmt) -> &'static str {
    match s {
        Stmt::Expr(_) => "Expr",
        Stmt::Let(_, _) => "Let",
        Stmt::Assign(_, _) => "Assign",
        Stmt::If { .. } => "If",
        Stmt::While { .. } => "While",
        Stmt::Block(_) => "Block",
        Stmt::Function(_, _, _) => "Function",
    }
}
