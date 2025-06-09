# Section 19: Tooling & Developer Experience

## 19.1 Command‑Line Interface (CLI)
- **Primary executable**: `tlang`
- **Subcommands**:
    - `build` — compile one or more T sources into executable or library
    - `run` — build and immediately execute a T program
    - `test` — discover and run unit/integration tests
    - `fmt` — format T source files according to style guidelines
    - `lint` — static analysis/lint checks (unused code, style, complexity)
    - `doc` — generate API and user documentation
    - `package` — bundle artifacts (archives, installers)
    - `publish` — upload packages to a registry (official or private)
- **Global flags**: `--verbose`, `--quiet`, `--color`, `--config <file>`, `--features <list>`, `--target <triple>`

---

## 19.2 Package & Dependency Management
- **`T.toml` manifest**
  ```toml
  [package]
  name = "my_t_project"
  version = "0.1.0"
  authors = ["You <you@example.com>"]
  description = "…"

  [dependencies]
  foo = "1.2"
  bar = { git = "https://…" }

  [features]
  embedded = []
