// File: compiler/src/lib.rs

//! Core T-Lang compiler: parse → sema → IR → bytecode, plus backend registration.

/// Our in-crate AST for the compiler pipeline.
use anyhow::Result as MietteResult;
pub mod ast {
    /// A parsed program is just a list of statements.
    #[derive(Debug, Clone)]
    pub struct Program {
        pub stmts: Vec<Statement>,
    }

    /// Top-level statements.
    #[derive(Debug, Clone)]
    pub enum Statement {
        Print(Expression),
        // TODO: other statements…
    }

    /// Expressions we support.
    #[derive(Debug, Clone)]
    pub enum Expression {
        StringLiteral(String),
        // TODO: other expressions…
    }
}

#[allow(clippy::all)]
#[rustfmt::skip]
/// A very simple, inline "parser" stub until LALRPOP goes in.
mod grammar {
    use crate::ast::{Expression, Program, Statement};

    pub struct ProgramParser;

    impl ProgramParser {
        pub fn new() -> Self { ProgramParser }

        /// Parses exactly one `print("…");` statement or fails.
        pub fn parse(&self, input: &str) -> Result<Program, String> {
            let input = input.trim();
            // match `print("...");`
            if let Some(inner) = input
                .strip_prefix("print(\"")
                .and_then(|s| s.strip_suffix("\");"))
            {
                let expr = Expression::StringLiteral(inner.to_string());
                let stmt = Statement::Print(expr);
                Ok(Program { stmts: vec![stmt] })
            } else {
                Err(format!("Failed to parse `{}`", input))
            }
        }
    }
}

pub mod sema;
pub mod ir;
mod backends;

// Re-export all backends:
pub use backends::asm::AsmBackend;
pub use backends::c::CBackend;
pub use backends::clojure::ClojureBackend;
pub use backends::cobol::CobolBackend;
pub use backends::css::CssBackend;
pub use backends::elixir::ElixirBackend;
pub use backends::erlang::ErlangBackend;
pub use backends::go::GoBackend;
pub use backends::haskell::HaskellBackend;
pub use backends::html::HtmlBackend;
pub use backends::java::JavaBackend;
pub use backends::javascript::JavascriptBackend;
pub use backends::kotlin::KotlinBackend;
pub use backends::llvm_backend::LlvmBackend;
pub use backends::lua::LuaBackend;
pub use backends::nim::NimBackend;
pub use backends::ocaml::OcamlBackend;
pub use backends::powershell::PowershellBackend;
pub use backends::r::RBackend;
pub use backends::ruby::RubyBackend;
pub use backends::rust::RustBackend;
pub use backends::scheme::SchemeBackend;
pub use backends::shell::ShellBackend;
pub use backends::swift::SwiftBackend;
pub use backends::typescript::TypescriptBackend;
pub use backends::v::VBackend;
pub use backends::zig::ZigBackend;
pub use backends::wasm::WasmBackend;
pub use backends::python::PythonBackend;

use crate::grammar::ProgramParser;
use plugin_api::CompiledModule;
use errors::{parse, TlError};

/// Compile T-Lang source into a `CompiledModule`.
pub fn compile_source(code: &str) -> Result<CompiledModule, TlError> {
    // 1. Parse
    let ast = ProgramParser::new()
        .parse(code)
        .map_err(|e| parse::unexpected_token("<input>", code, (0, 0).into(), "token", &e))?;

    // 2. Semantic checks
    sema::check_program(&ast)?;

    // 3. Lower to IR
    let module_ir = ir::lower_program(&ast);

    // 4. Emit debug-text bytecode
    let mut bytecode = Vec::new();
    for instr in module_ir {
        bytecode.extend_from_slice(format!("{:?}\n", instr).as_bytes());
    }

    Ok(CompiledModule::new(bytecode, vec![]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_print() {
        let src = r#"print(\"hello\");"#;
        let m = compile_source(src).expect("compile failed");
        let out = String::from_utf8(m.bytecode).unwrap();
        assert!(out.contains("PushStr(\"hello\"") );
        assert!(out.contains("CallPrint"));
    }
}
