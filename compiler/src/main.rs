//! File: compiler/src/main.rs
//! CLI entry point for the T-Lang compiler.
//!
//! Usage:
//!     cargo run --bin compiler -- <input_file.t> [--out-dir <directory>]
//!
//! Reads `<input_file.t>`, compiles to `CompiledModule` (stub), then for each
//! registered backend (via plugin_api), calls `backend.compile(...)` and writes
//! the resulting IR blob to `<out-dir>/<backend_name>.bin`.

use anyhow::{bail, Context, Result};
use plugin_api::{CompiledModule, list_backends};
use shared::fs::read_to_string;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Configuration parsed from CLI arguments.
struct Config {
    input_path: PathBuf,
    out_dir: PathBuf,
}

impl Config {
    fn parse_args() -> Result<Self> {
        let mut args = env::args().skip(1); // skip binary name

        // Expect at least one argument: the input .t file.
        let input = args.next().context("Expected path to <input_file.t>")?;
        let mut out_dir = PathBuf::from("out"); // default “out” directory

        // Optional: allow “--out-dir <dir>”
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--out-dir" => {
                    if let Some(dir) = args.next() {
                        out_dir = PathBuf::from(dir);
                    } else {
                        bail!("--out-dir requires a directory path");
                    }
                }
                unknown => {
                    bail!("Unrecognized argument: {}", unknown);
                }
            }
        }

        Ok(Config {
            input_path: PathBuf::from(input),
            out_dir,
        })
    }
}

fn main() -> Result<()> {
    let cfg = Config::parse_args()?;

    // 1. Read source
    let source = read_to_string(&cfg.input_path)
        .with_context(|| format!("Failed to read source file: {:?}", cfg.input_path))?;

    // 2. Compile to bytecode (stub)
    let module: CompiledModule = compiler::compile_source(&source)?;
    let bc_len = module.bytecode.len();
    println!(
        "Compiled '{:?}' → {} bytes of bytecode.",
        cfg.input_path, bc_len
    );

    // 3. Ensure output directory exists
    if !cfg.out_dir.exists() {
        fs::create_dir_all(&cfg.out_dir)
            .with_context(|| format!("Failed to create output directory {:?}", cfg.out_dir))?;
    }

    // 4. Dispatch to each registered backend
    for backend in list_backends() {
        let ir = backend
            .compile(module.clone())
            .unwrap_or_else(|e| {
                panic!("Backend '{}' failed to compile: {}", backend.name(), e);
            });

        // ir is `Box<dyn Any + Send + Sync>` –– assume each backend boxed a Vec<u8>.
        let bytes: &Vec<u8> = ir
            .downcast_ref::<Vec<u8>>()
            .unwrap_or_else(|| panic!("Backend '{}' returned non-Vec<u8> IR", backend.name()));

        let out_path = cfg.out_dir.join(format!("{}.bin", backend.name()));
        let mut file = fs::File::create(&out_path)
            .with_context(|| format!("Failed to create backend output file {:?}", out_path))?;
        file.write_all(bytes)
            .with_context(|| format!("Failed to write IR to {:?}", out_path))?;

        println!(
            "{} backend produced output ({} bytes) → {:?}",
            backend.name(),
            bytes.len(),
            out_path,
        );
    }

    Ok(())
}

