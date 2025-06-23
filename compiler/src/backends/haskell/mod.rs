// File: compiler/src/backends/haskell/mod.rs
//! Haskell codegen backend for T-Lang.
//! Reads our IR debug-text and emits a standalone Haskell program
//! that replays the instructions on two simple stacks and prints values.

use plugin_api::{register_backend, Backend, CompiledModule, BackendError};
use once_cell::sync::Lazy;
use std::{any::Any, str};

#[derive(Debug)]
pub struct HaskellBackend;

impl Backend<CompiledModule> for HaskellBackend {
    type ModuleIr = Box<dyn Any + Send + Sync>;

    fn compile(&self, module: CompiledModule) -> Result<Self::ModuleIr, BackendError> {
        // 1. Decode IR text
        let ir = str::from_utf8(&module.bytecode)
            .map_err(|e| BackendError::Generic(format!("Invalid UTF-8 IR: {}", e)))?;

        // 2. Begin Haskell source
        let mut code = String::new();
        code.push_str("module Main where\n\n");
        code.push_str("import Data.List (isPrefixOf)\n\n");
        code.push_str("main :: IO ()\n");
        code.push_str("main = evaluate [] [] ir\n\n");
        code.push_str("  where\n");
        code.push_str("    ir :: [String]\n");
        code.push_str("    ir = [\n");
        for line in ir.lines() {
            let esc = line.replace('\\', "\\\\").replace('"', "\\\"");
            code.push_str(&format!("      \"{}\",\n", esc));
        }
        code.push_str("      ]\n\n");
        code.push_str("evaluate :: [Integer] -> [String] -> [String] -> IO ()\n");
        code.push_str("evaluate _ _ [] = return ()\n");
        code.push_str("evaluate ints strs (instr:rest)\n");
        code.push_str("  | \"PushInt(\" `isPrefixOf` instr =\n");
        code.push_str("      let n = read (takeWhile (/=')') (drop 8 instr)) :: Integer\n");
        code.push_str("      in evaluate (n:ints) strs rest\n\n");
        code.push_str("  | \"PushStr(\\\"\" `isPrefixOf` instr =\n");
        code.push_str("      let s = read (drop 8 instr) :: String\n");
        code.push_str("      in evaluate ints (s:strs) rest\n\n");
        code.push_str("  | instr == \"CallPrint\" = do\n");
        code.push_str("      case strs of\n");
        code.push_str("        (s:ss) -> putStr s >> evaluate ints ss rest\n");
        code.push_str("        [] -> case ints of\n");
        code.push_str("          (i:is) -> putStr (show i) >> evaluate is strs rest\n");
        code.push_str("          []     -> evaluate ints strs rest\n\n");
        code.push_str("  | otherwise = evaluate ints strs rest\n");

        Ok(Box::new(code.into_bytes()))
    }

    fn name(&self) -> &'static str {
        "haskell"
    }
}

// Register this backend at startup
static HASKELL_REG: Lazy<()> = Lazy::new(|| {
    register_backend(HaskellBackend);
});

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
static FORCE_HASKELL_REG: fn() = {
    fn init() {
        Lazy::force(&HASKELL_REG);
    }
    init
};
