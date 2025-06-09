# Section 08: Plugin API & Backend Interface

## 8.1 Overview

T‑Lang’s compilation pipeline is extensible via a **Plugin API** that allows third‑party and first‑party backends to be registered dynamically or statically. This section defines:

* The core traits and data types exposed to plugins.
* How backends are discovered and loaded at runtime.
* Versioning, stability guarantees, and compatibility.

## 8.2 Core API Surface

### 8.2.1 `Backend` Trait

```rust
/// All codegen backends must implement this trait.
pub trait Backend {
    /// Returns the backend's unique name (e.g., "llvm", "cranelift").
    fn name(&self) -> &str;

    /// Given a lowered Module and Config, produce a CompiledArtifact.
    fn compile(&self, module: &tir::Module, config: &Config)
        -> Result<CompiledArtifact, CompileError>;
}
```

* **Stability**: Methods may only gain optional parameters via defaulted traits; breaking changes require major version bump.

### 8.2.2 `Config` & `CompiledArtifact`

* `Config` carries flags (optimization level, target triple, features).
* `CompiledArtifact` encapsulates emitted bytes (object, wasm, archive) and metadata.

```rust
pub struct Config {
    pub target: TargetTriple,
    pub opt_level: OptLevel,
    pub feature_flags: Vec<String>,
}

pub struct CompiledArtifact {
    pub bytes: Vec<u8>,
    pub format: ArtifactFormat,
}
```

## 8.3 Plugin Registration

### 8.3.1 Static Registration

Backends compiled into the T‑Compiler binary register themselves in a central registry at startup:

```rust
#[no_mangle]
pub extern "C" fn register(reg: &mut PluginRegistry) {
    reg.register_backend(Box::new(LlvmBackend::new()?));
    // ... other built‑in backends
}
```

* **Initialization order**: Deterministic by crate dependency order.

### 8.3.2 Dynamic Loading (Optional)

Using [libloading], backends can be shipped as separate `.so`/`.dll`/`.dylib` files:

1. **Discovery**: scan `--plugin-dir` for files matching `libtlang_backend_*`.
2. **Loading**: load library via `libloading::Library`, lookup `register` symbol.
3. **Safety**: All symbols must adhere to `extern "C"` ABI.

> **Pros**: Smaller core binary; plugins can be updated independently.
> **Cons**: Slight startup overhead; ABI stability must be carefully maintained.

## 8.4 `PluginRegistry`

```rust
pub struct PluginRegistry {
    backends: HashMap<String, Box<dyn Backend>>,
}

impl PluginRegistry {
    pub fn register_backend(&mut self, backend: Box<dyn Backend>) {
        let name = backend.name().to_string();
        self.backends.insert(name, backend);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Backend> {
        self.backends.get(name).map(Box::as_ref)
    }
}
```

* Ensures **unique** backend names; duplicates result in a descriptive error.

## 8.5 Versioning & Compatibility

* The `plugin_api` crate uses semantic versioning. Minor bumps add non‑breaking APIs; major bumps may break.
* Plugins must depend on the exact same `plugin_api` version as the host compiler.
* Consider a minimal shim layer to support multiple minor versions if dynamic loading is critical.

## 8.6 Testing & Validation

* Each backend crate includes an integration test that registers itself and performs a compile of a trivial TIR module.
* The core compiler test suite pulls every registered backend and verifies:

    * Successful registration
    * Compilation round‑trip of an empty or simple module

## 8.7 Future Extensions

* **Feature Flags**: Backends may query `Config.feature_flags` for experimental options.
* **Backend Metadata**: Expose descriptions, supported file types, and version strings via an optional API.
* **Hot Reload**: In long‑running services (e.g., LSP), support unloading and reloading plugin libraries.

---

[libloading]: https://crates.io/crates/libloading
