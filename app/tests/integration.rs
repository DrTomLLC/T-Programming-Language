// tests/integration.rs
// Integration tests for the T-Lang CLI application.
// This file tests the CLI commands and their expected behavior.
// Requires the `assert_cmd`, `predicates` and `tempfile` crates in Cargo.toml.

use assert_cmd::Command;
use predicates::prelude::*;
use std::{fs, path::PathBuf};
use tempfile::TempDir;

#[test]
fn help_shows_name_and_usage() {
    Command::cargo_bin("app")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        // clap help always shows the binary name and a Usage line
        .stdout(predicate::str::contains("app").and(predicate::str::contains("Usage")));
}

#[test]
fn compile_hello_creates_bytecode() -> Result<(), Box<dyn std::error::Error>> {
    // create a fresh temporary output directory
    let out_dir = TempDir::new()?;
    let out_path = out_dir.path();

    // resolve the real path to examples/hello.t in the workspace root
    let hello_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()             // go from .../T-Lang/app -> .../T-Lang
        .join("examples")
        .join("hello.t");

    // run the CLI
    Command::cargo_bin("app")?
        .arg(&hello_file)
        .arg("--out-dir")
        .arg(out_path)
        .assert()
        .success();

    // grab the first file in the output directory
    let bin_entry = fs::read_dir(out_path)?
        .filter_map(Result::ok)
        .find(|e| e.path().is_file())
        .expect("Expected at least one output file in the out-dir");

    let path = bin_entry.path();
    let bytecode = fs::read_to_string(&path)?;

    // verify the bytecode listing
    assert!(
        bytecode.contains("PushStr(\"Hello, T-Lang!\\n\")"),
        "didn't see the hello string in {:?}",
        path
    );
    assert!(
        bytecode.contains("CallPrint"),
        "didn't see a CallPrint instruction in {:?}",
        path
    );

    Ok(())
}
