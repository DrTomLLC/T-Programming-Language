# Plugin System & Extension API Architecture

## 1. Overview

* **Purpose**: Define how T‑Lang can be extended at compile time (front‑end) and codegen time (back‑end) via third‑party or first‑party plugins.
* **Scope**:

    * Language plugins (new syntax/features)
    * Backend plugins (new target codegens)
    * Analysis and transformation plugins (linting, profiling)

## 2. Goals & Constraints

1. **Safety & Isolation**

    * Prevent plugin bugs from crashing the compiler.
    * Provide clear error boundaries.
2. **Discoverability**

    * Auto‑discover plugins in known directories.
    * Command‑line flags to list and configure plugins.
3. **Version Compatibility**

    * Semantic versioning for the plugin API.
    * Mechanism to check at startup whether a plugin is compatible with the compiler version.
4. **Performance**

    * Minimal startup overhead.
    * Lazy initialization where possible.
5. **Usability**

    * Clear documentation for plugin authors.
    * Examples and templates.

## 3. Terminology

* **Host compiler**: The `tlang` binary executing the plugin framework.
* **Plugin**: A dynamically loaded library exposing a well‑defined API.
* **API Surface**: The set of Rust traits, data structures, and helper functions exposed to plugins.
* **Registry**: Internal structure where plugins register their capabilities.
* **Facet**: One area of extension:

    * *Frontend facet*: Parser, TIR transformers.
    * *Backend facet*: Code generators (LLVM, Cranelift, C, Zig, etc.).

## 4. Plugin Lifecycle

1. **Discovery**

    * Scan configured directories (e.g. `$T_HOME/plugins`, workspace `plugins/`).
2. **Loading**

    * Use `libloading` or equivalent to open the dynamic library.
    * Call exported `register` function (no panics).
3. **Registration**

    * Plugin calls `registry.register_frontend(...)`, etc.
    * Registry enforces no duplicate keys.
4. **Initialization**

    * Plugins can define `init` hooks (for global state).
5. **Execution**

    * When compiler reaches extension point, call plugin callbacks.
6. **Shutdown**

    * Plugins may define `cleanup` hooks if needed (release resources).

## 5. API Surface

### 5.1 Core Traits & Types

* `trait Plugin { fn register(reg: &mut PluginRegistry); }`
* `struct PluginRegistry { ... }`
* **Facets**:

    * `trait FrontendPlugin { fn transform(&self, tir: &mut Module) -> Result<(), CompileError>; }`
    * `trait BackendPlugin { fn name(&self) -> &str; fn compile(&self, tir: &Module, config: &Config) -> Result<CompiledArtifact, CompileError>; }`
    * ... (lint plugins, formatter plugins, etc.)

### 5.2 Helper Types

* `enum CompileError { ... }`
* `struct Config { flags: HashMap<String, Value> }`
* `struct CompiledArtifact { bytes: Vec<u8>, format: String }`

## 6. Dynamic vs. Static Linking

* **Static**: Bundled into the compiler binary. No discovery at runtime.
* **Dynamic**: Loaded via shared libraries. Enables third‑party distribution.
* **Hybrid**: Core backends (Rust, LLVM, Cranelift) statically linked; experimental or niche backends loaded dynamically.

## 7. Versioning & Compatibility

* **SemVer** for the plugin API crate.
* **Compatibility check**: Plugins declare `api_version` constant. Registry compares with host.
* **Graceful fallback**: If incompatible, emit a warning and skip plugin.

## 8. Security & Sandboxing

* **Sandboxing considerations**: Plugins run native code; recommend running in CI with minimal privileges.
* **No automatic code execution**: Plugins must explicitly opt into dangerous operations.

## 9. Examples & Templates

* Minimal plugin example in `tlang-plugin-template`:

  ```rust
  #[no_mangle]
  pub extern "C" fn register(reg: &mut PluginRegistry) {
      reg.register_frontend(MyTransform::default());
  }
  ```

## 10. Future Extensions

* **WASM plugins**: Load sandboxed WebAssembly modules.
* **Remote plugins**: Query extension microservices.
* **UI/IDE plugins**: Expose LSP extensions via plugin mechanism.
