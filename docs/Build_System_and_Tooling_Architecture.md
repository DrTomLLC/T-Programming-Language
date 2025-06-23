# Build System & Tooling Architecture

## 1. Overview

The build system and tooling layer orchestrates compilation, testing, packaging, and distribution of T‑Lang projects. It provides:

* **Project configuration** and manifest conventions
* **Dependency resolution** across languages and backends
* **Incremental builds** and caching
* **Plugin discovery** and runtime wiring
* **Cross‑language tooling** (formatters, linters, docs)
* **CI/CD integration** and release management
* **IDE/editor support** (language server, debug adapters)

---

## 2. Design Goals

* **Unified UX:** Single `tlang` CLI for all tasks (build, test, run, deploy).
* **Extensible:** Support custom build plugins for new languages/backends.
* **Fast:** Incremental and parallel builds with minimal overhead.
* **Reproducible:** Deterministic artifact generation for releases.
* **Cross‑language:** Allow polyglot dependencies and tooling.
* **Configurable:** Profiles for embedded, debug, release, mobile.

---

## 3. Project Manifest & Configuration

### 3.1 `tlang.toml`

* **Metadata:** name, version, authors, description
* **Dependencies:** semantic version ranges, local path, git
* **Features & profiles:** embedded, mobile, experimental
* **Workspace:** multi‑crate/project support

### 3.2 CLI Commands

* `tlang new <project>`
* `tlang build [--profile <name>]`
* `tlang test [--backend <name>]`
* \`tlang run \[--args]
* `tlang fmt`, `tlang lint`, `tlang doc`
* `tlang publish`, `tlang install`

---

## 4. Dependency Management

* **Resolution:** Lockfile (`tlang.lock`) for reproducibility.
* **Graph:** Multi‑language DAG; resolves version conflicts.
* **Local overrides:** Path and Git-based dep searches.

### 4.1 Backend Plugins

* Built as dynamic libraries (or static once frozen)
* Discovered via manifest `plugins.backends`
* Verified against API version

---

## 5. Build Pipeline

1. **Resolve deps →** 2. **Generate IR** for each crate/language → 3. **Backend codegen** → 4. **Linking** → 5. **Artifact packaging**

* **Incremental mode:** Track file hashes and IR fingerprints
* **Parallelism:** Per crate and per backend tasks
* **Caching:** On-disk cache keyed by content hashes

---

## 6. Cross‑Language Tooling

* **Formatters:** Built‑in `t fmt` or language‑specific via plugins
* **Linters:** AST and IR-level checks, extensible rules
* **Doc generator:** Combined docs for polyglot code
* **Benchmark harness:** `t bench` across languages/backends

---

## 7. Testing & CI/CD

* **Test harness:** `#[test]` annotations across languages
* **Coverage:** Multi-language coverage report
* **CI integration:** GitHub Actions, GitLab CI templates
* **Release pipelines:** Automated version bump, tag, publish

---

## 8. IDE & Editor Integration

* **Language Server Protocol** (`tlang-lsp`): code completion, diagnostics
* **Debug Adapter Protocol** (`tlang-dap`): breakpoints, watches
* **Editor plugins:** VSCode, Neovim, IntelliJ

---

## 9. Packaging & Distribution

* **Binary crates:** Executable distributions, installers
* **Library crates:** Push to central registry, language‑specific repos
* **Docker:** Standard container images

---

## 10. Security & Reproducible Builds

* **Deterministic output:** Fixed timestamps, ordered metadata
* **Signing:** Artifact signatures via GPG
* **Sandboxed builds:** Isolated environments for untrusted deps

---

*Next up: **Runtime & Standard Library Architecture** was completed; after reviewing, we’ll move on to **Language Specification & Compiler Architecture**.*
