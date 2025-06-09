# Section 23: Scripting & Interactive Mode

## 23.1 Overview
While T‑Lang shines at ahead‑of‑time compilation, it also offers a modern, first‑class scripting experience:

- **Shebang & One‑File Scripts**
    - `#!/usr/bin/env tlang` at top of `.t` files
    - No boilerplate required: scripts execute immediately

- **Interactive REPL**
    - `tlang repl` launches a live prompt
    - Supports multi‑line editing, history, tab‑completion
    - Lightweight “scratch” sessions for experimentation

- **JIT & Incremental Compilation**
    - Hot‑reload individual functions or modules
    - Uses Cranelift/LLVM under the hood to deliver near‑native speed

- **Polyglot Embedding**
    - Inline snippets of any supported backend language (Rust, C, Zig, Assembly, etc.)
    - Instantly compile & link to your script via `tlang embed rust { … }`

- **Rich Standard Library for Automation**
    - Built‑in modules for filesystem, HTTP, JSON/YAML, databases
    - Async/await for non‑blocking I/O in scripts
    - Shell‑style pipelines (`|`, `&&`, `||`, globbing)

- **Plugin Hooks & DSLs**
    - Write domain‑specific mini‑languages on top of T’s AST
    - Plugins can inject new syntax or transform scripts at parse time

## 23.2 Script Packaging & Distribution
- **Single‑File Bundles**
    - `tlang pack --standalone myscript.t` → self‑contained executable
- **Module Registry**
    - Publish scripts & libraries to a central index
    - `tlang install net/request@1.2.3`

## 23.3 Cross‑Platform Scripting
- Write once, run anywhere: same script runs on Windows, Linux, macOS, mobile, embedded
- Auto‑detect platform and dispatch to the correct backend (e.g. uses Zig for small embedded targets)

## 23.4 IDE & Editor Integration
- Language Server Protocol support out of the box (`tlang-lsp`)
- Live error reporting, auto‑formatting, and code lenses for scripts
- Rich debugging for both compiled binaries and live REPL sessions

## 23.5 Use Cases & Best Practices
- **DevOps & CI**: concise deployment scripts that compile to fast, self‑contained binaries
- **Data Processing**: rapid prototyping in REPL, then “freeze” into optimized pipelines
- **Build Systems**: author `Tbuild.toml` as scriptable manifests instead of YAML/JSON
- **Teaching & Prototyping**: immediate feedback in REPL lowers barrier to entry

---

## 23.6 Security & Sandboxing
- **Capability‐Based Permissions**
    - Grant scripts only the capabilities they need (filesystem, network, FFI)
- **Isolated Execution**
    - Run untrusted scripts inside lightweight OS‐level containers or WASM sandbox
- **Audit Logging**
    - Automatic provenance tracking of actions performed during script run

## 23.7 Profiling, Tracing & Telemetry
- **Built‑In Profiler**
    - `tlang profile myscript.t` generates flamegraphs and hotspot reports
- **Event Tracing**
    - Instrument script sections with `trace!("label")` for fine‑grained diagnostics
- **Telemetry Hooks**
    - Export metrics via Prometheus or OpenTelemetry during script execution

## 23.8 Dependency Isolation & “Virtual Environments”
- **Per‑Project Lockfiles**
    - `tlang env init` creates isolated deps & cache in `tlang.lock`
- **Local Module Paths**
    - Override global registry with a `./tlang_modules/` directory
- **Reproducible Builds**
    - Lockfile+checksum enforcement for byte‑for‑byte identical script artifacts

## 23.9 Concurrency & Parallelism in Scripts
- **Structured Concurrency**
    - `spawn` / `await` primitives with deterministic cancellation
- **Data‑Parallel Pipelines**
    - Map/Reduce helpers for filesystem and network streams
- **Thread‑Pool Configuration**
    - Tune number of worker threads via `--threads N` or env var `TLANG_THREADS`

## 23.10 GUI & OS Automation
- **Cross‑Platform Clipboard & Mouse/Keyboard APIs**
    - `ui::click(x, y)`, `ui::type("hello")`, `clipboard::read()`
- **Window Management**
    - List, resize, and focus windows across Windows/macOS/Linux
- **OCR & Image Recognition**
    - Basic on‑screen text/image matching for tests and robots

## 23.11 CI/CD & Workflow Integration
- **Git Hooks**
    - `tlang plugin install git-hooks` provides pre‑commit & CI test runners
- **Webhooks & Cloud Tasks**
    - Trigger scripts on HTTP events via built‑in `http::server` module
- **Containerized Execution**
    - `tlang run --docker myscript.t` spins up an ephemeral container

---

> **Note:** All scripting features share the same “frozen” compiler code paths—your end artifacts are unaffected by whether you called `tlang build` or ran a one‑off script.

```markdown
