// File: compiler/build.rs

fn main() {
    // Instruct Cargo to rerun this script if the grammar file changes:
    println!("cargo:rerun-if-changed=src/grammar.lalrpop");

    // Process all .lalrpop files under src/, catching any errors:
    match lalrpop::Configuration::new()
        .emit_rerun_directives(false) // we handle rerun ourselves above
        .process()
    {
        Ok(()) => {
            // success! nothing more to do
        }
        Err(e) => {
            eprintln!();
            eprintln!("🦀 LALRPOP build error:");
            // Print the full debug info so you can see spans, file names, etc.
            eprintln!("{:#?}", e);
            std::process::exit(1);
        }
    }
}
