# Section 18: Runtime & Standard Library

## 18.1 Overview
The **runtime** provides the minimal support services needed by generated code at execution time (startup/shutdown hooks, memory management, threading, I/O), while the **standard library** supplies high‑level data structures, algorithms, and platform abstractions.
- **Goals**
    - Zero‑cost abstractions where possible
    - Pluggable memory and threading implementations
    - Uniform API across targets (desktop, embedded, WASM)
    - Safe defaults with opt‑in “unsafe” escape hatches

---

## 18.2 Runtime Architecture
- **Bootstrap**
    - Platform‑specific entry (CRT stub, `main()` wrapper)
    - Initialize global state, heap, thread subsystem, signal/interrupt handlers
- **Lifecycle Hooks**
    - `tlang_startup()` → registers constructors, TLS setup
    - `tlang_shutdown()` → run destructors, flush I/O, cleanly terminate threads
- **ABI & Calling Convention**
    - Conform to target’s native ABI for interoperability (C‑FFI, system calls)

---

## 18.3 Memory Management
- **Allocator Interface**
  ```rust
  pub trait Allocator {
      unsafe fn alloc(&self, size: usize, align: usize) -> *mut u8;
      unsafe fn dealloc(&self, ptr: *mut u8, size: usize, align: usize);
  }
