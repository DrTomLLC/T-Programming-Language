//! scaffold/src/main.rs - Complete T-Lang compilation pipeline
//!
//! Goal: T-Lang source → parsed AST → type checked → Rust code → executable

mod ast;
mod parser;
mod typechecker;
mod codegen;
mod compile;

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <input.tlang>", args[0]);
        eprintln!("Example: {} test_program.tlang", args[0]);
        process::exit(1);
    }

    let filename = &args[1];

    // Step 1: Read the source file
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{filename}': {e}");
            process::exit(1);
        }
    };

    println!("🔍 Parsing T-Lang source...");

    // Step 2: Parse the source into AST
    let mut parser = parser::Parser::new(&source);
    let program = match parser.parse() {
        Ok(program) => program,
        Err(e) => {
            eprintln!("❌ Parse error: {e}");
            process::exit(1);
        }
    };

    println!("✅ Parsing successful!");
    println!("📋 AST: {:#?}", program);

    println!("\n🔍 Type checking...");

    // Step 3: Type check the AST
    let typechecker = typechecker::TypeChecker::new();
    if let Err(e) = typechecker.check_program(&program) {
        eprintln!("❌ Type error: {e}");
        process::exit(1);
    }

    println!("✅ Type checking passed!");

    println!("\n🔍 Generating Rust code...");

    // Step 4: Generate Rust code
    let mut codegen = codegen::CodeGenerator::new();
    let rust_code = match codegen.generate_program(&program) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("❌ Code generation error: {e}");
            process::exit(1);
        }
    };

    println!("✅ Code generation successful!");
    println!("📄 Generated Rust code:");
    println!("{}", rust_code);

    println!("\n🔍 Compiling to executable...");

    // Step 5: Compile Rust code to executable
    let compiler = compile::Compiler::new();

    // Extract base name for output
    let base_name = filename
        .split('.')
        .next()
        .unwrap_or("output");

    match compiler.compile_and_run(&rust_code, base_name) {
        Ok((exe_path, output, exit_code)) => {
            println!("✅ Compilation successful!");
            println!("📦 Executable: {}", exe_path);
            println!("🚀 Execution result:");
            if output.is_empty() {
                println!("   (No output - program completed successfully)");
            } else {
                println!("   Output: {}", output.trim());
            }

            println!("   Exit code: {} (returned from main)", exit_code);
        }
        Err(e) => {
            eprintln!("❌ Compilation error: {e}");
            process::exit(1);
        }
    }

    println!("\n🎉 T-Lang compilation pipeline completed successfully!");
    println!("✨ {} → AST → Type Check → Rust Code → Executable", filename);

    // Cleanup temp files
    let _ = compiler.cleanup();
}