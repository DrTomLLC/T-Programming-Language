# Codegen & Backends Architecture

This section defines the architecture and design of T‑Lang's code generation and backend subsystem. It covers the plugin API surface, supported backends, loading mechanisms, build/link strategies, and guidelines for extending or modifying backends in the future.

---

## 1. Overview

* **Purpose**: Transform the lowered TIR (T‑Lang Intermediate Representation) into runnable binaries, object files, or libraries for various target platforms.
* **Scope**:

    * Static backends (compiled into the compiler binary).
    * Dynamic backends (loaded at runtime via `libloading`).
    * Hybrid approaches (combo of static and dynamic).
    * Cross-compilation and target triples.

---

## 2. Plugin API Surface

* **Location**: `plugin_api/src/lib.rs`
* **Key Traits & Types**:

    * `Backend` trait:

      ```rust
      pub trait Backend {
          /// Unique name
          fn name(&self) -> &str;
          /// Produce code for a `tir::Module`
          fn compile(&self, module: &tir::Module, config: &Config) -> Result<CompiledArtifact, CompileError>;
      }
      ```
    * `Config` struct: target triple, optimization level, output formats.
    * `CompiledArtifact`: bytes + format identifier.
    * `PluginRegistry`:

      ```rust
      pub struct PluginRegistry {
          backends: HashMap<String, Box<dyn Backend>>,
      }
  
      impl PluginRegistry {
          pub fn register(&mut self, backend: Box<dyn Backend>);
          pub fn get(&self, name: &str) -> Option<&dyn Backend>;
      }
      ```

---

## 3. Backend Interface & Lifecycle

1. **Initialization**:

    * Compiler core creates a `PluginRegistry` instance.
    * Static backends register themselves in `mod.rs`.
    * Dynamic backends loaded from `--backend-dir` register via a known entrypoint (`fn register(reg: &mut PluginRegistry)`).
2. **Selection**:

    * CLI flag (`--backend llvm`, `--backend rust`, etc.) selects the backend by name.
3. **Compilation**:

    * The chosen backend’s `compile` is invoked, receiving the final `tir::Module` and `Config`.
4. **Emission**:

    * Backend returns a `CompiledArtifact` containing the binary section(s) (object file, shared library, executable, WASM, etc.).

---

## 4. Loading Mechanisms

### 4.1 Static Linking

* Backends built directly into the `compiler` crate.
* Advantages:

    * Zero runtime overhead.
    * Simpler distribution.
* Drawbacks:

    * Larger compiler binary.
    * Requires recompilation to add/remove backends.

### 4.2 Dynamic Loading (`libloading`)

* Backends compiled as shared libraries (`.so` / `.dll` / `.dylib`).
* Loaded at runtime by scanning a directory or using `--backend-path`.
* Must expose a `#[no_mangle] pub extern "C" fn register(reg: &mut PluginRegistry)`.
* Advantages:

    * Pluggable without rebuilding the compiler.
    * Fine-grained versioning of backends.
* Drawbacks:

    * Slight performance penalty on load.
    * Platform-specific complexity.

### 4.3 Hybrid

* Core, low-level backends (Rust, C) statically linked.
* Experimental or third-party backends dynamically loaded.

---

## 5. Built-in Backends

### 5.1 Rust Backend

* Translates TIR into Rust source and invokes `rustc`.

### 5.2 C Backend

* Emits C code, calls platform C compiler (e.g. `gcc`/`clang`).

### 5.3 Assembly Backend

* Emits raw assembly for target CPU.

### 5.4 Zig Backend

* Emits Zig code or leverages Zig’s compilation for cross-compilation.

---

## 6. External Backends

### 6.1 LLVM via `inkwell`

* Provides broad target coverage and optimization.

### 6.2 Cranelift

* JIT-friendly, fast compile-times.
* Ideal for REPLs and rapid prototyping.

---

## 7. Build & Link Strategy

* **Artifact Formats**: object file (`.o`), static lib (`.a`), shared lib, executable, WASM.
* **Linker Interface**:

    * Invoke system linker or custom linker API.
    * Control flags via `Config.link_args`.

---

## 8. Extensibility & Versioning

* **Version Compatibility**:

    * Plugin API semver.
    * Backends declare supported API versions.
* **Registration**:

    * Each backend’s `Cargo.toml` lists `plugin_api = { path = "../plugin_api" }`.

---

## 9. Testing & CI

* **Unit Tests**:

    * Mock `tir::Module`s.
    * Assert produced binaries or object sections.
* **Integration Tests**:

    * End-to-end compile of sample T code.
    * Compare produced behavior across backends.

---

## 10. Errors & Diagnostics

* Backends must translate internal errors to `CompileError::Backend { msg, code_section }`.
* Compiler core aggregates and formats error messages with source context.

---

*Next:* section on **Runtime & Execution Model**, defining how compiled artifacts are executed, sandboxed, or embedded.
