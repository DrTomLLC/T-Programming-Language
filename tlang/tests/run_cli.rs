// tlang/tests/run_cli.rs
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::thread;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

const MAX_RECURSION_DEPTH: usize = 512;

#[test]
fn run_string_literal_script() {
    let start_time = Instant::now();
    let timeout = Duration::from_secs(120);

    let exe_path = PathBuf::from(env!("CARGO_BIN_EXE_tlang"));

    let mut child = match Command::new(exe_path)
        .arg("run")
        .arg("tests/hello_cli.t")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Failed to spawn tlang executable: {}", e);
            assert!(false, "Could not start tlang process");
            return;
        }
    };

    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            eprintln!("Failed to capture stdout from child process");
            let _ = child.kill();
            assert!(false, "No stdout available");
            return;
        }
    };

    let stderr = match child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            eprintln!("Failed to capture stderr from child process");
            let _ = child.kill();
            assert!(false, "No stderr available");
            return;
        }
    };

    let stdout_handle = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(line_content) => println!("[stdout] {}", line_content),
                Err(e) => eprintln!("Error reading stdout: {}", e),
            }
        }
    });

    let stderr_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line_content) => eprintln!("[stderr] {}", line_content),
                Err(e) => eprintln!("Error reading stderr: {}", e),
            }
        }
    });

    let mut test_passed = false;

    loop {
        if start_time.elapsed() >= timeout {
            if let Err(e) = child.kill() {
                eprintln!("Failed to terminate child process: {}", e);
            }
            assert!(false, "Test timed out after {:?}", timeout);
            break;
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                test_passed = status.success();
                if !test_passed {
                    eprintln!("Child process failed with status: {:?}", status);
                }
                break;
            }
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(e) => {
                eprintln!("Error waiting for child process: {}", e);
                assert!(false, "Failed to monitor child process");
                break;
            }
        }
    }

    // Wait for output threads to finish
    if let Err(e) = stdout_handle.join() {
        eprintln!("Stdout thread join error: {:?}", e);
    }
    if let Err(e) = stderr_handle.join() {
        eprintln!("Stderr thread join error: {:?}", e);
    }

    assert!(test_passed, "Test execution was not successful");
}

// Helper for safe recursion depth checking
fn eval_expr_with_depth_check(expr: &Expression, depth: usize) -> Result<Value, RuntimeError> {
    if depth > MAX_RECURSION_DEPTH {
        return Err(RuntimeError::RecursionLimit {
            max_depth: MAX_RECURSION_DEPTH,
            current_depth: depth,
        });
    }

    match expr {
        Expression::Block(stmts) => {
            eval_block_with_depth_check(stmts, depth + 1)
        }
        Expression::Call { callee, args } => {
            eval_call_with_depth_check(callee, args, depth + 1)
        }
        // ... other expression types with depth + 1
        _ => Ok(Value::Unit), // Placeholder
    }
}

fn eval_block_with_depth_check(stmts: &[Statement], depth: usize) -> Result<Value, RuntimeError> {
    if depth > MAX_RECURSION_DEPTH {
        return Err(RuntimeError::RecursionLimit {
            max_depth: MAX_RECURSION_DEPTH,
            current_depth: depth,
        });
    }

    // Process statements safely...
    Ok(Value::Unit)
}

fn eval_call_with_depth_check(callee: &Expression, args: &[Expression], depth: usize) -> Result<Value, RuntimeError> {
    if depth > MAX_RECURSION_DEPTH {
        return Err(RuntimeError::RecursionLimit {
            max_depth: MAX_RECURSION_DEPTH,
            current_depth: depth,
        });
    }

    // Process function call safely...
    Ok(Value::Unit)
}

// Runtime error types for safe error handling
#[derive(Debug)]
enum RuntimeError {
    RecursionLimit { max_depth: usize, current_depth: usize },
    TypeError(String),
    UndefinedVariable(String),
    InvalidOperation(String),
}

#[derive(Debug)]
enum Value {
    Unit,
    Integer(i64),
    String(String),
    Boolean(bool),
}

#[derive(Debug)]
enum Expression {
    Block(Vec<Statement>),
    Call { callee: Box<Expression>, args: Vec<Expression> },
    Literal(Value),
}

#[derive(Debug)]
enum Statement {
    Expression(Expression),
    Let { name: String, value: Expression },
}
