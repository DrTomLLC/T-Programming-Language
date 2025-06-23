# Runtime & Execution Model

This section defines how T‑Lang compiled artifacts are executed, the runtime environment, execution contexts, and integration points for embedding or sandboxing. It covers static vs. dynamic execution, JIT vs. AOT, runtime library support, memory and concurrency models, error handling, instrumentation, and cross‑platform considerations.

---

## 1. Objectives & Scope

* **Purpose**: Ensure compiled T code runs correctly, efficiently, and safely across diverse targets.
* **Scope**:

    * Execution modes: Ahead‑of‑Time (AOT) vs. Just‑In‑Time (JIT).
    * Embedding in host processes (applications, microcontrollers).
    * Sandbox and security: WebAssembly, process isolation, OS sandboxing.
    * Runtime library ("tlangrt") design: memory allocation, I/O, threading.
    * Diagnostics and instrumentation (profiling, tracing).

---

## 2. Execution Modes

### 2.1 Ahead‑of‑Time (AOT)

* **Definition**: Compile TIR to native code or portable bytecode at compile time.
* **Use Cases**: Production binaries, static linking, embedded systems.
* **Advantages**: Maximum performance, smaller runtime footprint.
* **Constraints**: No runtime optimizations based on hot paths.

### 2.2 Just‑In‑Time (JIT)

* **Definition**: Compile or optimize code at runtime, potentially applying profile‑guided optimizations.
* **Use Cases**: REPL, scripting, rapid prototyping, dynamic plugin execution.
* **Engines**: Cranelift JIT backend, LLVM MCJIT.
* **Advantages**: Faster startup for small code, adaptive optimization.
* **Constraints**: Larger memory footprint, added runtime complexity.

### 2.3 Hybrid

* **Ahead‑of‑Time Base + JITed Hotspots**: AOT generate baseline code; JIT compile frequently called functions on demand.
* **Use Cases**: High‑performance servers, long‑running applications.

---

## 3. Runtime Library (`tlangrt`)

### 3.1 Core Responsibilities

* **Memory Management**: Allocator interface (malloc/free abstraction), GC hooks if enabled.
* **Platform Abstraction**: File I/O, networking sockets, threading primitives.
* **Startup & Teardown**: Module initialization, global constructors, cleanup.
* **Foreign Function Interface (FFI)**: Bindings for host language calls (C, Rust, Zig).

### 3.2 Configuration & Features

* **Features**:

    * `tlangrt-std`: Full standard library support (collections, I/O, concurrency).
    * `tlangrt-nostd`: Minimal runtime for embedded (no dynamic memory, direct hardware access).
    * `tlangrt-sandbox`: System calls mediated through a secure layer.
* **Build Flags**: Enable debug mode (bounds checks), instrumentation, sanitizers.

### 3.3 Embedding API

* **Initialization**:

  ```rust
  extern "C" fn tlang_init(config: &RuntimeConfig) -> RuntimeHandle;
  ```
* **Invocation**:

  ```rust
  fn tlang_execute(handle: &RuntimeHandle, entry: &str, args: &[Value]) -> ExecutionResult;
  ```
* **Shutdown**:

  ```rust
  extern "C" fn tlang_shutdown(handle: RuntimeHandle);
  ```

---

## 4. Sandboxing & Isolation

* **WebAssembly Target**:

    * Compile TIR to WASM via Wasmtime or Wasmer.
    * Use WASI for host interfaces.
* **OS-Level Sandboxing**:

    * Linux seccomp filters, macOS sandbox-exec, Windows Job Objects.
* **Process Isolation**:

    * Launch compiled binaries in restricted processes with limited privileges.

---

## 5. Concurrency & Threading Model

* **Lightweight Fibers / Green Threads**:

    * User‑space scheduling for async I/O.
    * Integration with OS threads via a thread pool.
* **OS Threads**:

    * Platform threads for CPU‑bound tasks.
    * Shared mutable state with locks or lock-free structures.
* **Async / Await**:

    * Support `async fn` in T code.
    * Runtime executor in `tlangrt-async`.

---

## 6. Garbage Collection & Memory Safety

* **Ownership & Borrowing**: Leverage static checks in compiled code.
* **Optional GC Modes**:

    * Reference counting (`Rc`/`Arc`-style).
    * Tracing GC for large heap workloads.
* **Integration**:

    * Compile flags to enable/disable GC.
    * Hooks in runtime library for allocation.

---

## 7. Error Handling & Diagnostics

* **Panic vs. Error Results**:

    * `Result<T, E>`-style for recoverable errors.
    * Panic unwinding or abort, depending on target.
* **Stack Traces**:

    * Include source locations via DWARF (AOT) or debug metadata (JIT).
* **Logging & Tracing**:

    * Integrate with `tracing` crate for events.
    * Runtime metrics (allocations, function calls).

---

## 8. Cross‑Platform Considerations

* **Target Triples**: Configurable in `Config.target_triple`.
* **Endianness & Data Layout**: Runtime checks for host CPU.
* **Signal Handling & Exceptions**:

    * Map OS signals to T‑Lang exceptions.

---

## 9. Testing & Validation

* **Unit Tests**: Mock runtime handle, simulate errors.
* **Integration Tests**: Execute real compiled binaries across targets (via QEMU, Wine, mobile emulators).
* **Fuzzing**: Memory safety tests with AFL/LibAFL.

---

## 10. Future Extensions

* **Distributed Execution**: Remote code execution over network.
* **Lightweight VMs**: Container‑like runtime for microservices.
* **Hardware Accelerators**: GPU, TPU offload.

*Next:* **Language Design & Compiler Frontend**, detailing parsing, AST, semantic analysis, and TIR generation.
