# Section 11: Tooling & Build System

## 11.1 Overview

Describe the suite of tools and build infrastructure that support T‑Lang development, including the package manager, compiler driver, build pipelines, and extensibility.

## 11.2 Package Management

* **Package Manifest**: `T.toml` format, fields, workspace support.
* **Dependency Resolution**: Version requirements, semver policy, conflict handling.
* **Registries & Sources**: Official registry, mirrors, git/path dependencies.
* **Publishing Workflow**: Publishing, yanking, deprecation.

## 11.3 Build Invocation & Configuration

* **Compiler Driver (`tlangc`)**: Command‑line flags, profiles (dev, release, embedded).
* **Build Scripts**: Custom pre/post‑build hooks, build dependencies.
* **Features & Flags**: Feature flags, conditional compilation, profile configuration.

## 11.4 Module & Workspace Layout

* **Source Tree Conventions**: `src/lib.t`, `src/main.t`, workspaces, examples.
* **Module System**: Naming conventions, module imports, visibility rules.
* **Workspaces**: Multi‑crate projects, shared dependencies, overriding features.

## 11.5 REPL & Interactive Mode

* **REPL Interface**: Read–Eval–Print Loop behavior, line editing, history.
* **Scripting Support**: Shebang approach, script execution, quickstarts.

## 11.6 Debugging & Profiling Tools

* **Debugger Integration**: Source‑level debugging, DWARF support, remote debug.
* **Profiling Tools**: Built‑in instrumentation, flame graphs, sampling.
* **Trace & Log Collection**: Tracing macros, structured logging, external sinks.

## 11.7 Language Server Protocol (LSP)

* **LSP Server**: Code completion, diagnostics, refactorings.
* **IDE Integrations**: VS Code, Neovim, JetBrains, Emacs.

## 11.8 Build Extensibility & Plugins

* **Custom Cargo Plugins**: Plugin API for build extensions, lifecycle hooks.
* **Third‑Party Tooling**: Linters, formatters, code generators.

## 11.9 Continuous Integration & Deployment

* **CI Pipelines**: Recommended workflows for GitHub Actions, GitLab CI.
* **Artifact Publishing**: Binary packaging, container images, version tagging.

## 11.10 Future Enhancements

* **Incremental Compilation**: Fine‑grained rebuilds, cache invalidation.
* **Distributed Builds**: Remote caching, cloud build farms.
* **IDE Deep Integrations**: Live error checking, in‑editor execution.
