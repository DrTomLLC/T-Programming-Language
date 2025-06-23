# Code Generation & Backend Integration

This section details how T‑Lang’s intermediate representation (TIR) is translated into executable or linkable artifacts via multiple backends. It defines the plugin API for codegen, static vs. dynamic linking choices, supported targets, and backend-specific responsibilities.

---

## 1. Objectives & Scope

* **Purpose**: Provide a flexible, high-performance code generation pipeline supporting multiple output formats and targets, while exposing extension points for custom backends.
* **Scope**:

    * Backend plugin registration and discovery
    * Static and dynamic backend loading
    * Common codegen interface (`Backend` trait)
    * Lowering from TIR to each target's IR or machine code
    * Target feature configuration, optimization levels
    * Artifact packaging (object files, shared libraries, executables)
    * Cross-compilation and multi-target support
    * Diagnostics and error propagation from backends

---

## 2. Supported Backends

1. **Rust (native)**
2. **Cranelift** (JIT & AOT via Wasmtime)
3. **LLVM** (AOT via `inkwell`)
4. **C** (emit C code for portability)
5. **Zig** (emit Zig code or use Zig compiler as backend)
6. **Assembly** (emit raw assembly for custom toolchains)
7. **Embedded-specific** (multi-architecture microcontroller support)

Each backend lives in `compiler/src/backends/<name>/` and implements the common `Backend` interface.

---

## 3. Backend Plugin API

```rust
pub trait Backend {
    /// Unique name, e.g. "llvm", "cranelift".
    fn name(&self) -> &'static str;

    /// Lower a TIR module into target-specific format.
    fn compile(
        &self,
        module: &tir::Module,
        config: &Config,
    ) -> Result<CompiledArtifact, CompileError>;
}

/// Registry for discovered backends.
pub struct PluginRegistry {
    backends: HashMap<String, Box<dyn Backend>>,
}

impl PluginRegistry {
    pub fn register_backend(&mut self, backend: Box<dyn Backend>);
    pub fn get(&self, name: &str) -> Option<&dyn Backend>;
}
```

* **Registration**: Each backend crate exposes `#[no_mangle] pub extern "C" fn register(reg: &mut PluginRegistry)` that instantiates and registers itself.
* **Configuration** (`Config`): includes target triple, CPU, features, optimization level, output directory, dynamic/static flag.

---

## 4. Static vs. Dynamic Loading

* **Static linking**:

    * Compile all backends into the `tlangc` executable.
    * Pros: zero-dependency at runtime, predictable behavior.
    * Cons: larger binary, rebuild required when adding/removing backends.

* **Dynamic loading** (`libloading`):

    * Load backend shared libraries (`.so`, `.dll`, `.dylib`) at runtime.
    * Pros: pluggable post-deployment, smaller core binary.
    * Cons: versioning complexity, startup overhead.

* **Hybrid approach**:

    * Statically include core/backends that are critical (Rust, LLVM, Cranelift).
    * Dynamically load less-common or user-provided backends (Custom C, Zig, Embedded).

---

## 5. Code Generation Workflow

1. **Configure**: user selects backend(s) via CLI flag (`--backend llvm`, `--backend cranelift`, or `all`).
2. **Load**: statically or dynamically register backends in `PluginRegistry`.
3. **Lower TIR**:

    * Common pre-codegen passes (SSA cleanup, CFG canonicalization).
    * Backend-specific prelude (e.g. declare runtime support functions).
4. **Compile**:

    * Invoke `backend.compile(module, &config)`.
    * Capture artifacts: object bytes, metadata (format, target triple).
5. **Link/Package**:

    * For object outputs: driver invokes system linker or Zig to produce final executable/library.
    * For native codegen: emit shared library if requested.
6. **Emit Diagnostics**: collect errors/warnings from codegen and emit via `Diagnostic` API.

---

## 6. Individual Backend Responsibilities

### 6.1 Rust Backend

* Directly generate and compile Rust source via `rustc` or `cargo`.
* Leverage Rust’s macro system for runtime support.

### 6.2 Cranelift Backend

* Translate TIR to Cranelift IR.
* Leverage `cranelift-codegen` for JIT and AOT.
* Manage function trampolines and relocations for dynamic linking.

### 6.3 LLVM Backend

* Lower TIR to LLVM IR.
* Use `inkwell` for IR construction and `TargetMachine` to emit object code.
* Expose LLVM optimization passes configuration.

### 6.4 C Backend

* Emit portable C source.
* Provide headers and runtime support library in C.
* Support cross-compilation via configured C toolchain.

### 6.5 Zig Backend

* Emit Zig source or directly call Zig compiler API for codegen.
* Benefit from Zig’s cross-compilation and dependency management.

### 6.6 Assembly Backend

* Emit raw assembly for user-provided assembler/linker.
* Allow fine-grained control over instructions and calling conventions.

### 6.7 Embedded Backend

* Support additional sections (interrupt vectors, linker scripts).
* Integrate with LLVM/Cranelift/AOT for microcontroller targets.
* Provide board-specific runtime glue.

---

## 7. Cross-Compilation & Multi-Target Support

* **Target triples**: `x86_64-unknown-linux-gnu`, `armv7-unknown-linux-gnueabihf`, `thumbv7em-none-eabi`, `aarch64-apple-ios`, etc.
* **Feature matrices**: specify CPU features (`sse4.2`, `neon`) per backend.
* **Build matrix**: CI pipelines to exercise all major backends on supported targets.

---

## 8. Testing & Validation

* **Backend unit tests**: isolate codegen of simple TIR constructs.
* **Integration tests**: compile end-to-end T programs to binaries via each backend and execute.
* **Performance benchmarks**: compare code size and runtime speed across backends.
* **Fuzzing**: feed random TIR patterns to backends to detect crashes.

---

## 9. Future Extensions

* **Link-Time Optimization (LTO)** for LLVM & Zig.
* **Incremental codegen**: reuse cached object files when TIR unchanged.
* **Distributed codegen**: offload heavy code generation to remote workers.
* **Backend-specific plugin hooks**: allow injection of custom IR passes.

---

*Next:* **Runtime & Standard Library Architecture** section.
