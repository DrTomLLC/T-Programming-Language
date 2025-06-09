# Section 15: Toolchain & Developer Workflow

## 15.1 Command-Line Interface & Invocation

* `tlangc` entrypoint: subcommands (build, run, test, doc), positional arguments
* Flags: `--target`, `--features`, `--release` vs `--debug`, verbosity levels
* Plugin loading options: `--backend`, `--plugin-path`, dynamic vs static linking

## 15.2 Configuration & Manifest Files

* `Tlang.toml` project manifest: package metadata, dependencies, edition
* Workspace configuration: multi-crate support, path overrides
* Formatters & linters: `.tlangfmt.toml`, `.tlanglint.toml` conventions

## 15.3 Plugin & Backend Management

* Built-in vs external backends: discovery, registration order, version checks
* Installing/removing plugins: package registry, local development workflows
* Environment variables and search paths for plugin loading

## 15.4 Build System & Toolchain Integration

* Cargo integration for Rust toolchain, interoperability guides
* Support for CMake, Bazel, Makefiles and custom build scripts
* Cross-compilation workflows: sysroot management, target specification

## 15.5 IDE & Editor Integration

* Language Server Protocol (LSP) server: architecture, message flow
* Editor plugins: JetBrains, VSCode, Neovim, Emacs feature set mapping
* Debug Adapter Protocol (DAP) support and debugger configuration

## 15.6 Continuous Integration & Automated Testing

* CI/CD pipeline templates: GitHub Actions, GitLab CI, Jenkins
* Test harness integration: unit tests, integration tests, golden tests
* Performance benchmarks in CI: threshold-based failure, reporting

## 15.7 Documentation & Examples

* Documentation generator: doc comment parsing, website generation
* Versioned docs, search, theming, hosting strategies
* Example projects and tutorials: git submodules, code samples repository

## 15.8 Future Tooling Enhancements

* Graphical IDE & Dev UX: plugin marketplace, extensions API
* Telemetry & Analytics: anonymous usage reporting, crash diagnostics
* GUI debugging & profiling tools: integrated flamegraph viewers
