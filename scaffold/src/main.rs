//! scaffold/src/main.rs - Enhanced T-Lang CLI with proper commands
//!
//! Commands:
//! - tlang compile <file>     : Compile T-Lang source to executable
//! - tlang run <file>         : Compile and run T-Lang source
//! - tlang check <file>       : Parse and type-check only
//! - tlang ast <file>         : Show parsed AST
//! - tlang --help             : Show help
//! - tlang --version          : Show version

mod ast;
mod parser;
mod typechecker;
mod codegen;
mod compile;

use std::env;
use std::fs;
use std::process;
use std::path::Path;

const VERSION: &str = "0.1.0-scaffold";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help(&args[0]);
        process::exit(1);
    }

    match args[1].as_str() {
        "compile" => {
            if args.len() != 3 {
                eprintln!("Usage: {} compile <input.t>", args[0]);
                process::exit(1);
            }
            command_compile(&args[2]);
        }
        "run" => {
            if args.len() != 3 {
                eprintln!("Usage: {} run <input.t>", args[0]);
                process::exit(1);
            }
            command_run(&args[2]);
        }
        "check" => {
            if args.len() != 3 {
                eprintln!("Usage: {} check <input.t>", args[0]);
                process::exit(1);
            }
            command_check(&args[2]);
        }
        "ast" => {
            if args.len() != 3 {
                eprintln!("Usage: {} ast <input.t>", args[0]);
                process::exit(1);
            }
            command_ast(&args[2]);
        }
        "--help" | "-h" | "help" => {
            print_help(&args[0]);
        }
        "--version" | "-v" | "version" => {
            println!("T-Lang Compiler {}", VERSION);
        }
        _ => {
            eprintln!("❌ Unknown command: {}", args[1]);
            eprintln!("Run '{} --help' for usage information.", args[0]);
            process::exit(1);
        }
    }
}

fn print_help(program_name: &str) {
    println!("T-Lang Compiler {} - Scaffold Phase", VERSION);
    println!();
    println!("USAGE:");
    println!("    {} <COMMAND> [OPTIONS] <FILE>", program_name);
    println!();
    println!("COMMANDS:");
    println!("    compile <file>    Compile T-Lang source to executable");
    println!("    run <file>        Compile and run T-Lang source");
    println!("    check <file>      Parse and type-check only");
    println!("    ast <file>        Show parsed AST");
    println!("    help              Show this help message");
    println!("    version           Show version information");
    println!();
    println!("EXAMPLES:");
    println!("    {} compile hello.t", program_name);
    println!("    {} run hello.t", program_name);
    println!("    {} check hello.t", program_name);
    println!();
    println!("For more information, visit: https://github.com/yourusername/t-lang");
}

/// Compile T-Lang source to executable
fn command_compile(filename: &str) {
    println!("🚀 T-Lang Compiler {} - Compiling {}", VERSION, filename);

    let (source, program) = parse_and_check(filename);

    println!("\n🔍 Generating code...");
    let rust_code = generate_code(&program);

    println!("🔧 Compiling to executable...");
    let base_name = get_base_name(filename);
    let compiler = compile::Compiler::new();

    match compiler.compile_rust_code(&rust_code, &base_name) {
        Ok(exe_path) => {
            println!("✅ Compilation successful!");
            println!("📦 Executable: {}", exe_path);
        }
        Err(e) => {
            eprintln!("❌ Compilation failed: {}", e);
            process::exit(1);
        }
    }
}

/// Compile and run T-Lang source
fn command_run(filename: &str) {
    println!("🚀 T-Lang Compiler {} - Running {}", VERSION, filename);

    let (source, program) = parse_and_check(filename);

    println!("\n🔍 Generating code...");
    let rust_code = generate_code(&program);

    println!("🔧 Compiling and running...");
    let base_name = get_base_name(filename);
    let compiler = compile::Compiler::new();

    match compiler.compile_and_run(&rust_code, &base_name) {
        Ok((exe_path, output, exit_code)) => {
            println!("✅ Compilation successful!");
            println!("🚀 Execution result:");
            if !output.is_empty() {
                println!("   Output: {}", output.trim());
            }
            println!("   Exit code: {}", exit_code);
        }
        Err(e) => {
            eprintln!("❌ Execution failed: {}", e);
            process::exit(1);
        }
    }
}

/// Check T-Lang source (parse and type-check only)
fn command_check(filename: &str) {
    println!("🔍 T-Lang Compiler {} - Checking {}", VERSION, filename);

    let (_source, _program) = parse_and_check(filename);

    println!("✅ Check passed! No errors found.");
}

/// Show parsed AST
fn command_ast(filename: &str) {
    println!("🔍 T-Lang Compiler {} - AST for {}", VERSION, filename);

    let source = read_source_file(filename);
    let program = parse_source(&source, filename);

    println!("\n📋 Abstract Syntax Tree:");
    println!("{:#?}", program);
}

/// Helper: Read and validate source file
fn read_source_file(filename: &str) -> String {
    if !Path::new(filename).exists() {
        eprintln!("❌ File not found: {}", filename);
        process::exit(1);
    }

    match fs::read_to_string(filename) {
        Ok(content) => {
            if content.trim().is_empty() {
                eprintln!("❌ File is empty: {}", filename);
                process::exit(1);
            }
            content
        }
        Err(e) => {
            eprintln!("❌ Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    }
}

/// Helper: Parse source into AST
fn parse_source(source: &str, filename: &str) -> ast::Program {
    let mut parser = parser::Parser::new(source);
    match parser.parse() {
        Ok(program) => program,
        Err(e) => {
            eprintln!("❌ Parse error in '{}': {}", filename, e);
            process::exit(1);
        }
    }
}

/// Helper: Type check AST
fn type_check_program(program: &ast::Program, filename: &str) {
    let typechecker = typechecker::TypeChecker::new();
    if let Err(e) = typechecker.check_program(program) {
        eprintln!("❌ Type error in '{}': {}", filename, e);
        process::exit(1);
    }
}

/// Helper: Generate code from AST
fn generate_code(program: &ast::Program) -> String {
    let mut codegen = codegen::CodeGenerator::new();
    match codegen.generate_program(program) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("❌ Code generation error: {}", e);
            process::exit(1);
        }
    }
}

/// Helper: Parse and type-check (common pattern)
fn parse_and_check(filename: &str) -> (String, ast::Program) {
    let source = read_source_file(filename);

    println!("🔍 Parsing...");
    let program = parse_source(&source, filename);

    println!("✅ Parsing successful!");

    println!("🔍 Type checking...");
    type_check_program(&program, filename);

    println!("✅ Type checking passed!");

    (source, program)
}

/// Helper: Extract base name from filename
fn get_base_name(filename: &str) -> String {
    Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output")
        .to_string()
}