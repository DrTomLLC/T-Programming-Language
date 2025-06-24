//! Integration tests for T-Lang compiler
//!
//! These tests verify that the complete compilation pipeline works correctly
//! from source code to compiled output.

use compiler::{Compiler, init_builtin_backends, CompileConfigBuilder, OptimizationLevel};
use shared::compile::quick_compile;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_complete_compilation_pipeline() {
    init_builtin_backends();

    let source = r#"
        fn main() {
            let x = 42;
            let y = x + 8;
            return y;
        }
    "#;

    let result = quick_compile(source);
    assert!(result.is_ok(), "Compilation should succeed");

    let module = result.unwrap();
    assert_eq!(module.name(), "main");
    assert_eq!(module.tir().functions.len(), 1);
    assert!(!module.bytecode().is_empty());
}

#[test]
fn test_lexer_parser_integration() {
    let source = r#"
        fn factorial(n: i32) -> i32 {
            if n <= 1 {
                return 1;
            } else {
                return n * factorial(n - 1);
            }
        }

        fn main() {
            let result = factorial(5);
            return result;
        }
    "#;

    let result = quick_compile(source);
    assert!(result.is_ok(), "Complex function compilation should succeed");

    let module = result.unwrap();
    assert_eq!(module.tir().functions.len(), 2); // factorial + main
}

#[test]
fn test_backend_compilation() {
    init_builtin_backends();

    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }

        fn main() {
            let result = add(10, 20);
            return result;
        }
    "#;

    let compiler = Compiler::with_global_registry();
    let backends = compiler.list_backends();

    assert!(!backends.is_empty(), "Should have at least one backend");

    for backend_name in backends {
        let result = compiler.compile_with_backend(source, "test", &backend_name);
        assert!(result.is_ok(), "Backend {} should compile successfully", backend_name);

        let artifact = result.unwrap();
        assert_eq!(artifact.target, backend_name);
        assert!(!artifact.data.is_empty());
    }
}

#[test]
fn test_different_optimization_levels() {
    init_builtin_backends();

    let source = r#"
        fn compute() -> i32 {
            let mut sum = 0;
            let mut i = 0;
            while i < 100 {
                sum = sum + i;
                i = i + 1;
            }
            return sum;
        }

        fn main() {
            return compute();
        }
    "#;

    let opt_levels = [
        OptimizationLevel::None,
        OptimizationLevel::Less,
        OptimizationLevel::Default,
        OptimizationLevel::Aggressive,
    ];

    for opt_level in opt_levels {
        let config = CompileConfigBuilder::new()
            .optimization(opt_level)
            .build();

        let compiler = Compiler::with_global_registry().with_config(config);
        let result = compiler.compile_with_backend(source, "test", "stub");

        assert!(result.is_ok(), "Optimization level {:?} should work", opt_level);
    }
}

#[test]
fn test_error_handling() {
    init_builtin_backends();

    // Test lexer error
    let invalid_source = r#"
        fn main() {
            let x = "unterminated string
        }
    "#;

    let result = quick_compile(invalid_source);
    assert!(result.is_err(), "Invalid syntax should produce error");

    // Test parser error
    let parser_error_source = r#"
        fn main() {
            let x = ;
        }
    "#;

    let result = quick_compile(parser_error_source);
    assert!(result.is_err(), "Parser error should be caught");

    // Test semantic error (undefined variable)
    let semantic_error_source = r#"
        fn main() {
            return undefined_variable;
        }
    "#;

    let result = quick_compile(semantic_error_source);
    assert!(result.is_err(), "Undefined variable should cause error");
}

#[test]
fn test_file_compilation() {
    init_builtin_backends();

    let temp_dir = TempDir::new().unwrap();
    let source_file = temp_dir.path().join("test.t");

    let source_content = r#"
        fn greet(name: string) {
            print("Hello, " + name + "!");
        }

        fn main() {
            greet("World");
            return 0;
        }
    "#;

    std::fs::write(&source_file, source_content).unwrap();

    let compiler = Compiler::with_global_registry();
    let result = compiler.compile_file_to_tir(&source_file);

    assert!(result.is_ok(), "File compilation should succeed");

    let module = result.unwrap();
    assert_eq!(module.name, "test");
    assert_eq!(module.functions.len(), 2); // greet + main
}

#[test]
fn test_multiple_files_compilation() {
    init_builtin_backends();

    let temp_dir = TempDir::new().unwrap();

    // Create multiple source files
    let files = [
        ("math.t", r#"
            fn add(a: i32, b: i32) -> i32 {
                return a + b;
            }

            fn multiply(a: i32, b: i32) -> i32 {
                return a * b;
            }
        "#),
        ("main.t", r#"
            fn main() -> i32 {
                let sum = add(5, 3);
                let product = multiply(sum, 2);
                return product;
            }
        "#),
    ];

    let compiler = Compiler::with_global_registry();

    for (filename, content) in &files {
        let file_path = temp_dir.path().join(filename);
        std::fs::write(&file_path, content).unwrap();

        let result = compiler.compile_file_to_tir(&file_path);
        assert!(result.is_ok(), "File {} should compile", filename);
    }
}

#[test]
fn test_complex_control_flow() {
    init_builtin_backends();

    let source = r#"
        fn fibonacci(n: i32) -> i32 {
            if n <= 1 {
                return n;
            }

            let mut a = 0;
            let mut b = 1;
            let mut i = 2;

            while i <= n {
                let temp = a + b;
                a = b;
                b = temp;
                i = i + 1;
            }

            return b;
        }

        fn main() -> i32 {
            let result = fibonacci(10);
            return result;
        }
    "#;

    let result = quick_compile(source);
    assert!(result.is_ok(), "Complex control flow should compile");

    let module = result.unwrap();
    assert_eq!(module.tir().functions.len(), 2);

    // Verify that the fibonacci function has the expected structure
    let fibonacci_func = module.tir().functions.values()
        .find(|f| f.name == "fibonacci")
        .expect("Should find fibonacci function");

    assert!(!fibonacci_func.blocks.is_empty(), "Function should have basic blocks");
}

#[test]
fn test_struct_and_enum_compilation() {
    init_builtin_backends();

    let source = r#"
        struct Point {
            x: i32,
            y: i32,
        }

        enum Color {
            Red,
            Green,
            Blue,
        }

        fn main() -> i32 {
            const origin = Point { x: 0, y: 0 };
            const favorite_color = Color::Blue;
            return 0;
        }
    "#;

    let result = quick_compile(source);
    assert!(result.is_ok(), "Struct and enum compilation should succeed");

    let module = result.unwrap();
    assert!(!module.tir().types.is_empty(), "Should have custom types defined");
}

#[test]
fn test_standard_library_integration() {
    // This test would verify that T-Lang programs can use the standard library
    // For now, we'll just test that the standard library compiles

    let _version = tstd::VERSION;
    let _math_pi = tstd::math::PI;

    // Test that we can use standard library types
    let _result: tstd::core::Result<i32, &str> = Ok(42);
    let _option: tstd::core::Option<i32> = Some(42);
}

#[test]
fn test_compilation_with_debug_info() {
    init_builtin_backends();

    let source = r#"
        fn debug_function() -> i32 {
            let x = 42;
            let y = x * 2;
            return y;
        }

        fn main() -> i32 {
            return debug_function();
        }
    "#;

    let config = CompileConfigBuilder::new()
        .debug_info(true)
        .optimization(OptimizationLevel::None)
        .build();

    let compiler = Compiler::with_global_registry().with_config(config);
    let result = compiler.compile_with_backend(source, "test", "stub");

    assert!(result.is_ok(), "Debug compilation should succeed");
}

#[test]
fn test_error_reporting_with_spans() {
    init_builtin_backends();

    // Test that errors include proper source location information
    let source_with_error = r#"
        fn main() {
            let x = undefined_function();
            return x;
        }
    "#;

    let result = quick_compile(source_with_error);
    assert!(result.is_err(), "Should produce error for undefined function");

    // The error should contain useful information about where the error occurred
    let error = result.unwrap_err();
    let error_string = format!("{}", error);
    assert!(error_string.contains("undefined"), "Error should mention undefined identifier");
}

#[test]
fn test_performance_benchmarks() {
    init_builtin_backends();

    // Test compilation performance with a moderately complex program
    let complex_source = r#"
        fn bubble_sort(arr: [i32; 10]) -> [i32; 10] {
            let mut result = arr;
            let mut i = 0;

            while i < 10 {
                let mut j = 0;
                while j < 9 - i {
                    if result[j] > result[j + 1] {
                        let temp = result[j];
                        result[j] = result[j + 1];
                        result[j + 1] = temp;
                    }
                    j = j + 1;
                }
                i = i + 1;
            }

            return result;
        }

        fn main() -> i32 {
            let numbers = [64, 34, 25, 12, 22, 11, 90, 88, 76, 50];
            let sorted = bubble_sort(numbers);
            return sorted[0];
        }
    "#;

    let start = std::time::Instant::now();
    let result = quick_compile(complex_source);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Complex program should compile");
    assert!(duration.as_millis() < 1000, "Compilation should be reasonably fast");

    println!("Compilation took: {:?}", duration);
}

#[cfg(test)]
mod helper_functions {
    use super::*;

    pub fn create_temp_source_file(content: &str, filename: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(filename);
        std::fs::write(&file_path, content).unwrap();
        (temp_dir, file_path)
    }

    pub fn assert_compilation_succeeds(source: &str) {
        init_builtin_backends();
        let result = quick_compile(source);
        assert!(result.is_ok(), "Compilation should succeed for: {}", source);
    }

    pub fn assert_compilation_fails(source: &str) {
        init_builtin_backends();
        let result = quick_compile(source);
        assert!(result.is_err(), "Compilation should fail for: {}", source);
    }
}

// Use the helper functions in additional tests
#[test]
fn test_simple_programs_compile() {
    use helper_functions::assert_compilation_succeeds;

    assert_compilation_succeeds("fn main() { return 0; }");
    assert_compilation_succeeds("fn main() -> i32 { return 42; }");
    assert_compilation_succeeds(r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
        fn main() {
            return add(1, 2);
        }
    "#);
}

#[test]
fn test_invalid_programs_fail() {
    use helper_functions::assert_compilation_fails;

    assert_compilation_fails("fn main() { return; }"); // Missing function body
    assert_compilation_fails("fn main() { let x; }");   // Uninitialized variable
    assert_compilation_fails("fn main() { x = 1; }");   // Undefined variable
}