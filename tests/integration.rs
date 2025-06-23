// File: tests/integration.rs

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn help() {
    Command::cargo_bin("app")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("T-Lang CLI"));
}

#[test]
fn compile_hello() -> Result<(), Box<dyn std::error::Error>> {
    let out = assert_cmd::assert::Assert::success(
        Command::cargo_bin("app")?
            .arg("examples/hello.t")
            .arg("--out-dir").arg("target/test-out")
    );

    // Check that at least the 'rust.bin' was produced and contains our IR
    let rust_bin = fs::read_to_string("target/test-out/rust.bin")?;
    assert!(rust_bin.contains("PushStr(\"Hello, T-Lang!\\n\")"));
    assert!(rust_bin.contains("CallPrint"));

    Ok(())
}
