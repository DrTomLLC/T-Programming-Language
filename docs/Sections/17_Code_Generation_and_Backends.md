# Section 17: Code Generation & Backends

## 17.1 Overview
Code generation transforms the TIR into executable artifacts (object files, libraries, binaries). The design goals are:
- **Modularity**: support multiple backends (LLVM, Cranelift, native compilers) via a stable plugin API
- **Configurability**: per‑target options (optimizations, relocation, code model)
- **Performance**: zero or minimal runtime overhead, incremental compilation, caching
- **Portability**: cross‑compilation for diverse OSes, architectures, and embedded targets

---

## 17.2 Backend Plugin API
- **`Backend` trait**:
  ```rust
  pub trait Backend {
      /// Unique name, e.g. "llvm", "cranelift", "rustc"
      fn name(&self) -> &str;

      /// Lower one TIR `Module` into a `CompiledArtifact`
      fn compile(&self, module: &tir::Module, config: &Config)
          -> Result<CompiledArtifact, CompileError>;
  }
