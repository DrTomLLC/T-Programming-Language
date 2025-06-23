# 🅣 T Programming Language

**T** is a fast, small, safe, and future-proof programming language built to solve modern software development pain points without the complexity, bloat, or dependency chaos seen in most ecosystems today. Whether you're scripting, building operating systems, creating web apps, or working with embedded hardware, T is your all-in-one tool — minimal in size, maximum in power.

---

## ✨ Purpose
T was created to be **the last language you need** — blending the speed of C, the safety of Rust, the clarity of Python, and the flexibility of Zig. It is designed to:

- Eliminate dependency hell
- Be highly readable and maintainable
- Offer total safety and memory control when you need it
- Support scripting and compiled use cases seamlessly
- Provide full system-level access for embedded, kernel, and real-time work
- Build blazing-fast web backends, GUIs, and native apps with ease

---

## 🚀 What Makes T Special

### ✅ Simplicity
- Python-like syntax, human-readable
- Whitespace aware, optional braces
- Strong type inference, optional annotations

### ✅ Performance
- Faster than Rust in many scenarios (with lower compile times)
- Ahead-of-Time (AOT) compiler for full optimization
- Just-In-Time (JIT) interpreter for near-native scripting

### ✅ Safety
- Rust-like ownership and borrowing system
- Null safety and strict bounds checking
- Powerful unsafe block analysis (even in scripting mode)
- Built-in static analysis assistant (Tippy)

### ✅ Universality
- Use T for:
  - Systems programming (OS dev, drivers, embedded)
  - Web APIs and full-stack apps (WASM included)
  - Shell scripting and automation
  - GUI development (native, cross-platform)
  - Game engines and simulations
  - Data processing and scientific computing

### ✅ Seamless Interoperability
- Native interop with Rust, Zig, Roc, V, C, C++, Assembly
- High-level bindings to Python, Go, JS, Java/Kotlin, Julia, R, etc.
- Easy plugin integration and embedding

### ✅ Built-In Power Tools
- `Targo`: Dependency and build system (zero dependency hell)
- `Tippy`: Smart, friendly, real-time linter and advisor
- `Tup`: Toolchain updater (switch versions, rollback safely)
- `t`: One CLI to run, compile, script, build, test, format, and update

### ✅ Modular & Zero-Cost
- All major systems are opt-in modules: GUI, DB, Net, Web, Sys, etc.
- Unused features = zero impact on size or runtime
- Ideal for minimal containers, microcontrollers, large apps alike

---

## 📚 Core Language Features

- `fn` for function declarations
- Pattern matching
- First-class functions and closures
- Async/await support
- Powerful macros and metaprogramming (safe by default)
- Compile-time evaluation (`comptime`) support
- Declarative error handling
- Native GUI toolkit, database interface, networking stack

---

## 📦 Built-in Modules (Modular, Optional)

| Module | Description |
|--------|-------------|
| `gui`  | Cross-platform, native UI toolkit |
| `net`  | TCP/UDP/HTTP/WebSockets/WebRTC support |
| `db`   | Safe, type-checked SQL/NoSQL access |
| `web`  | Web server framework + WebAssembly support |
| `sys`  | Low-level OS interfaces, file access, shell integration |

---

## 🧱 Architecture

- Lexer → Parser → AST → Type Checker → Optimizer → Codegen
- Modular compiler backend (custom, LLVM, or Cranelift)
- Optional runtime, pluggable GC, and zero runtime overhead by default
- Unified scripting/compiled architecture

---

## 🧪 Tippy: Smart Analysis Engine

- Lints in real time (REPL or compiled)
- Fixes unsafe blocks
- Detects logic issues, unused code, performance hints
- Suggests fixes with examples

---

## 🔄 Tup: Toolchain Updater

- Manages T versions (stable/beta/nightly)
- Rolls forward/back instantly
- Updates T CLI, compiler, stdlib, and tooling

---

## 🔧 Targo: Build and Dependency Manager

- Zero configuration required
- Reproducible builds via lockfiles
- Uses local or decentralized repositories (no crates.io needed)
- Supports vendoring and version pinning

---

## 🌍 Language Interoperability

T works natively with:
- Rust, Zig, V, Roc, C, C++, Assembly
- Python, Go, Java, Kotlin, JS, Julia, R, PHP, .NET (via bindings or WASM/JNI/FFI)
- Clean FFI layer and tools for exporting/importing across boundaries

---

## 📖 Documentation

See `/docs` for:
- Getting Started
- Syntax Overview
- Memory Model
- Unsafe Block Design
- Modules: GUI, Web, Net, DB, Sys

---

## 💡 Philosophy

> "Small. Fast. Safe. For everything. Forever."

- No bloat. No surprises.
- Everything designed to work together.
- One CLI. One language. One vision.

---

## 👨‍💻 Contributing

T is open to contributors who share the vision of a small, powerful, respectful language built for real-world needs.

See: [CONTRIBUTING.md](./CONTRIBUTING.md)

---

## 📜 License

T is open for use and contribution but governed by the [T Language License](./LICENSE) to protect the vision and original authorship. Commercial use is allowed with required credit and sharing of core improvements.

---

## 🧪 Example Code

```t
fn main():
    print("Hello, world!")
```

---

T is being built to do everything a language should — and nothing it shouldn’t.

Minimal. Powerful. Done right.

