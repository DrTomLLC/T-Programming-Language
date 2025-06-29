//! scaffold/tests/integration_test.rs
//!
//! Integration tests that verify the complete T-Lang compilation pipeline:
//! T-Lang source → Parse → Type Check → Code Gen → Rust Compile → Executable → Run
//!
//! This ensures our scaffold compiler actually works end-to-end.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::env;

/// Test case definition
#[derive(Debug)]
struct TestCase {
    name: &'static str,
    source_file: &'static str,
    source_content: &'static str,
    expected_exit_code: i32,
    description: &'static str,
}

/// All test cases to run
const TEST_CASES: &[TestCase] = &[
    TestCase {
        name: "test1_hello",
        source_file: "test1_hello.t",
        source_content: "fn main() -> i32 {\n    return 42;\n}",
        expected_exit_code: 42,
        description: "Basic function returning 42",
    },
    TestCase {
        name: "test2_zero",
        source_file: "test2_zero.t",
        source_content: "fn main() -> i32 {\n    return 0;\n}",
        expected_exit_code: 0,
        description: "Function returning 0 (success)",
    },
    TestCase {
        name: "test3_negative",
        source_file: "test3_negative.t",
        source_content: "fn main() -> i32 {\n    return -1;\n}",
        expected_exit_code: 255, // -1 wraps to 255 on most systems
        description: "Function returning negative number",
    },
    TestCase {
        name: "test4_large",
        source_file: "test4_large.t",
        source_content: "fn main() -> i32 {\n    return 1000;\n}",
        expected_exit_code: 232, // 1000 % 256 = 232 (8-bit exit codes)
        description: "Function returning large number",
    },
    TestCase {
        name: "test5_max",
        source_file: "test5_max.t",
        source_content: "fn main() -> i32 {\n    return 2147483647;\n}",
        expected_exit_code: 255, // Large number wraps in exit code
        description: "Function returning max i32",
    },
];

fn main() {
    println!("🧪 T-Lang Scaffold Integration Tests");
    println!("=====================================");

    // Setup test environment
    setup_test_environment();

    let mut passed = 0;
    let mut failed = 0;

    // Run each test case
    for test_case in TEST_CASES {
        println!("\n🔍 Running test: {} - {}", test_case.name, test_case.description);

        match run_test_case(test_case) {
            Ok(()) => {
                println!("✅ PASSED: {}", test_case.name);
                passed += 1;
            }
            Err(e) => {
                println!("❌ FAILED: {} - {}", test_case.name, e);
                failed += 1;
            }
        }
    }

    // Print summary
    println!("\n📊 Test Results:");
    println!("===============");
    println!("✅ Passed: {}", passed);
    println!("❌ Failed: {}", failed);
    println!("📈 Total:  {}", passed + failed);

    if failed > 0 {
        println!("\n❌ Some tests failed!");
        std::process::exit(1);
    } else {
        println!("\n🎉 All tests passed! Scaffold compiler is working correctly.");
    }
}

/// Setup test environment (create test files)
fn setup_test_environment() {
    println!("📁 Setting up test environment...");

    // Create tests directory if it doesn't exist
    let tests_dir = Path::new("tests");
    if !tests_dir.exists() {
        fs::create_dir_all(tests_dir).expect("Failed to create tests directory");
    }

    // Create each test file
    for test_case in TEST_CASES {
        let file_path = tests_dir.join(test_case.source_file);
        fs::write(&file_path, test_case.source_content)
            .expect(&format!("Failed to create test file: {}", test_case.source_file));
    }

    println!("✅ Test environment ready!");
}

/// Run a single test case through the complete pipeline
fn run_test_case(test_case: &TestCase) -> Result<(), String> {
    let test_file = format!("tests/{}", test_case.source_file);

    // Step 1: Check that our scaffold compiler exists
    let scaffold_binary = find_scaffold_binary()?;

    // Step 2: Test parsing only (should succeed)
    println!("   🔍 Testing parse/check...");
    test_parse_check(&scaffold_binary, &test_file)?;

    // Step 3: Test compilation (should produce executable)
    println!("   🔧 Testing compilation...");
    test_compilation(&scaffold_binary, &test_file)?;

    // Step 4: Test execution (verify exit code)
    println!("   🚀 Testing execution...");
    test_execution(&scaffold_binary, &test_file, test_case.expected_exit_code)?;

    Ok(())
}

/// Find the scaffold binary to test
fn find_scaffold_binary() -> Result<String, String> {
    // Look for the compiled binary
    let possible_paths = [
        "target/debug/scaffold",
        "target/release/scaffold",
        "./scaffold",
        "../target/debug/scaffold",
        "../target/release/scaffold",
    ];

    for path in &possible_paths {
        if Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }

    Err("Could not find scaffold binary. Please run 'cargo build' first.".to_string())
}

/// Test parsing and type checking
fn test_parse_check(scaffold_binary: &str, test_file: &str) -> Result<(), String> {
    let output = Command::new(scaffold_binary)
        .args(&["check", test_file])
        .output()
        .map_err(|e| format!("Failed to run check command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Parse/check failed: {}", stderr));
    }

    Ok(())
}

/// Test compilation to executable
fn test_compilation(scaffold_binary: &str, test_file: &str) -> Result<(), String> {
    let output = Command::new(scaffold_binary)
        .args(&["compile", test_file])
        .output()
        .map_err(|e| format!("Failed to run compile command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Compilation failed: {}", stderr));
    }

    // Check that executable was created
    let exe_name = test_file.replace(".t", "").replace("tests/", "");
    let exe_path = Path::new(&exe_name);

    if !exe_path.exists() {
        return Err(format!("Expected executable '{}' was not created", exe_name));
    }

    Ok(())
}

/// Test execution with expected exit code
fn test_execution(scaffold_binary: &str, test_file: &str, expected_exit_code: i32) -> Result<(), String> {
    let output = Command::new(scaffold_binary)
        .args(&["run", test_file])
        .output()
        .map_err(|e| format!("Failed to run execution command: {}", e))?;

    // Note: Our scaffold prints the exit code, so we parse it from stdout
    let stdout = String::from_utf8_lossy(&output.stdout);

    if let Some(line) = stdout.lines().find(|line| line.contains("Exit code:")) {
        if let Some(code_str) = line.split("Exit code:").nth(1) {
            let actual_exit_code: i32 = code_str.trim()
                .parse()
                .map_err(|_| format!("Could not parse exit code from: {}", line))?;

            if actual_exit_code != expected_exit_code {
                return Err(format!(
                    "Expected exit code {}, but got {}",
                    expected_exit_code, actual_exit_code
                ));
            }
        } else {
            return Err("Could not find exit code in output".to_string());
        }
    } else {
        return Err("No exit code found in output".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_definitions_are_valid() {
        // Verify all test cases are well-formed
        for test_case in TEST_CASES {
            assert!(!test_case.name.is_empty());
            assert!(!test_case.source_file.is_empty());
            assert!(!test_case.source_content.is_empty());
            assert!(test_case.source_file.ends_with(".t"));
            assert!(test_case.source_content.contains("fn main()"));
        }
    }

    #[test]
    fn test_environment_setup() {
        // Test that we can create test files
        let temp_dir = std::env::temp_dir().join("t_lang_test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let test_file = temp_dir.join("test.t");
        std::fs::write(&test_file, "fn main() -> i32 { return 0; }").unwrap();

        assert!(test_file.exists());

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }
}