// app/src/main.rs

use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
use anyhow::{Context, Result};
use clap::Parser;
use compiler::compile_source;
use plugin_api::CompiledModule;

/// T-Lang compiler frontend
#[derive(Parser)]
#[command(name = "app")]
struct Opt {
    /// The input `.t` file to compile
    input: PathBuf,

    /// Where to put the generated `.bin` files
    #[arg(long, default_value = ".")]
    out_dir: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opt::parse();

    // Read the source
    let src = fs::read_to_string(&opt.input)
        .with_context(|| format!("reading `{}`", opt.input.display()))?;

    // Compile it to bytecode
    let module: CompiledModule = compile_source(&src)
        .with_context(|| format!("compiling `{}`", opt.input.display()))?;

    // Make sure output directory exists
    fs::create_dir_all(&opt.out_dir)
        .with_context(|| format!("creating output directory `{}`", opt.out_dir.display()))?;

    // Compute output file name: e.g. "examples/hello.t" -> "hello.bin"
    let stem = opt
        .input
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid input file name"))?;
    let out_file = opt.out_dir.join(format!("{}.bin", stem));

    // Create and write it
    let mut f = File::create(&out_file)
        .with_context(|| format!("creating output file `{}`", out_file.display()))?;
    for instr in &module.instructions() {
        writeln!(f, "{instr:?}")?;
    }

    Ok(())
}
