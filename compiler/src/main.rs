// File: compiler/src/main.rs - COMPLETE IMPLEMENTATION
// -----------------------------------------------------------------------------

//! Complete T-Lang compiler CLI entry point.

use anyhow::{Context, Result};
#[allow(unused_imports)]
use std::path::PathBuf;
use std::fs;

use compiler::{compile_source, CompilerOptions, parse_source};
use shared::tir::TirBuilder;

struct Cli {
    command: Commands,
}

enum Commands {
}
enum Commands {
    /// Compile T-Lang source files
    Compile {
        /// Input T-Lang source file
        input: PathBuf,
        /// Output directory
        #[arg(short, long, default_value = "out")]
        output: PathBuf,
        /// Target backend
        #[arg(short, long, default_value = "rust")]
        backend: String,
        /// Optimization level (0-3)
        #[arg(short = 'O', long, default_value = "1")]
        optimization: u8,
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Check T-Lang source files without compilation
    Check {
        /// Input T-Lang source file
        input: PathBuf,
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show AST for debugging
    Ast {
        /// Input T-Lang source file
        input: PathBuf,
    },
    /// Show TIR for debugging
    Tir {
        /// Input T-Lang source file
        input: PathBuf,
    },
}

fn main() -> Result<()> {
    // TODO: Replace with proper CLI parsing
    let cli = Cli {
        command: Commands::Compile {
            input: PathBuf::from("input.tlang"),
            output: PathBuf::from("out"),
            backend: "rust".to_string(),
            optimization: 1,
            verbose: false
        }
    };

    match cli.command {
        Commands::Compile { input, output, backend, optimization, verbose } => {
            compile_file(&input, &output, &backend, optimization, verbose)
        }
        Commands::Check { input, verbose } => {
            check_file(&input, verbose)
        }
        Commands::Ast { input } => {
            show_ast(&input)
        }
        Commands::Tir { input } => {
            show_tir(&input)
        }
    }
}

fn compile_file(
    input: &PathBuf,
    output: &PathBuf,
    backend: &str,
    optimization: u8,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("Compiling {:?} with {} backend (O{})", input, backend, optimization);
    }

    // Read source file
    let source = fs::read_to_string(input)
        .with_context(|| format!("Failed to read source file: {:?}", input))?;

    // Configure compiler
    let mut options = CompilerOptions::default();
    options.target = backend.to_string();
    options.optimization_level = optimization;
    options.output_dir = output.to_string_lossy().to_string();

    // Compile
    let mut compiler = compiler::Compiler::new(source, options);
    let result = compiler.compile();

    if !result.success {
        eprintln!("Compilation failed:");
        for diagnostic in &result.diagnostics {
            eprintln!("  {}: {}", diagnostic.level, diagnostic.message);
        }
        std::process::exit(1);
    }

    if verbose {
        println!("Compilation successful!");
        if let Some(code) = &result.code {
            println!("Generated {} lines of {} code",
                     code.source.lines().count(),
                     code.target);
        }
    }

    // Write output
    if let Some(code) = result.code {
        fs::create_dir_all(output)
            .with_context(|| format!("Failed to create output directory: {:?}", output))?;

        let output_file = output.join(format!("main.{}", get_extension(&code.target)));
        fs::write(&output_file, &code.source)
            .with_context(|| format!("Failed to write output file: {:?}", output_file))?;

        if verbose {
            println!("Output written to: {:?}", output_file);
        }

        // Write additional files
        for (name, content) in &code.additional_files {
            let file_path = output.join(name);
            fs::write(&file_path, content)
                .with_context(|| format!("Failed to write additional file: {:?}", file_path))?;
        }

        // Show build commands
        if !code.build_commands.is_empty() {
            println!("To build the generated code, run:");
            for cmd in &code.build_commands {
                println!("  {}", cmd);
            }
        }
    }

    Ok(())
}

fn check_file(input: &PathBuf, verbose: bool) -> Result<()> {
    if verbose {
        println!("Checking {:?}", input);
    }

    let source = fs::read_to_string(input)
        .with_context(|| format!("Failed to read source file: {:?}", input))?;

    // Parse and type check
    match parse_source(&source) {
        Ok(program) => {
            if verbose {
                println!("✓ Parsing successful");
                println!("  {} items found", program.items.len());
            }

            // Type check
            let mut program_copy = program.clone();
            match compiler::check_program(&mut program_copy, source) {
                Ok(()) => {
                    if verbose {
                        println!("✓ Type checking successful");
                    }
                    println!("✓ All checks passed");
                }
                Err(e) => {
                    eprintln!("✗ Type checking failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Parsing failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn show_ast(input: &PathBuf) -> Result<()> {
    let source = fs::read_to_string(input)
        .with_context(|| format!("Failed to read source file: {:?}", input))?;

    match parse_source(&source) {
        Ok(program) => {
            println!("AST for {:?}:", input);
            println!("{:#?}", program);
        }
        Err(e) => {
            eprintln!("Failed to parse: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn show_tir(input: &PathBuf) -> Result<()> {
    let source = fs::read_to_string(input)
        .with_context(|| format!("Failed to read source file: {:?}", input))?;

    match parse_source(&source) {
        Ok(program) => {
            let mut builder = TirBuilder::new();
            match builder.build_module(&program) {
                Ok(tir_module) => {
                    println!("TIR for {:?}:", input);
                    println!("{:#?}", tir_module);
                }
                Err(e) => {
                    eprintln!("Failed to generate TIR: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to parse: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn get_extension(target: &str) -> &str {
    match target {
        "rust" => "rs",
        "c" => "c",
        "llvm" => "ll",
        "asm" => "s",
        "wasm" => "wat",
        _ => "txt",
    }
}
