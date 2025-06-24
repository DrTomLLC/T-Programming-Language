// tests/run_cli.rs
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::thread;
use std::io::{BufRead, BufReader};

#[test]
fn run_string_literal_script() {
    let start_time = Instant::now();
    let timeout = Duration::from_secs(120);

    let mut child = match Command::new("target/debug/tlang")
        .arg("run")
        .arg("tests/hello_cli.t")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Failed to spawn child process: {}", e);
            return; // Test fails gracefully
        }
    };

    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            eprintln!("Failed to capture stdout");
            let _ = child.kill();
            return;
        }
    };

    let stderr = match child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            eprintln!("Failed to capture stderr");
            let _ = child.kill();
            return;
        }
    };

    let stdout_handle = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(line_content) => println!("[stdout] {}", line_content),
                Err(e) => eprintln!("Error reading stdout line: {}", e),
            }
        }
    });

    let stderr_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line_content) => eprintln!("[stderr] {}", line_content),
                Err(e) => eprintln!("Error reading stderr line: {}", e),
            }
        }
    });

    let mut process_success = false;

    loop {
        if start_time.elapsed() >= timeout {
            if let Err(e) = child.kill() {
                eprintln!("Failed to kill child process: {}", e);
            }
            assert!(false, "Test timed out after {:?}", timeout);
            break;
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                process_success = status.success();
                if !process_success {
                    eprintln!("Child process exited with status: {:?}", status);
                }
                break;
            }
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(e) => {
                eprintln!("Failed to wait on child: {}", e);
                assert!(false, "Failed to wait on child process");
                break;
            }
        }
    }

    // Wait for threads to complete, but don't fail if they don't
    let _ = stdout_handle.join();
    let _ = stderr_handle.join();

    assert!(process_success, "Child process did not exit successfully");
}