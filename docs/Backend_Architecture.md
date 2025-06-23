# Backend Architecture

This document describes the design and implementation of T‑Lang’s compilation backends, covering both static and dynamic linkage, the plugin API, supported code generators, and extensibility for future targets.

---

## 1. Plugin API Surface

### 1.1. `Backend` Trait

```rust
pub trait Backend {
    /// Returns the backend's unique name (e.g. "llvm", "cranelift").
    fn name(&self) -> &'static str;

    /// Compile the given TIR `Module` into a `CompiledArtifact`.
    fn compile(
        &self,
        module: &tir::Module,
        config: &Config,
    ) -> Result<CompiledArtifact, CompileError>;
}
```

* **Responsibility:** Transform TIR into target-specific bytes.
* **Error Handling:** Return `CompileError` variants for failures—avoid `.unwrap()` or panics.

### 1.2. `PluginRegistry`

```rust
pub struct PluginRegistry {
    backends: HashMap<String, Box<dyn Backend + Send + Sync>>,
}

impl PluginRegistry {
    pub fn register_backend(&mut self, backend: Box<dyn Backend + Send + Sync>);
    pub fn get_backend(&self, name: &str) -> Option<&(dyn Backend + Send + Sync)>;
    pub fn list_backends(&self) -> Vec<&str>;
}
```

* **Registration:** Each backend provides a `register(registry: &mut PluginRegistry)` entrypoint.
* **Lookup:** The compiler core queries by name based on `--backend` flag or default.

### 1.3. Configuration & Artifacts

* **`Config`:** Carries user flags (optimizations, output paths, target triples).
* **`CompiledArtifact`:** Contains `bytes: Vec<u8>` and `format: String` (e.g. "llvm-object").

---

## 2. Backend Discovery & Loading

### 2.1. Static vs. Dynamic Plugins

* **Static Linkage:** Backends compiled into the `compiler` binary, selected at runtime by name.
* **Dynamic Loading (optional):** Using \[`libloading`] to load `*.so` / `*.dll` libraries implementing the `register` symbol.

| Approach                   | Pros                                                           | Cons                                                 |
| -------------------------- | -------------------------------------------------------------- | ---------------------------------------------------- |
| **Static**                 | No runtime dependencies, zero cost of lookup, simpler.         | Binary size grows; need recompile to add backends.   |
| **Dynamic (`libloading`)** | Load/unload without recompilation; plugin third-party targets. | Runtime linkage overhead; symbol/version mismatches. |

### 2.2. Hybrid Strategy

* Compile core backends (LLVM, Cranelift, Rust-to-Rust) statically for guaranteed support.
* Allow optional dynamic backends for niche or third-party targets.

---

## 3. Supported Backends

### 3.1. LLVM

* **Location:** `compiler/src/backends/llvm_backend`
* **Crate:** `inkwell` or `llvm-sys`
* **Features:** Native object files, shared libraries, custom optimization levels.

### 3.2. Cranelift

* **Location:** `compiler/src/backends/cranelift`
* **Crate:** `cranelift-codegen` + `cranelift-module`
* **Use Cases:** JIT, fast codegen, self-hosted incremental compile.

### 3.3. Native Rust

* **Location:** `compiler/src/backends/rust`
* **Strategy:** Convert TIR to Rust AST, emit `rustc`-compatible code, invoke `rustc` via `Command`.

### 3.4. C

* **Location:** `compiler/src/backends/c`
* **Strategy:** Lower TIR to C99+ code, compile with `cc` crate to object or executable.

### 3.5. Zig

* **Location:** `compiler/src/backends/zig`
* **Strategy:** Emit Zig source, leverage Zig’s cross-compilation and packaging.

### 3.6. Assembly

* **Location:** `compiler/src/backends/assembly`
* **Strategy:** Direct TIR → target-specific assembly; useful for embedded, bare-metal.

---

## 4. Backend Extensibility

1. **Creating a New Backend:**

    * Scaffold a crate under `compiler/src/backends/<name>` with `Cargo.toml` and `src/lib.rs`.
    * Implement `Backend` trait and expose `#[no_mangle] pub extern "C" fn register(reg: &mut PluginRegistry)`.
2. **Testing:** Add backend-specific tests under `tests/` directory invoking `compile()` on sample TIR modules.
3. **Documentation:** Update `docs/backend_architecture.md` and `compiler/README.md` with instructions.

---

## 5. Future Roadmap

* **Plugin Versioning:** Embed semantic-version in `register` entrypoint for compatibility checks.
* **Sandboxing:** Optionally isolate dynamic backends in separate processes.
* **Performance Profiling Hooks:** Expose timing and memory usage data per backend.

---

*End of Backend Architecture Document.*
