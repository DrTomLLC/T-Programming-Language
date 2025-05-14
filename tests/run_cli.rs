// tests/run_cli.rs

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::thread;
use std::io::{BufRead, BufReader};

#[test]
fn run_string_literal_script() {
    let start_time = Instant::now();
    let timeout = Duration::from_secs(120); // ⏱️ Increased timeout to 120 seconds

    let mut child = Command::new("target/debug/tlang")
        .arg("run")
        .arg("tests/hello_cli.t")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    let stdout = child.stdout.take().expect("no stdout");
    let stderr = child.stderr.take().expect("no stderr");

    let stdout_handle = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            println!("[stdout] {}", line.unwrap());
        }
    });

    let stderr_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            eprintln!("[stderr] {}", line.unwrap());
        }
    });

    loop {
        if start_time.elapsed() >= timeout {
            child.kill().expect("Failed to kill child process");
            panic!("Test timed out after {:?}", timeout);
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                assert!(status.success(), "Child process exited with {:?}", status);
                break;
            }
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(e) => panic!("Failed to wait on child: {}", e),
        }
    }

    let _ = stdout_handle.join();
    let _ = stderr_handle.join();
}
