Vision & Scope

1. Vision

T‑Lang is a universal, polyglot compiler platform designed to:

Enable seamless authoring and compilation of multi-language codebases.

Provide a unified interface for targeting diverse backends (native, embedded, web, mobile).

Offer first-class extensibility for future languages, runtimes, and deployment environments.

Deliver zero-cost abstractions: end-user artifacts incur no extra overhead beyond static compilation.

Maintain a small trusted core; all language support lives in stable, versioned plugins.

2. Scope & Boundaries

In Scope

Core Compiler Infrastructure

Parsing, type-checking, intermediate representation (TIR).

Plugin-based codegen backends for at least: Rust, C, Assembly, Zig, Cranelift.

Plugin API

Stable plugin_api crate with traits: Backend, Config, CompiledArtifact, PluginRegistry.

Dynamic loading support (via libloading) and static linking options.

Supported Platforms & Targets

OS: Windows, Linux, macOS, iOS, Android, embedded OS.

Hardware: x86_64, ARM32/64, RISC-V, microcontrollers.

Application types: desktop, web, mobile, embedded firmware.

Tooling & Integration

tlang CLI for project scaffolding, building, testing.

LSP server (tlang-lsp) for IDE/editor integration.

Out of Scope (for initial versions)

JIT-based execution engines (beyond optional Cranelift).

Garbage-collected or interpreter runtimes.

Full standard libraries for every language (plugins may provide their own).

Managed languages (e.g., Java, C#) — focus on native compilation.

3. Stakeholders & Audiences

Language Designers: want a consistent IR and plugin model to target new languages.

Systems Engineers: need to build embedded or high-performance apps across platforms.

Polyglot Developers: integrate multiple languages in one unified build system.

Tooling Authors: extend T‑Lang IDE support, analyzers, profilers.

4. High-Level Use Cases

Single-Language Build: write Rust code and compile to native binaries via the Rust backend.

Cross-Language Linking: compile C and Zig modules, then link them into one executable.

Embedded Firmware: compile C, Assembly, and Rust into a microcontroller image with --features embedded.

Mobile Targets: produce Android .so libraries and iOS static frameworks from T‑Lang sources.

Plugin Authoring: create a new .so backend for a new language (e.g., Julia) without modifying core.

Next: Architecture Overview

We'll detail the core components, data flow, plugin loading, and directory layouts